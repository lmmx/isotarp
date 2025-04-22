use std::path::{Path, PathBuf};

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
    test_output_dir(output_dir, test_name).join("tarpaulin-target")
}

/// Constructs the tarpaulin report file path for a specific test
pub fn test_report_path(output_dir: &Path, test_name: &str) -> PathBuf {
    test_output_dir(output_dir, test_name).join("tarpaulin-report.json")
}
