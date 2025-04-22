use crate::utils::paths::{test_target_dir, artifacts_dir};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Determines if a directory is empty or only contains empty directories
fn is_effectively_empty(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    
    // If we encounter any file, the directory is not empty
    // If we encounter a non-empty directory, the directory is not empty
    for entry in WalkDir::new(path).min_depth(1) {
        match entry {
            Ok(entry) => {
                // If we found a file, the directory is not empty
                if !entry.file_type().is_dir() {
                    return false;
                }
                
                // If we found a non-empty directory, the directory is not empty
                // We don't need to check this since we're doing a depth-first walk
                // and will encounter files before their parent directories
            }
            Err(_) => {
                // Error walking directory - conservatively say it's not empty
                return false;
            }
        }
    }
    
    // If we got here, there are no files, only possibly empty directories
    true
}

/// Recursively remove empty directories
fn remove_empty_directories(path: &Path) -> bool {
    if !path.is_dir() || !is_effectively_empty(path) {
        return false;
    }
    
    // First remove all empty subdirectories
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                remove_empty_directories(&path);
            }
        }
    }
    
    // Now try to remove this directory
    match fs::remove_dir(path) {
        Ok(_) => {
            println!("Removed empty directory: {}", path.display());
            true
        }
        Err(e) => {
            println!(
                "Warning: Failed to clean up empty directory '{}': {}",
                path.display(),
                e
            );
            false
        }
    }
}

/// Clean up target directories to save disk space
pub fn cleanup_target_dirs(output_dir: &Path, test_names: &[String]) {
    println!("Cleaning up temporary target directories...");

    let artifacts_directory = artifacts_dir(output_dir);

    for test_name in test_names {
        // Remove the target directory
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
    
    // Clean up any empty directories in the artifacts directory
    if artifacts_directory.exists() {
        remove_empty_directories(&artifacts_directory);
    }
}