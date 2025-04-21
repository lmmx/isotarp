use crate::coverage::analysis::run_analysis;
use crate::coverage::tarpaulin::list_tests;
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

pub fn execute_analyze_command(
    package: &str,
    tests: Option<Vec<String>>,
    output_dir: &PathBuf,
    report: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create the output directory if it doesn't exist
    std::fs::create_dir_all(output_dir)?;

    let test_names = match tests {
        Some(specified_tests) => specified_tests,
        None => {
            println!("No specific tests provided, analyzing all tests...");
            list_tests(package)?
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
    let mut tests_by_unique: Vec<_> = analysis.tests.iter().collect();
    tests_by_unique.sort_by(|a, b| b.1.unique_covered_lines.cmp(&a.1.unique_covered_lines));

    println!("\nTests ranked by unique line coverage:");
    for (test_name, stats) in tests_by_unique {
        let unique_pct = if stats.total_covered_lines > 0 {
            (stats.unique_covered_lines as f64 / stats.total_covered_lines as f64) * 100.0
        } else {
            0.0
        };

        println!(
            "  {}: {} unique lines ({:.1}% of {} total covered lines)",
            test_name, stats.unique_covered_lines, unique_pct, stats.total_covered_lines
        );
    }

    // Find tests with no unique coverage
    let no_unique = analysis
        .tests
        .iter()
        .filter(|(_, stats)| stats.unique_covered_lines == 0)
        .map(|(name, _)| name)
        .collect::<Vec<_>>();

    if !no_unique.is_empty() {
        println!("\nTests with NO unique coverage:");
        for test in no_unique {
            println!("  {}", test);
        }
    }

    // Final cleanup just to be extra sure
    cleanup_target_dirs(output_dir, &test_names);

    Ok(())
}
