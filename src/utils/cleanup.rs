use std::fs;
use std::path::PathBuf;

/// Clean up target directories to save disk space
pub fn cleanup_target_dirs(output_dir: &PathBuf, test_names: &[String]) {
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

