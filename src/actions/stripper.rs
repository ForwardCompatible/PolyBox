// src/actions/stripper.rs
// Action tag stripping utility for removing [ACTION:...] tags from text
//
// Responsibilities:
// - Remove all properly formatted action tags from input text
// - Preserve all surrounding text content exactly
// - Handle multiple tags and edge cases correctly
//
// Dependencies:
// - Standard library regex for pattern matching
//
// Created: 2026-04-20

use regex::Regex;

/// Removes all [ACTION:...] tags from input text
/// 
/// Returns clean text with all action tags removed, preserving all other content exactly.
/// Tags are matched case-insensitively and may contain any characters except closing bracket.
pub fn strip_action_tags(input: &str) -> String {
    // Match [ACTION:...] tags, case insensitive, non-greedy
    // Pattern matches:
    // - Literal [ACTION: (case insensitive)
    // - Any characters except ] (non-greedy)
    // - Closing ]
    let re = Regex::new(r"(?i)\[ACTION:[^\]]+\]").unwrap();
    re.replace_all(input, "").to_string()
}
