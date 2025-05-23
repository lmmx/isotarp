use crate::coverage::tarpaulin::run_isolated_test_coverage;
use crate::types::errors::Error;
use crate::types::models::{
    FileCoverageAnalysis, IsotarpAnalysis, TargetMode, TestCoverageAnalysis, TestCoverageResult,
};
use crate::utils::cleanup::{cleanup_single_test_dir, cleanup_target_dirs};
use crate::utils::pipeline::TargetPipeline;
use crate::utils::target_symlink::prepare_target_dirs;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

/// Run all tests at once using tarpaulin and process the results
pub fn run_analysis(
    package_name: &str,
    test_names: &[String],
    output_dir: &std::path::Path,
    target_mode: TargetMode,
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

    // Get the master target directory
    let master_target_dir = Path::new("target");

    // Track progress
    let total_tests = test_names.len();

    // Collect the results
    let collected_results: Vec<(String, HashMap<String, HashSet<u64>>)>;

    match target_mode {
        TargetMode::Per => {
            // Use the original parallel approach
            // Prepare target directories (only as needed to reduce memory usage)
            println!("Target mode: Per - Preparing individual target directories for execution...");
            let test_target_dirs = prepare_target_dirs(master_target_dir, test_names, output_dir)?;

            // Configure thread pool with reasonable concurrency
            let num_cpus = num_cpus::get();
            let thread_count = std::cmp::min(num_cpus, 8); // Limit to 8 or CPU count, whichever is smaller
            let pool = ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build()
                .map_err(|e| {
                    Error::CommandFailed(format!("Failed to create thread pool: {}", e))
                })?;

            // Define a closure for cleanup to use in multiple places
            let cleanup_fn = |test_name: &str| {
                if let Err(e) = cleanup_single_test_dir(output_dir, test_name) {
                    eprintln!(
                        "Warning: Failed to clean up after test '{}': {}",
                        test_name, e
                    );
                }
            };

            // Process tests in parallel with controlled concurrency, collecting results
            println!("Running tests in parallel with {} threads", thread_count);
            println!("Processing {} tests in sorted order", total_tests);

            // Use a scoped threadpool and collect results
            // Using par_bridge to maintain ordering
            let results: Result<Vec<TestCoverageResult>, Error> = pool.install(|| {
                test_names
                    .iter()
                    .enumerate()
                    .par_bridge()
                    .map(|(idx, test_name)| {
                        // Display progress with the correct sequential numbering
                        println!(
                            "[{}/{}] Running coverage for test: {}",
                            idx + 1,
                            total_tests,
                            test_name
                        );

                        // Get the target directory for this test
                        let target_dir = &test_target_dirs[idx];

                        println!("Running coverage for test: {}", test_name);
                        let result = run_isolated_test_coverage(
                            package_name,
                            test_name,
                            output_dir,
                            target_dir,
                            true,
                        );

                        // Immediate cleanup regardless of success or failure
                        cleanup_fn(test_name);

                        // Return the result paired with the test name
                        match result {
                            Ok(covered_lines) => Ok((test_name.clone(), covered_lines)),
                            Err(e) => {
                                eprintln!("Error running test {}: {}", test_name, e);
                                Err(e)
                            }
                        }
                    })
                    .collect()
            });

            // Handle errors from the parallel execution
            collected_results = match results {
                Ok(results_vec) => results_vec,
                Err(e) => {
                    println!("Error during test execution: {}", e);
                    // Final cleanup of any remaining directories
                    cleanup_target_dirs(output_dir, test_names);
                    return Err(e);
                }
            };

            // Final cleanup just to be sure
            cleanup_target_dirs(output_dir, test_names);
        }
        TargetMode::One => {
            // Use the sequential pipelined approach
            println!(
                "Target mode: One - Using a single reused target directory (sequential execution)"
            );

            // Create a pipeline manager for target directories
            let mut pipeline = TargetPipeline::new(master_target_dir, output_dir)?;

            // Process tests sequentially with pipelined directory preparation
            println!("Running tests sequentially with pipeline preparation");
            println!("Processing {} tests", total_tests);

            let mut results_vec = Vec::with_capacity(test_names.len());

            // If we have at least one test, start preparing for it
            if !test_names.is_empty() {
                pipeline.prepare_next(&test_names[0])?;
            }

            // Process each test
            for (idx, test_name) in test_names.iter().enumerate() {
                // Display progress
                println!(
                    "[{}/{}] Running coverage for test: {}",
                    idx + 1,
                    total_tests,
                    test_name
                );

                // Get the prepared target directory
                let target_dir = pipeline.get_ready_target_dir()?;

                // If there's a next test, start preparing its directory in the background
                if idx + 1 < test_names.len() {
                    pipeline.prepare_next(&test_names[idx + 1])?;
                }

                // Run test coverage
                println!("Running coverage for test: {}", test_name);
                match run_isolated_test_coverage(
                    package_name,
                    test_name,
                    output_dir,
                    &target_dir,
                    true,
                ) {
                    Ok(covered_lines) => {
                        results_vec.push((test_name.clone(), covered_lines));
                    }
                    Err(e) => {
                        eprintln!("Error running test {}: {}", test_name, e);
                        pipeline.cleanup()?;
                        return Err(e);
                    }
                }

                // Clean up the test output directory (not the target directory)
                if let Err(e) = cleanup_single_test_dir(output_dir, test_name) {
                    eprintln!(
                        "Warning: Failed to clean up after test '{}': {}",
                        test_name, e
                    );
                }
            }

            // Final cleanup
            pipeline.cleanup()?;
            collected_results = results_vec;
        }
    }

    // Convert the collected results into a HashMap
    let mut test_coverage = HashMap::new();
    for (test_name, coverage) in collected_results {
        test_coverage.insert(test_name, coverage);
    }

    // Generate analysis from the collected coverage data
    let analysis = analyze_test_coverage(&test_coverage);

    Ok(IsotarpAnalysis {
        package: package_name.to_string(),
        tests: analysis,
    })
}

/// Analyze coverage to find unique lines covered by each test
pub fn analyze_test_coverage(
    results: &HashMap<String, HashMap<String, HashSet<u64>>>,
) -> HashMap<String, TestCoverageAnalysis> {
    // The rest of this function remains unchanged...
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
                        true
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
