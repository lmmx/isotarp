#![allow(clippy::too_many_arguments)]
// tests/commands.rs
use isotarp::cli::execute_analyze_command;
use isotarp::types::models::TargetMode;
use rstest::*;
use std::{env, fs, path::Path, path::PathBuf};
use tempfile::TempDir;

#[fixture]
#[once]
fn temp_dir() -> TempDir {
    tempfile::tempdir().unwrap()
}

#[fixture]
fn output_dir(temp_dir: &TempDir) -> PathBuf {
    let output_dir = temp_dir.path().to_path_buf();
    fs::create_dir_all(&output_dir).unwrap();
    output_dir
}

#[fixture]
fn report_path(temp_dir: &TempDir) -> PathBuf {
    temp_dir.path().join("analysis-report.json")
}

#[fixture]
fn demo_path() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    Path::new(&manifest_dir).join("tests/fixtures/demolib")
}

/// Store current directory to restore it later in a test
#[fixture]
fn original_dir() -> PathBuf {
    env::current_dir().unwrap()
}

#[rstest]
#[case::nonexistent_pattern(
    Some(vec!["definitely_nonexistent_pattern_xyz123".to_string()]),
    false,
    Some("No matching tests to analyze"),
    "Pattern that doesn't match any test",
    None
)]
#[case::error_handling(
    Some(vec!["tests::test_foo".to_string()]),
    false,
    None,
    "Error handling test with invalid output directory",
    Some(true)
)]
#[case::zero_coverage(
    Some(vec!["tests::test_not_bar".to_string()]),
    true,
    None,
    "Test with zero coverage",
    None
)]
#[case::mixed_tests(
    Some(vec![
        "tests::test_foo".to_string(),
        "tests::test_not_bar".to_string(),
    ]),
    true,
    None,
    "Mixed tests with and without coverage",
    None
)]
fn test_execute_analyze_command(
    #[case] tests: Option<Vec<String>>,
    #[case] expect_ok: bool,
    #[case] error_substring: Option<&str>,
    #[case] description: &str,
    #[case] create_file_instead_of_dir: Option<bool>,
    temp_dir: &TempDir,
    output_dir: PathBuf,
    report_path: PathBuf,
    demo_path: PathBuf,
    original_dir: PathBuf,
) {
    // Important: We need to ensure demo_path exists
    assert!(
        demo_path.exists(),
        "Demo path does not exist: {:?}",
        demo_path
    );

    // Set up output location (file or directory based on test case)
    let output_location = if create_file_instead_of_dir.unwrap_or(false) {
        // Create a file at the output_dir path to cause directory creation to fail
        let file_path = temp_dir.path().join("output");
        fs::write(&file_path, "not a directory").unwrap();
        file_path
    } else {
        // Ensure the output directory exists
        fs::create_dir_all(&output_dir).unwrap();
        output_dir
    };

    // Switch to demo directory for executing the command
    env::set_current_dir(&demo_path).unwrap();

    // Print debug info
    println!("Current dir: {:?}", env::current_dir().unwrap());
    println!("Output dir: {:?}", output_location);
    println!("Report path: {:?}", report_path);

    // Execute the analyze command
    let result = execute_analyze_command(
        "demolib",
        tests.clone(),
        &output_location,
        &report_path,
        TargetMode::default(),
    );

    // Restore the original directory - use current_dir captured right before the test
    env::set_current_dir(original_dir).unwrap();

    // Verify the result
    if expect_ok {
        assert!(
            result.is_ok(),
            "Expected success for '{}', but got error: {:?}",
            description,
            result.err()
        );
    } else {
        assert!(
            result.is_err(),
            "Expected error for '{}', but got success",
            description
        );

        if let Some(substring) = error_substring {
            let err = result.unwrap_err();
            let err_string = err.to_string();
            assert!(
                err_string.contains(substring),
                "For '{}', expected error containing '{}', got: {}",
                description,
                substring,
                err_string
            );
        }
    }
}
