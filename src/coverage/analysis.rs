use crate::coverage::tarpaulin::run_isolated_test_coverage;
use crate::types::errors::Error;
use crate::types::models::{FileCoverageAnalysis, IsotarpAnalysis, TestCoverageAnalysis};
use crate::utils::target_symlink::prepare_target_dirs;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Clean up target directories to save disk space
fn cleanup_target_dirs(output_dir: &Path, test_names: &[String]) {
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

    // Get the master target directory path
    let master_target_dir = Path::new("target");

    // Prepare copied target directories for each test
    println!("Preparing target directories for parallel execution...");
    let test_target_dirs =
        prepare_target_dirs(master_target_dir, test_names, output_dir).map_err(|e| Error::Io(e))?;

    let mut test_coverage = HashMap::new();

    // Define a closure for cleanup that we can call in multiple places
    let cleanup = || cleanup_target_dirs(output_dir, test_names);

    // Run tarpaulin for each test in parallel
    let results: Result<Vec<(String, HashMap<String, HashSet<u64>>)>, Error> = test_names
        .par_iter()
        .enumerate()
        .map(|(i, test_name)| {
            let covered_lines = run_isolated_test_coverage(
                package_name,
                test_name,
                output_dir,
                &test_target_dirs[i],
                true, // skip_clean is always true for parallel runs
            )?;
            Ok((test_name.clone(), covered_lines))
        })
        .collect();

    // Handle the results - either populate test_coverage or return error
    match results {
        Ok(results_vec) => {
            // Convert Vec to HashMap
            for (test_name, covered_lines) in results_vec {
                test_coverage.insert(test_name, covered_lines);
            }
        }
        Err(e) => {
            // Clean up if there was an error during test coverage collection
            cleanup();
            return Err(e);
        }
    }

    // Analyze the collected data
    let analysis = analyze_test_coverage(&test_coverage);

    // Clean up target directories
    cleanup();

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
