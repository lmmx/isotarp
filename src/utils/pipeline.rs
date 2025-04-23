use crate::types::errors::Error;
use crate::utils::paths::{artifacts_dir, test_target_dir};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

/// Manages a pipeline of target directories for sequential test execution
pub struct TargetPipeline {
    master_target_dir: PathBuf,
    output_dir: PathBuf,
    shared_target_dir: PathBuf,
    staging_dir: PathBuf,
    prepare_thread: Option<thread::JoinHandle<Result<(), Error>>>,
    next_test: Arc<Mutex<Option<String>>>,
    is_running: Arc<Mutex<bool>>,
}

impl TargetPipeline {
    /// Creates a new target pipeline manager
    pub fn new(master_target_dir: &Path, output_dir: &Path) -> Result<Self, Error> {
        // Create the artifacts directory
        let artifacts = artifacts_dir(output_dir);
        fs::create_dir_all(&artifacts)?;

        // Create shared and staging directories
        let shared_target_dir = artifacts.join("shared_target");
        let staging_dir = artifacts.join("staging_target");

        // Create directories if they don't exist
        if !shared_target_dir.exists() {
            fs::create_dir_all(&shared_target_dir)?;
        }

        if !staging_dir.exists() {
            fs::create_dir_all(&staging_dir)?;
        }

        // Prepare the initial shared target directory (minimal setup)
        Self::setup_minimal_target_dir(&shared_target_dir)?;

        Ok(TargetPipeline {
            master_target_dir: master_target_dir.to_path_buf(),
            output_dir: output_dir.to_path_buf(),
            shared_target_dir,
            staging_dir,
            prepare_thread: None,
            next_test: Arc::new(Mutex::new(None)),
            is_running: Arc::new(Mutex::new(true)),
        })
    }

    /// Set up a minimal target directory structure
    fn setup_minimal_target_dir(target_dir: &Path) -> Result<(), Error> {
        // Create the debug directory
        let debug_dir = target_dir.join("debug");
        fs::create_dir_all(&debug_dir)?;

        // Create basic structure
        for dir in &[
            "debug/.fingerprint",
            "debug/deps",
            "debug/build",
            "debug/incremental",
        ] {
            fs::create_dir_all(target_dir.join(dir))?;
        }

        // Create an empty .cargo-lock file
        let cargo_lock_file = debug_dir.join(".cargo-lock");
        if !cargo_lock_file.exists() {
            fs::write(&cargo_lock_file, "")?;
        }

        Ok(())
    }

    /// Start preparing the target directory for the next test
    pub fn prepare_next(&mut self, test_name: &str) -> Result<(), Error> {
        // Stop any existing preparation thread
        self.stop_preparation();

        // Set the next test name
        {
            let mut next = self.next_test.lock().unwrap();
            *next = Some(test_name.to_string());
        }

        // Clone necessary data for the thread
        let master_dir = self.master_target_dir.clone();
        let staging = self.staging_dir.clone();
        let next_test = Arc::clone(&self.next_test);
        let is_running = Arc::clone(&self.is_running);
        let output_dir = self.output_dir.clone();

        // Start a new thread to prepare the next target directory
        let handle = thread::spawn(move || -> Result<(), Error> {
            // Clean the staging directory first
            if staging.exists() {
                fs::remove_dir_all(&staging)?;
            }
            fs::create_dir_all(&staging)?;

            // Set up the minimal directory structure
            Self::setup_minimal_target_dir(&staging)?;

            // Get the test-specific target directory path (for reference)
            let test_name_str = {
                let next = next_test.lock().unwrap();
                match &*next {
                    Some(name) => name.clone(),
                    None => return Ok(()),
                }
            };

            let _test_dir = test_target_dir(&output_dir, &test_name_str);

            // Copy selected files from the master target directory
            // This is the main work of preparing for the next test

            println!(
                "Preparing target directory for test '{}' in the background",
                test_name_str
            );

            // Copy executable files from the master to the staging directory
            let master_deps = master_dir.join("debug/deps");
            let staging_deps = staging.join("debug/deps");

            if master_deps.exists() {
                for entry in fs::read_dir(&master_deps)? {
                    let entry = entry?;
                    let path = entry.path();

                    // Only copy files (not directories)
                    if path.is_file() {
                        // Check if preparation should continue
                        if !*is_running.lock().unwrap() {
                            return Ok(());
                        }

                        let dest = staging_deps.join(path.file_name().unwrap());
                        fs::copy(&path, &dest)?;
                    }
                }
            }

            println!(
                "Background preparation complete for test '{}'",
                test_name_str
            );
            Ok(())
        });

        self.prepare_thread = Some(handle);
        Ok(())
    }

    /// Wait for the preparation to complete and swap in the new directory
    pub fn get_ready_target_dir(&mut self) -> Result<PathBuf, Error> {
        // Wait for preparation to complete if there's a thread running
        if let Some(thread) = self.prepare_thread.take() {
            match thread.join() {
                Ok(result) => {
                    if let Err(e) = result {
                        eprintln!("Error preparing target directory: {}", e);
                        // Fall back to using the shared directory as-is
                    }
                }
                Err(_) => {
                    eprintln!("Background preparation thread panicked");
                    // Fall back to using the shared directory as-is
                }
            }
        }

        // Get the current test name
        let current_test = {
            let mut next = self.next_test.lock().unwrap();
            next.take()
        };

        if let Some(test_name) = current_test {
            println!(
                "Swapping prepared target directory for test '{}'",
                test_name
            );

            // Swap the staging directory with the shared directory
            if self.staging_dir.exists() {
                // Remove the old shared directory
                if self.shared_target_dir.exists() {
                    fs::remove_dir_all(&self.shared_target_dir)?;
                }

                // Rename the staging directory to the shared directory
                fs::rename(&self.staging_dir, &self.shared_target_dir)?;

                // Create a new empty staging directory
                fs::create_dir_all(&self.staging_dir)?;
            }
        }

        Ok(self.shared_target_dir.clone())
    }

    /// Stop background preparation
    fn stop_preparation(&mut self) {
        // Signal the thread to stop
        {
            let mut running = self.is_running.lock().unwrap();
            *running = false;
        }

        // Wait for the thread to finish
        if let Some(thread) = self.prepare_thread.take() {
            let _ = thread.join();
        }

        // Reset the running flag
        {
            let mut running = self.is_running.lock().unwrap();
            *running = true;
        }
    }

    /// Clean up resources
    pub fn cleanup(&mut self) -> Result<(), Error> {
        self.stop_preparation();

        // Clean up directories
        if self.shared_target_dir.exists() {
            fs::remove_dir_all(&self.shared_target_dir)?;
        }

        if self.staging_dir.exists() {
            fs::remove_dir_all(&self.staging_dir)?;
        }

        Ok(())
    }
}

impl Drop for TargetPipeline {
    fn drop(&mut self) {
        self.stop_preparation();
        let _ = self.cleanup();
    }
}
