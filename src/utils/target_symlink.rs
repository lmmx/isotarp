use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Wraps an IO error with context about which file was being processed
fn with_path_context<P: AsRef<Path>>(err: io::Error, path: P) -> io::Error {
    io::Error::new(
        err.kind(),
        format!("Error processing path '{}': {}", path.as_ref().display(), err),
    )
}

/// Prepares target directories for parallel tarpaulin runs by creating
/// directory structure and copying build artifacts from a master target directory.
pub fn prepare_target_dirs(
    master_target_dir: &Path,
    test_names: &[String],
    output_dir: &Path,
) -> io::Result<Vec<PathBuf>> {
    let mut test_target_dirs = Vec::new();

    for test_name in test_names {
        println!("Preparing target directory for test: {}", test_name);
        
        let test_output_dir = output_dir.join(test_name.replace("::", "/"));
        fs::create_dir_all(&test_output_dir)
            .map_err(|e| with_path_context(e, &test_output_dir))?;

        let test_target_dir = test_output_dir.join("tarpaulin-target");
        fs::create_dir_all(&test_target_dir)
            .map_err(|e| with_path_context(e, &test_target_dir))?;

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
                fs::create_dir_all(&dest_dir)
                    .map_err(|e| with_path_context(e, &dest_dir))?;
            }
        }

        // Now copy only essential files instead of all files
        // This reduces the amount of IO and makes the process more reliable
        let essential_dirs = [
            "debug/deps",
            "debug/.fingerprint",
            "debug/build",
        ];
        
        for dir in essential_dirs.iter() {
            let source_dir = master_target_dir.join(dir);
            if !source_dir.exists() {
                continue;
            }
            
            for entry in WalkDir::new(&source_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let rel_path = entry
                    .path()
                    .strip_prefix(master_target_dir)
                    .expect("Failed to strip prefix");
                
                let dest_file = test_target_dir.join(rel_path);
                
                // Create parent directory if needed
                if let Some(parent) = dest_file.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)
                            .map_err(|e| with_path_context(e, parent))?;
                    }
                }
                
                // Copy the file instead of creating a symlink
                if !dest_file.exists() {
                    fs::copy(entry.path(), &dest_file)
                        .map_err(|e| with_path_context(e, format!(
                            "Failed to copy from '{}' to '{}'",
                            entry.path().display(),
                            dest_file.display()
                        )))?;
                }
            }
        }
        
        // Create empty .cargo-lock files in debug directories
        let debug_dir = test_target_dir.join("debug");
        if debug_dir.exists() {
            let cargo_lock_file = debug_dir.join(".cargo-lock");
            if !cargo_lock_file.exists() {
                fs::write(&cargo_lock_file, "")
                    .map_err(|e| with_path_context(e, &cargo_lock_file))?;
            }
        }

        test_target_dirs.push(test_target_dir);
    }

    Ok(test_target_dirs)
}