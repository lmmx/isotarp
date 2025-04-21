use crate::coverage::tarpaulin::run_isolated_test_coverage;
use crate::types::errors::Error;
use crate::types::models::{FileCoverageAnalysis, IsotarpAnalysis, TestCoverageAnalysis};
use std::collections::{HashMap, HashSet};
use std::process::Command;
use rayon::prelude::*;

/// Run all tests at once using tarpaulin and process the results
pub fn run_analysis(
    package_name: &str,
    test_names: &[String],
    output_dir: &std::path::Path,
) -> Result<IsotarpAnalysis, Error> {
    // Create output directory
    std::fs::create_dir_all(output_dir)?;

    // Clean and build once at the beginning
    println!("Cleaning and building package...");
    let status = Command::new("cargo")
        .args(["clean", "-p", package_name])
        .status()?;

    if !status.success() {
        return Err(Error::CommandFailed("cargo clean".to_string()));
    }

    // Build the package
    let status = Command::new("cargo")
        .args(["build", "--tests", "-p", package_name])
        .status()?;

    if !status.success() {
        return Err(Error::CommandFailed("cargo build --tests".to_string()));
    }

    let test_coverage: HashMap<String, HashMap<String, HashSet<u64>>> = test_names
        .par_iter() // Use rayon to run tests in parallel
        .map(|test_name| {
            let covered_lines = run_isolated_test_coverage(package_name, test_name, output_dir, true)?;
            Ok((test_name.clone(), covered_lines))
        })
        .collect::<Result<HashMap<_, _>, Error>>()?; // Collect results into a HashMap

    // Analyze the collected data
    let analysis = analyze_test_coverage(&test_coverage);

    // Return the final analysis
    Ok(IsotarpAnalysis {
        package: package_name.to_string(),
        tests: analysis,
    })
}

/// Analyze coverage to find unique lines covered by each test
pub fn analyze_test_coverage(
    results: &HashMap<String, HashMap<String, HashSet<u64>>>,
) -> HashMap<String, TestCoverageAnalysis> {
    let mut analysis = HashMap::new();

    // Get all files and lines
    let mut all_files = HashSet::new();
    for file_lines in results.values() {
        all_files.extend(file_lines.keys().cloned());
    }

    // For each test
    for (test_name, file_lines) in results {
        let mut analysis_entry = TestCoverageAnalysis {
            total_covered_lines: 0,
            unique_covered_lines: 0,
            files: HashMap::new(),
        };

        // For each file
        for file in &all_files {
            let covered_lines = file_lines.get(file).cloned().unwrap_or_default();
            if covered_lines.is_empty() {
                continue;
            }

            let total_lines = covered_lines.len() as u32;
            analysis_entry.total_covered_lines += total_lines;

            // Find unique lines
            let mut unique_lines = HashSet::new();

            for line in &covered_lines {
                let unique = results.iter().all(|(other_test, other_files)| {
                    if other_test == test_name {
                        return true; // Skip comparing with self
                    }

                    if let Some(other_lines) = other_files.get(file) {
                        !other_lines.contains(line)
                    } else {
                        true // Line is unique if other test doesn't cover this file
                    }
                });

                if unique {
                    unique_lines.insert(*line);
                }
            }

            let unique_count = unique_lines.len() as u32;
            analysis_entry.unique_covered_lines += unique_count;

            if total_lines > 0 {
                analysis_entry.files.insert(
                    file.clone(),
                    FileCoverageAnalysis {
                        total_covered_lines: total_lines,
                        unique_covered_lines: unique_count,
                        unique_lines: unique_lines.into_iter().collect(),
                    },
                );
            }
        }

        analysis.insert(test_name.clone(), analysis_entry);
    }

    analysis
}
