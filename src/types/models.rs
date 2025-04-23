use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type TestCoverageResult = (String, HashMap<String, HashSet<u64>>);

/// Representation of Tarpaulin's JSON output
#[derive(Debug, Deserialize, Serialize)]
pub struct TarpaulinReport {
    pub files: Vec<SourceFile>,
    pub coverage: f64,
    pub covered: usize,
    pub coverable: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceFile {
    pub path: Vec<String>,
    pub content: String,
    pub traces: Vec<Trace>,
    pub covered: usize,
    pub coverable: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Trace {
    pub line: u64,
    pub stats: LineStat,
    pub address: HashSet<u64>,
    pub length: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum LineStat {
    Line(u64),
}

/// Analysis of a single test's coverage
#[derive(Debug, Serialize)]
pub struct TestCoverageAnalysis {
    pub total_covered_lines: u32,
    pub unique_covered_lines: u32,
    pub files: HashMap<String, FileCoverageAnalysis>,
}

/// Analysis of a file's coverage by a test
#[derive(Debug, Serialize)]
pub struct FileCoverageAnalysis {
    pub total_covered_lines: u32,
    pub unique_covered_lines: u32,
    pub unique_lines: Vec<u64>,
}

/// Complete analysis output
#[derive(Debug, Serialize)]
pub struct IsotarpAnalysis {
    pub package: String,
    pub tests: HashMap<String, TestCoverageAnalysis>,
}

/// Mode for managing target directories during test execution
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum TargetMode {
    /// Create a separate target directory for each test (more disk space, parallel execution)
    #[default]
    Per,
    /// Reuse a single target directory for all tests (less disk space, sequential execution)
    One,
}

impl std::fmt::Display for TargetMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetMode::Per => write!(f, "per"),
            TargetMode::One => write!(f, "one"),
        }
    }
}
