use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Prepares target directories for parallel tarpaulin runs by creating
/// directory structure and symlinking build artifacts from a master target directory.
pub fn prepare_target_dirs(
    master_target_dir: &Path,
    test_names: &[String],
    output_dir: &Path,
) -> io::Result<Vec<PathBuf>> {
    let mut test_target_dirs = Vec::new();

    for test_name in test_names {
        let test_output_dir = output_dir.join(test_name.replace("::", "/"));
        fs::create_dir_all(&test_output_dir)?;

        let test_target_dir = test_output_dir.join("tarpaulin-target");
        fs::create_dir_all(&test_target_dir)?;

        // First create all directories to match master structure
        for entry in WalkDir::new(master_target_dir)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_dir() {
                let rel_path = entry
                    .path()
                    .strip_prefix(master_target_dir)
                    .expect("Failed to strip prefix");

                let dest_dir = test_target_dir.join(rel_path);
                fs::create_dir_all(&dest_dir)?;
            }
        }

        // Create .cargo-lock files in debug directories
        let debug_dir = test_target_dir.join("debug");
        if debug_dir.exists() {
            let cargo_lock_file = debug_dir.join(".cargo-lock");
            if !cargo_lock_file.exists() {
                // Create an empty file instead of a symlink
                fs::write(&cargo_lock_file, "")?;
            }
        }

        // Then symlink all files
        for entry in WalkDir::new(master_target_dir)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_file() {
                let rel_path = entry
                    .path()
                    .strip_prefix(master_target_dir)
                    .expect("Failed to strip prefix");
                
                // Skip .cargo-lock files - they should be created as empty files, not symlinks
                if entry.file_name() == ".cargo-lock" {
                    continue;
                }

                let dest_file = test_target_dir.join(rel_path);

                // Check if destination file already exists
                if !dest_file.exists() {
                    // Create parent directory if needed
                    if let Some(parent) = dest_file.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    // Create symlink with proper error handling
                    match symlink(entry.path(), &dest_file) {
                        Ok(_) => (),
                        Err(e) => {
                            // If the error is "File exists", just continue
                            if e.kind() == io::ErrorKind::AlreadyExists {
                                continue;
                            }
                            // Otherwise, propagate the error
                            return Err(e);
                        }
                    }
                }
            }
        }

        test_target_dirs.push(test_target_dir);
    }

    Ok(test_target_dirs)
}