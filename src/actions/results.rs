// src/actions/results.rs
//
// Handler Result Schema Validation
// Validates JSON result format returned by action handlers according to SPEC.md §1.4
//
// Responsibilities:
// - Validate success result format: { "success": true, "data": { ... } }
// - Validate failure result format: { "success": false, "error": "message" }
// - Return boolean indicating valid format
//
// Dependencies:
// - serde_json for JSON parsing
//
// Created: 2026-04-20

use serde_json::Value;

/// Validate that a handler result JSON string conforms to the standard schema
/// defined in SPEC.md §1.4
///
/// Valid formats:
/// - Success: { "success": true, "data": { ... } }
/// - Failure: { "success": false, "error": "human-readable description" }
///
/// Returns true if the format is valid, false otherwise
pub fn validate_handler_result(result_json: &str) -> bool {
    let value = match serde_json::from_str::<Value>(result_json) {
        Ok(v) => v,
        Err(_) => return false,
    };

    // Must be an object
    let obj = match value.as_object() {
        Some(o) => o,
        None => return false,
    };

    // Must have success boolean field
    let success = match obj.get("success") {
        Some(Value::Bool(b)) => *b,
        _ => return false,
    };

    if success {
        // Success case: must have 'data' field (any type allowed)
        obj.contains_key("data")
    } else {
        // Failure case: must have 'error' field which is a string
        matches!(obj.get("error"), Some(Value::String(_)))
    }
}
