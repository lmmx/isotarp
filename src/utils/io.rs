use crate::types::errors::Error;
use crate::types::models::IsotarpAnalysis;
use std::path::Path;

/// Save the analysis to a JSON file
pub fn save_analysis(analysis: &IsotarpAnalysis, output_path: &Path) -> Result<(), Error> {
    let json = serde_json::to_string_pretty(analysis)?;
    std::fs::write(output_path, json)?;
    Ok(())
}
