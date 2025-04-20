// src/types.rs
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Tarpaulin failed: {0}")]
    TarpaulinFailed(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),
}

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
#[serde(tag = "type", content = "value")]
pub enum LineStat {
    #[serde(rename = "Line")]
    Line(u64),
    // Add other stats as needed
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
