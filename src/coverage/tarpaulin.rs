use crate::types::errors::Error;
use crate::types::models::{LineStat, TarpaulinReport};
use crate::utils::paths::{test_output_dir, test_report_path};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

/// Run a specific test using tarpaulin and return the covered lines
/// This function assumes the package has already been built
pub fn run_isolated_test_coverage(
    package_name: &str,
    test_name: &str,
    output_dir: &Path,
    target_dir: &Path,
    skip_clean: bool,
) -> Result<HashMap<String, HashSet<u64>>, Error> {
    // Create output directory for this test
    let test_output_dir = test_output_dir(output_dir, test_name);
    std::fs::create_dir_all(&test_output_dir).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!(
                "Failed to create output directory '{}': {}",
                test_output_dir.display(),
                e
            ),
        ))
    })?;

    // Build command arguments
    let args = vec![
        "tarpaulin",
        "-p",
        package_name,
        "--no-fail-fast",
        if skip_clean {
            "--skip-clean"
        } else {
            "--force-clean"
        },
        "--target-dir",
        target_dir.to_str().unwrap_or_default(),
        "-o",
        "Json",
        "--output-dir",
        test_output_dir.to_str().unwrap_or_default(),
        "--",
        test_name,
    ];

    // Run tarpaulin for this specific test
    println!("Running coverage for test: {}", test_name);
    let output = Command::new("cargo")
        .args(&args)
        .output()
        .map_err(|e| Error::CommandFailed(format!("Failed to execute cargo command: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::TarpaulinFailed(format!(
            "Tarpaulin failed for test '{}' with status: {}\nStderr: {}",
            test_name, output.status, stderr
        )));
    }

    // Read and parse the report
    let report_path = test_report_path(output_dir, test_name);
    let report_content = std::fs::read_to_string(&report_path).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!(
                "Failed to read tarpaulin report '{}': {}",
                report_path.display(),
                e
            ),
        ))
    })?;

    let report: TarpaulinReport = serde_json::from_str(&report_content).map_err(Error::Json)?;

    // Extract the covered lines
    let covered_lines = extract_covered_lines(&report, package_name);

    Ok(covered_lines)
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

/// Get all test names from the package
pub fn list_tests(package_name: &str) -> Result<Vec<String>, Error> {
    let output = Command::new("cargo")
        .args(["test", "-p", package_name, "--", "--quiet", "--list"])
        .output()
        .map_err(|e| {
            Error::CommandFailed(format!("Failed to execute 'cargo test --list': {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::CommandFailed(format!(
            "cargo test --list failed: {}\nStderr: {}",
            output.status, stderr
        )));
    }

    let output_str = String::from_utf8(output.stdout).map_err(Error::Utf8)?;

    // Parse the output to extract test names
    let tests: Vec<String> = output_str
        .lines()
        .filter(|line| line.contains(": test"))
        .map(|line| line.trim().trim_end_matches(": test").to_string())
        .collect();

    Ok(tests)
}
