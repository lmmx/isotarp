use isotarp::utils::pipeline::TargetPipeline;
use rstest::*;
use std::fs;
use std::path::{Path, PathBuf};
use temp_testdir::TempDir;

// Common fixtures for our tests
#[fixture]
fn temp_dir() -> TempDir {
    TempDir::default()
}

#[fixture]
fn master_dir(temp_dir: TempDir) -> PathBuf {
    let dir = temp_dir.join("master");
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[fixture]
fn output_dir(temp_dir: TempDir) -> PathBuf {
    let dir = temp_dir.join("output");
    fs::create_dir_all(&dir).unwrap();
    dir
}

// Helper to create test directory structure
fn setup_test_dirs(master_dir: &Path, output_dir: &Path, create_test_binary: bool) {
    fs::create_dir_all(master_dir.join("debug/deps")).unwrap();
    fs::create_dir_all(output_dir).unwrap();

    if create_test_binary {
        fs::write(master_dir.join("debug/deps/test_binary"), "dummy content").unwrap();
    }
}

// Test preparing the next test
#[rstest]
fn test_pipeline_prepare_next(master_dir: PathBuf, output_dir: PathBuf) {
    // Setup test directories with a test binary
    setup_test_dirs(&master_dir, &output_dir, true);

    // Initialize pipeline
    let mut pipeline = TargetPipeline::new(&master_dir, &output_dir).unwrap();

    // Prepare for a test
    let test_name = "test_example";
    let result = pipeline.prepare_next(test_name);
    assert!(result.is_ok(), "Failed to prepare for next test");

    // Let the background thread complete
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Verify next test was set
    let artifacts_dir = output_dir.parent().unwrap().join(".isotarp-artifacts");
    let staging_dir = artifacts_dir.join("staging_target");

    assert!(staging_dir.exists(), "Staging directory should exist");
    assert!(
        staging_dir.join("debug/deps").exists(),
        "Deps directory should be created in staging"
    );
}

// Test getting the ready directory
#[rstest]
fn test_pipeline_get_ready_dir(master_dir: PathBuf, output_dir: PathBuf) {
    // Setup test directories
    setup_test_dirs(&master_dir, &output_dir, true);

    // Initialize pipeline
    let mut pipeline = TargetPipeline::new(&master_dir, &output_dir).unwrap();

    // Prepare for a test
    pipeline.prepare_next("test_example").unwrap();

    // Let the background thread complete
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Get the ready directory
    let target_dir = pipeline.get_ready_target_dir().unwrap();

    // Verify the directory is correct
    let artifacts_dir = output_dir.parent().unwrap().join(".isotarp-artifacts");
    let shared_dir = artifacts_dir.join("shared_target");

    assert_eq!(
        target_dir, shared_dir,
        "Target directory should be the shared directory"
    );
    assert!(target_dir.exists(), "Target directory should exist");

    // Verify staging dir was reset
    let staging_dir = artifacts_dir.join("staging_target");
    assert!(staging_dir.exists(), "Staging directory should still exist");
}

// Test cleanup functionality
#[rstest]
fn test_pipeline_cleanup(master_dir: PathBuf, output_dir: PathBuf) {
    // Initialize pipeline
    let mut pipeline = TargetPipeline::new(&master_dir, &output_dir).unwrap();

    // Get the directory paths
    let artifacts_dir = output_dir.parent().unwrap().join(".isotarp-artifacts");
    let shared_dir = artifacts_dir.join("shared_target");
    let staging_dir = artifacts_dir.join("staging_target");

    // Verify directories exist
    assert!(shared_dir.exists(), "Shared directory should exist");
    assert!(staging_dir.exists(), "Staging directory should exist");

    // Call cleanup
    let result = pipeline.cleanup();
    assert!(result.is_ok(), "Cleanup should succeed");

    // Verify directories were removed
    assert!(!shared_dir.exists(), "Shared directory should be removed");
    assert!(!staging_dir.exists(), "Staging directory should be removed");
}

// Test full workflow
#[rstest]
fn test_pipeline_full_workflow(master_dir: PathBuf, output_dir: PathBuf) {
    // Setup test directories with test binaries
    setup_test_dirs(&master_dir, &output_dir, false);
    fs::write(master_dir.join("debug/deps/test_binary1"), "test1").unwrap();
    fs::write(master_dir.join("debug/deps/test_binary2"), "test2").unwrap();

    // Initialize pipeline
    let mut pipeline = TargetPipeline::new(&master_dir, &output_dir).unwrap();

    // Run through a sequence of tests
    let test_names = ["test1", "test2", "test3"];

    // Prepare first test
    pipeline.prepare_next(test_names[0]).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));

    for i in 0..test_names.len() {
        // Get the current test directory
        let target_dir = pipeline.get_ready_target_dir().unwrap();
        assert!(
            target_dir.exists(),
            "Target directory should exist for test {}",
            i
        );

        // Prepare next test if there is one
        if i + 1 < test_names.len() {
            pipeline.prepare_next(test_names[i + 1]).unwrap();
            // Let preparation start
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    // Clean up
    let result = pipeline.cleanup();
    assert!(result.is_ok(), "Final cleanup should succeed");
}

// // Test initialization
// #[rstest]
// fn test_pipeline_initialization(master_dir: PathBuf, output_dir: PathBuf) {
//     // Initialize the pipeline
//     let pipeline = TargetPipeline::new(&master_dir, &output_dir);
//     assert!(pipeline.is_ok(), "Failed to initialize pipeline");
//
//     pipeline.unwrap();
//
//     // Check that directories were created
//     let artifacts_dir = output_dir.parent().unwrap().join(".isotarp-artifacts");
//     let shared_dir = artifacts_dir.join("shared_target");
//     let staging_dir = artifacts_dir.join("staging_target");
//
//     assert!(shared_dir.exists(), "Shared target directory was not created");
//     assert!(staging_dir.exists(), "Staging target directory was not created");
//
//     // Verify minimal target structure was created
//     assert!(shared_dir.join("debug").exists(), "Debug directory was not created");
//     assert!(shared_dir.join("debug/.fingerprint").exists(), "Fingerprint directory was not created");
//     assert!(shared_dir.join("debug/deps").exists(), "Deps directory was not created");
//     assert!(shared_dir.join("debug/build").exists(), "Build directory was not created");
//     assert!(shared_dir.join("debug/incremental").exists(), "Incremental directory was not created");
//     assert!(shared_dir.join("debug/.cargo-lock").exists(), "Cargo lock file was not created");
// }
//
// // Test error handling
// #[rstest]
// fn test_pipeline_error_handling(temp_dir: TempDir) {
//     let master_dir = temp_dir.join("master");
//     let output_dir = temp_dir.join("output");
//
//     // Set up directories but make master a file instead of directory to cause errors
//     fs::create_dir_all(&output_dir).unwrap();
//     fs::write(&master_dir, "not a directory").unwrap();
//
//     // Initialize pipeline - should fail
//     let result = TargetPipeline::new(&master_dir, &output_dir);
//     assert!(result.is_err(), "Pipeline should fail with invalid master directory");
//
//     // Now test with a valid initialization but break the staging directory
//     fs::remove_file(&master_dir).unwrap();
//     fs::create_dir_all(&master_dir).unwrap();
//
//     let mut pipeline = TargetPipeline::new(&master_dir, &output_dir).unwrap();
//
//     // Break the staging directory
//     let artifacts_dir = output_dir.parent().unwrap().join(".isotarp-artifacts");
//     let staging_dir = artifacts_dir.join("staging_target");
//     fs::remove_dir_all(&staging_dir).unwrap();
//     fs::write(&staging_dir, "not a directory").unwrap();
//
//     // Trying to prepare next should handle the error gracefully
//     let result = pipeline.prepare_next("test_example");
//     assert!(result.is_err(), "Prepare next should fail with invalid staging directory");
// }
