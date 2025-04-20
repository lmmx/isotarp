// src/lib.rs
use std::collections::{HashMap, HashSet};
use std::fs::{self, create_dir_all};
use std::path::Path;
use std::process::Command;

pub mod types;
pub use types::*;

/// Run a specific test using tarpaulin and return the lines it covers
pub fn run_test_for_coverage(
    package_name: &str,
    test_name: &str,
    output_dir: &Path,
) -> Result<TarpaulinReport, Error> {
    // Create output directory
    let test_output_dir = output_dir.join(test_name.replace("::", "/"));
    create_dir_all(&test_output_dir)?;

    // Run tarpaulin for this specific test
    let status = Command::new("cargo")
        .args([
            "tarpaulin",
            "-p",
            package_name,
            "-o",
            "Json",
            "--output-dir",
            test_output_dir.to_str().unwrap(),
            "--",
            test_name,
        ])
        .status()?;

    if !status.success() {
        return Err(Error::TarpaulinFailed(status.to_string()));
    }

    // Read and parse the report
    let report_path = test_output_dir.join("tarpaulin-report.json");
    let report_content = fs::read_to_string(report_path)?;
    let report: TarpaulinReport = serde_json::from_str(&report_content)?;

    Ok(report)
}

/// Extract covered lines from a tarpaulin report
pub fn extract_covered_lines(
    report: &TarpaulinReport,
    package_name: &str,
) -> HashMap<String, HashSet<u64>> {
    let mut covered_lines = HashMap::new();

    for file in &report.files {
        // Extract path parts to see if this file belongs to the package we're analyzing
        let path_str = file.path.join("/");
        if !path_str.contains(package_name) {
            continue;
        }

        // Get covered lines for this file
        let lines: HashSet<u64> = file
            .traces
            .iter()
            .filter(|trace| {
                let LineStat::Line(hits) = trace.stats;
                hits > 0
            })
            .map(|trace| trace.line)
            .collect();

        if !lines.is_empty() {
            covered_lines.insert(path_str, lines);
        }
    }

    covered_lines
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

/// Run analysis for multiple tests
pub fn run_analysis(
    package_name: &str,
    test_names: &[String],
    output_dir: &Path,
) -> Result<IsotarpAnalysis, Error> {
    let mut test_coverage = HashMap::new();

    // Run each test and collect coverage data
    for test_name in test_names {
        println!("Running analysis for test: {}", test_name);

        let report = run_test_for_coverage(package_name, test_name, output_dir)?;
        let covered_lines = extract_covered_lines(&report, package_name);

        test_coverage.insert(test_name.clone(), covered_lines);
    }

    // Analyze the collected data
    let analysis = analyze_test_coverage(&test_coverage);

    // Return the final analysis
    Ok(IsotarpAnalysis {
        package: package_name.to_string(),
        tests: analysis,
    })
}

/// Get all test names from the package
pub fn list_tests(package_name: &str) -> Result<Vec<String>, Error> {
    let output = Command::new("cargo")
        .args(["test", "-p", package_name, "--", "--quiet", "--list"])
        .output()?;

    if !output.status.success() {
        return Err(Error::CommandFailed("cargo test --list".to_string()));
    }

    let output_str = String::from_utf8(output.stdout)?;

    // Parse the output to extract test names
    let tests: Vec<String> = output_str
        .lines()
        .filter(|line| line.contains(": test"))
        .map(|line| line.trim().trim_end_matches(": test").to_string())
        .collect();

    Ok(tests)
}

/// Save the analysis to a JSON file
pub fn save_analysis(analysis: &IsotarpAnalysis, output_path: &Path) -> Result<(), Error> {
    let json = serde_json::to_string_pretty(analysis)?;
    fs::write(output_path, json)?;
    Ok(())
}
