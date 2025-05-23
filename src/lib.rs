// Public API exports
pub mod cli;
pub mod coverage;
pub mod resolve;
pub mod types;
pub mod utils;

// Re-export commonly used items for convenience
pub use coverage::analysis::{analyze_test_coverage, run_analysis};
pub use coverage::tarpaulin::{extract_covered_lines, run_isolated_test_coverage};
pub use types::errors::Error;
pub use types::models::*;
pub use utils::io::save_analysis;
pub use utils::paths::{
    artifacts_dir, test_name_to_path_segment, test_output_dir, test_report_path, test_target_dir,
};
