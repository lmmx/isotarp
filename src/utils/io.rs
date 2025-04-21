use crate::types::errors::Error;
use crate::types::models::IsotarpAnalysis;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::Path;

/// Save the analysis to a JSON file with deterministic ordering
pub fn save_analysis(analysis: &IsotarpAnalysis, output_path: &Path) -> Result<(), Error> {
    // Convert to a Value first
    let mut value = serde_json::to_value(analysis)?;

    // Now sort the keys in all maps to ensure deterministic ordering
    if let Value::Object(map) = &mut value {
        let sorted_value = sort_json_object(map);

        // Convert back to a sorted Value
        let json = serde_json::to_string_pretty(&sorted_value)?;
        std::fs::write(output_path, json)?;
        return Ok(());
    }

    // Fallback to normal serialization if conversion to object failed
    let json = serde_json::to_string_pretty(analysis)?;
    std::fs::write(output_path, json)?;
    Ok(())
}

/// Recursively sort all objects in a JSON value by key
fn sort_json_object(map: &Map<String, Value>) -> Value {
    // Create a sorted map (BTreeMap maintains key order)
    let mut sorted_map = BTreeMap::new();

    // Add all entries, sorting nested objects recursively
    for (key, value) in map {
        let sorted_value = match value {
            Value::Object(obj) => sort_json_object(obj),
            Value::Array(arr) => {
                // Sort arrays that contain primitive values if possible
                let mut sorted_arr = Vec::new();
                for item in arr {
                    match item {
                        Value::Object(obj) => sorted_arr.push(sort_json_object(obj)),
                        Value::Array(_) => sorted_arr.push(item.clone()), // Not sorting nested arrays
                        // Sort arrays of numbers
                        Value::Number(_) => {
                            let mut numbers: Vec<i64> =
                                arr.iter().filter_map(|v| v.as_i64()).collect();
                            numbers.sort();
                            return Value::Array(numbers.into_iter().map(Value::from).collect());
                        }
                        // For other primitive types, just clone
                        _ => sorted_arr.push(item.clone()),
                    }
                }
                Value::Array(sorted_arr)
            }
            _ => value.clone(),
        };
        sorted_map.insert(key.clone(), sorted_value);
    }

    // Convert the BTreeMap to a JSON Value
    serde_json::to_value(sorted_map).unwrap_or(Value::Object(map.clone()))
}
