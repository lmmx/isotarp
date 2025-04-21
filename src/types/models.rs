use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
