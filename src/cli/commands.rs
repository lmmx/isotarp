use crate::coverage::analysis::run_analysis;
use crate::coverage::tarpaulin::list_tests;
use crate::resolve::resolve_test_patterns;
use crate::utils::io::save_analysis;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "isotarp",
    about = "Analyze test coverage at the individual test level",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all tests in a package
    List {
        /// Package name
        #[arg(short, long)]
        package: String,
    },

    /// Run analysis on all tests or specific tests
    Analyze {
        /// Package name
        #[arg(short, long)]
        package: String,

        /// Specific tests to analyze (if not provided, all tests will be analyzed)
        #[arg(short, long)]
        tests: Option<Vec<String>>,

        /// Output directory for intermediate results
        #[arg(short, long, default_value = "isotarp-output")]
        output_dir: PathBuf,

        /// Output file for the analysis result
        #[arg(short, long, default_value = "isotarp-analysis.json")]
        report: PathBuf,
    },
}

/// Clean up target directories to save disk space
fn cleanup_target_dirs(output_dir: &PathBuf, test_names: &[String]) {
    println!("Cleaning up temporary target directories...");

    for test_name in test_names {
        let test_dir = output_dir.join(test_name.replace("::", "/"));
        let target_dir = test_dir.join("tarpaulin-target");

        if target_dir.exists() {
            match fs::remove_dir_all(&target_dir) {
                Ok(_) => (),
                Err(e) => println!(
                    "Warning: Failed to clean up '{}': {}",
                    target_dir.display(),
                    e
                ),
            }
        }
    }
}

pub fn execute_list_command(package: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tests = list_tests(package)?;
    println!("Found {} tests in package '{}':", tests.len(), package);
    for test in tests {
        println!("  {}", test);
    }
    Ok(())
}

// Updated execute_analyze_command function
pub fn execute_analyze_command(
    package: &str,
    tests: Option<Vec<String>>,
    output_dir: &PathBuf,
    report: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create the output directory if it doesn't exist
    std::fs::create_dir_all(output_dir)?;

    let available_tests = list_tests(package)?;

    let test_names = match tests {
        Some(specified_tests) => {
            let (selected_tests, invalid_patterns) =
                resolve_test_patterns(&available_tests, &specified_tests);

            // Report invalid patterns
            if !invalid_patterns.is_empty() {
                println!("Warning: The following test patterns did not match any tests:");
                for pattern in &invalid_patterns {
                    println!("  {}", pattern);
                }
                if selected_tests.is_empty() {
                    return Err("No matching tests to analyze".into());
                }
                println!("Continuing with {} matching tests.", selected_tests.len());
            }

            selected_tests
        }
        None => {
            println!("No specific tests provided, analyzing all tests...");
            available_tests
        }
    };

    println!(
        "Analyzing {} tests in package '{}'",
        test_names.len(),
        package
    );

    // Run the analysis with cleanup in case of error
    let result = run_analysis(package, &test_names, output_dir);

    // Handle the result
    let analysis = match result {
        Ok(analysis) => analysis,
        Err(e) => {
            // Do a final cleanup in case there wasn't one
            cleanup_target_dirs(output_dir, &test_names);
            return Err(Box::new(e));
        }
    };

    // Save the analysis result
    save_analysis(&analysis, report)?;

    println!("Analysis complete! Results saved to {}", report.display());

    // Print a summary of the results
    let tests_by_unique: Vec<_> = analysis.tests.iter().collect();

    // Separate tests into categories
    let mut tests_with_unique_coverage = Vec::new();
    let mut tests_with_zero_unique_coverage = Vec::new();
    let mut tests_with_zero_total_coverage = Vec::new();

    for (test_name, stats) in tests_by_unique {
        if stats.unique_covered_lines > 0 {
            tests_with_unique_coverage.push((test_name, stats));
        } else if stats.total_covered_lines > 0 {
            tests_with_zero_unique_coverage.push((test_name, stats));
        } else {
            tests_with_zero_total_coverage.push((test_name, stats));
        }
    }

    // Sort tests with unique coverage by number of unique lines (descending)
    tests_with_unique_coverage
        .sort_by(|a, b| b.1.unique_covered_lines.cmp(&a.1.unique_covered_lines));

    // Display tests with unique coverage
    if !tests_with_unique_coverage.is_empty() {
        println!("\nTests with unique line coverage:");
        for (test_name, stats) in &tests_with_unique_coverage {
            let unique_pct =
                (stats.unique_covered_lines as f64 / stats.total_covered_lines as f64) * 100.0;
            println!(
                "  {}: {} unique lines ({:.1}% of {} total covered lines)",
                test_name, stats.unique_covered_lines, unique_pct, stats.total_covered_lines
            );
        }
    }

    // Display tests with no unique coverage but some total coverage
    if !tests_with_zero_unique_coverage.is_empty() {
        println!(
            "\nTests with NO unique coverage (but covering {} total lines):",
            tests_with_zero_unique_coverage
                .iter()
                .map(|(_, stats)| stats.total_covered_lines)
                .sum::<u32>()
        );
        for (test_name, stats) in &tests_with_zero_unique_coverage {
            println!(
                "  {}: 0 unique lines (covers {} total lines)",
                test_name, stats.total_covered_lines
            );
        }
    }

    // Display tests with zero total coverage
    if !tests_with_zero_total_coverage.is_empty() {
        println!("\nTests with NO code coverage:");
        for (test_name, _) in &tests_with_zero_total_coverage {
            println!("  {}", test_name);
        }
    }

    // Final cleanup just to be extra sure
    cleanup_target_dirs(output_dir, &test_names);

    Ok(())
}
