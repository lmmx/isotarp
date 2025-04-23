use std::path::{Path, PathBuf};

/// Returns the path to the central artifacts directory as a sibling of the output dir
pub fn artifacts_dir(output_dir: &Path) -> PathBuf {
    // Get the parent directory of the output dir and add a hidden artifacts dir
    output_dir
        .parent()
        .map(|parent| parent.join(".isotarp-artifacts"))
        .unwrap_or_else(|| PathBuf::from(".isotarp-artifacts"))
}

/// Converts a test name with '::' separators to a path-friendly format
/// Example: "module::submodule::test_name" -> "module/submodule/test_name"
pub fn test_name_to_path_segment(test_name: &str) -> String {
    test_name.replace("::", "/")
}

/// Constructs a directory path for a specific test within the output directory
pub fn test_output_dir(output_dir: &Path, test_name: &str) -> PathBuf {
    output_dir.join(test_name_to_path_segment(test_name))
}

/// Constructs the target directory path for a specific test
pub fn test_target_dir(output_dir: &Path, test_name: &str) -> PathBuf {
    // Use the central artifacts directory instead of per-test output directories
    artifacts_dir(output_dir)
        .join(test_name_to_path_segment(test_name))
        .join("tarpaulin-target")
}

/// Constructs the tarpaulin report file path for a specific test
pub fn test_report_path(output_dir: &Path, test_name: &str) -> PathBuf {
    test_output_dir(output_dir, test_name).join("tarpaulin-report.json")
}
