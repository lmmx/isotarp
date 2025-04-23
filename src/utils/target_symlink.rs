use crate::utils::paths::{artifacts_dir, test_output_dir, test_target_dir};
use std::fs;
use std::io;
#[cfg(unix)]
use std::os::unix::fs as unix_fs; // For Unix-like systems (Linux, macOS)
#[cfg(windows)]
use std::os::windows::fs as windows_fs; // For Windows systems
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Wraps an IO error with context about which file was being processed
fn with_path_context<P: AsRef<Path>>(err: io::Error, path: P) -> io::Error {
    io::Error::new(
        err.kind(),
        format!(
            "Error processing path '{}': {}",
            path.as_ref().display(),
            err
        ),
    )
}

/// Create a symlink that's compatible with the current platform
#[cfg(unix)]
fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> io::Result<()> {
    {
        let o = original.as_ref();
        let l = link.as_ref();
        println!("Symlinking {} → {}", o.display(), l.display());
    }
    unix_fs::symlink(original, link)
}

/// Create a symlink that's compatible with the current platform
/// On Windows, we need to specify if we're linking to a file or directory
#[cfg(windows)]
fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> io::Result<()> {
    {
        let o = original.as_ref();
        let l = link.as_ref();
        println!("Symlinking {} → {}", o.display(), l.display());
    }
    if original.as_ref().is_dir() {
        windows_fs::symlink_dir(original, link)
    } else {
        windows_fs::symlink_file(original, link)
    }
}

/// Prepares target directories for parallel tarpaulin runs by creating
/// directory structure and symlinking build artifacts from a master target directory.
pub fn prepare_target_dirs(
    master_target_dir: &Path,
    test_names: &[String],
    output_dir: &Path,
) -> io::Result<Vec<PathBuf>> {
    let mut test_target_dirs = Vec::new();

    // Ensure the central artifacts directory exists
    fs::create_dir_all(artifacts_dir(output_dir))?;

    for test_name in test_names {
        println!("Preparing target directory for test: {}", test_name);

        // Create test output directory (for reports)
        let test_output_dir = test_output_dir(output_dir, test_name);
        fs::create_dir_all(&test_output_dir).map_err(|e| with_path_context(e, &test_output_dir))?;

        // Create test target directory in the central artifacts location
        let test_target_dir = test_target_dir(output_dir, test_name);
        fs::create_dir_all(&test_target_dir).map_err(|e| with_path_context(e, &test_target_dir))?;

        // Create the debug directory
        let debug_dir = test_target_dir.join("debug");
        fs::create_dir_all(&debug_dir)?;

        // Create an empty .cargo-lock file in the debug directory
        let cargo_lock_file = debug_dir.join(".cargo-lock");
        if !cargo_lock_file.exists() {
            fs::write(&cargo_lock_file, "").map_err(|e| with_path_context(e, &cargo_lock_file))?;
        }

        // For directories that need to be writable during compilation,
        // we need to create real directories and potentially copy files
        let write_dirs = vec![
            "debug/.fingerprint",
            "debug/deps",
            "debug/build",
            "debug/incremental",
        ];

        for dir_path in &write_dirs {
            let dest_dir = test_target_dir.join(dir_path);
            fs::create_dir_all(&dest_dir).map_err(|e| with_path_context(e, &dest_dir))?;

            // For executable files in deps, copy them to preserve permissions
            if *dir_path == "debug/deps" {
                let source_dir = master_target_dir.join(dir_path);
                if source_dir.exists() {
                    for entry in WalkDir::new(&source_dir)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.file_type().is_file())
                    {
                        let rel_path = entry
                            .path()
                            .strip_prefix(&source_dir)
                            .expect("Failed to strip prefix");

                        let dest_file = dest_dir.join(rel_path);
                        if !dest_file.exists() {
                            // For executable test files, we want to copy them
                            // This is likely a compiled binary/test, copy it
                            fs::copy(entry.path(), &dest_file).map_err(|e| {
                                with_path_context(
                                    e,
                                    format!(
                                        "Failed to copy from '{}' to '{}'",
                                        entry.path().display(),
                                        dest_file.display()
                                    ),
                                )
                            })?;
                        }
                    }
                }
            }

            // For other directories, just create the directory structure
            // but don't copy or symlink files, let the compiler create them
        }

        // For read-only directories, we can still use symlinks if they exist in the master
        let symlink_dirs = vec!["debug/examples", "debug/build/src"];
        for dir_path in &symlink_dirs {
            let source_dir = master_target_dir.join(dir_path);
            if !source_dir.exists() {
                continue;
            }

            let dest_dir = test_target_dir.join(dir_path);
            fs::create_dir_all(&dest_dir).map_err(|e| with_path_context(e, &dest_dir))?;

            for entry in WalkDir::new(&source_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let rel_path = entry
                    .path()
                    .strip_prefix(&source_dir)
                    .expect("Failed to strip prefix");

                let dest_file = dest_dir.join(rel_path);

                if let Some(parent) = dest_file.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).map_err(|e| with_path_context(e, parent))?;
                    }
                }

                if !dest_file.exists() {
                    create_symlink(entry.path(), &dest_file).map_err(|e| {
                        with_path_context(
                            e,
                            format!(
                                "Failed to symlink from '{}' to '{}'",
                                entry.path().display(),
                                dest_file.display()
                            ),
                        )
                    })?;
                }
            }
        }

        test_target_dirs.push(test_target_dir);
    }

    Ok(test_target_dirs)
}
