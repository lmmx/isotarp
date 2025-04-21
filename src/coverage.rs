pub mod analysis;
pub mod tarpaulin;

// Re-export main functions
pub use analysis::{analyze_test_coverage, run_analysis};
pub use tarpaulin::{extract_covered_lines, list_tests, run_isolated_test_coverage};
