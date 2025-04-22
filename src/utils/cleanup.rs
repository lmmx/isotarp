use crate::utils::paths::test_target_dir;
use std::fs;
use std::path::Path;

/// Clean up target directories to save disk space
pub fn cleanup_target_dirs(output_dir: &Path, test_names: &[String]) {
    println!("Cleaning up temporary target directories...");

    for test_name in test_names {
        let target_dir = test_target_dir(output_dir, test_name);

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
