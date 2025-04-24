// tests/commands.rs
use isotarp::cli::execute_analyze_command;
use isotarp::types::models::TargetMode;
use std::env;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

// Test specifically targeting the pattern resolution code path
#[test]
fn test_pattern_resolution_with_invalid_pattern() {
    // Create temporary directory for outputs
    let temp_dir = tempdir().unwrap();
    let output_dir = temp_dir.path().to_path_buf();
    let report_path = temp_dir.path().join("report.json");

    // Create the output directory
    fs::create_dir_all(&output_dir).unwrap();

    // Use the demo package
    let package = "demolib";

    // IMPORTANT: Find the actual demolib path using environment variables
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let demo_path = Path::new(&manifest_dir).join("tests/fixtures/demolib");

    // Make sure we're within the demo directory when running the command
    let current_dir = env::current_dir().unwrap();
    env::set_current_dir(&demo_path).unwrap();

    // Create a pattern that definitely won't match any test in demolib
    let tests = Some(vec!["definitely_nonexistent_pattern_xyz123".to_string()]);

    // Execute the analyze command
    let result = execute_analyze_command(
        package,
        tests,
        &output_dir,
        &report_path,
        TargetMode::default(),
    );

    // Restore the original directory
    env::set_current_dir(current_dir).unwrap();

    // This should fail with a specific error message about no matching tests
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_string = err.to_string();
    assert!(
        err_string.contains("No matching tests to analyze"),
        "Expected error 'No matching tests to analyze', got: {}",
        err_string
    );
}

// Keep the working test for error handling
#[test]
fn test_execute_analyze_error_handling() {
    // Create temporary directory for outputs
    let temp_dir = tempdir().unwrap();

    // Create a file at the output_dir path, which will cause directory creation to fail
    let output_file = temp_dir.path().join("output");
    fs::write(&output_file, "not a directory").unwrap();

    let report_path = temp_dir.path().join("report.json");

    // Use the demo package
    let package = "demolib";

    // Specify a test that exists but will fail due to output_dir being a file
    let tests = Some(vec!["tests::test_foo".to_string()]);

    // Execute the analyze command
    let result = execute_analyze_command(
        package,
        tests,
        &output_file, // Pass a file instead of a directory
        &report_path,
        TargetMode::default(),
    );

    // This should fail but not panic
    assert!(result.is_err());
}

// Test for the code path where tests have no unique coverage
#[test]
fn test_zero_unique_coverage_reporting() {
    // Create temporary directory for outputs
    let temp_dir = tempdir().unwrap();
    let output_dir = temp_dir.path().to_path_buf();
    let report_path = temp_dir.path().join("analysis-report.json");

    // Create the output directory
    fs::create_dir_all(&output_dir).unwrap();

    // IMPORTANT: Find the actual demolib path using environment variables
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let demo_path = Path::new(&manifest_dir).join("tests/fixtures/demolib");

    // Make sure we're within the demo directory when running the command
    let current_dir = env::current_dir().unwrap();
    env::set_current_dir(&demo_path).unwrap();

    // We need to create specific test scenarios
    // For the demolib package, test_not_bar has zero coverage
    let tests = Some(vec!["tests::test_not_bar".to_string()]);

    // Execute the analyze command
    let result = execute_analyze_command(
        "demolib",
        tests,
        &output_dir,
        &report_path,
        TargetMode::default(),
    );

    // Restore the original directory
    env::set_current_dir(current_dir).unwrap();

    // Since test_not_bar has no coverage at all, this should succeed
    assert!(result.is_ok());
}

// Test for the full analyze path with both types of tests
#[test]
fn test_full_analysis_with_mixed_tests() {
    // Create temporary directory for outputs
    let temp_dir = tempdir().unwrap();
    let output_dir = temp_dir.path().to_path_buf();
    let report_path = temp_dir.path().join("analysis-report.json");

    // Create the output directory
    fs::create_dir_all(&output_dir).unwrap();

    // IMPORTANT: Find the actual demolib path using environment variables
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let demo_path = Path::new(&manifest_dir).join("tests/fixtures/demolib");

    // Make sure we're within the demo directory when running the command
    let current_dir = env::current_dir().unwrap();
    env::set_current_dir(&demo_path).unwrap();

    // Run both tests - test_foo has coverage, test_not_bar has none
    let tests = Some(vec![
        "tests::test_foo".to_string(),
        "tests::test_not_bar".to_string(),
    ]);

    // Execute the analyze command
    let result = execute_analyze_command(
        "demolib",
        tests,
        &output_dir,
        &report_path,
        TargetMode::default(),
    );

    // Restore the original directory
    env::set_current_dir(current_dir).unwrap();

    // This should succeed
    assert!(result.is_ok());
}
