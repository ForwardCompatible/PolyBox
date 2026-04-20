// src/actions/parser.rs
// Action Tag Parser
//
// Purpose: Parses [ACTION:TAG_NAME key="value"] tags from input text according to SPEC.md §1.1
// Responsibilities:
// - Extract all valid action tags from arbitrary input strings
// - Parse tag names and key/value parameter pairs
// - Handle escaped quotes inside parameter values correctly
// - Gracefully ignore malformed or invalid tags
//
// Dependencies:
// - std::collections::HashMap for parameter storage
//
// Created: 2026-04-20

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedAction {
    pub tag_name: String,
    pub parameters: HashMap<String, String>,
}

pub fn parse_action_tags(input: &str) -> Vec<ParsedAction> {
    let mut actions = Vec::new();
    let mut pos = 0;
    let input_bytes = input.as_bytes();

    while pos < input_bytes.len() {
        // Find start of action tag
        if let Some(start_idx) = input[pos..].find("[ACTION:") {
            let tag_start = pos + start_idx;
            pos = tag_start + "[ACTION:".len();

            // Find closing bracket
            if let Some(end_idx) = input[pos..].find(']') {
                let tag_end = pos + end_idx;
                let tag_content = &input[pos..tag_end];
                pos = tag_end + 1;

                if let Some(action) = parse_single_tag(tag_content) {
                    actions.push(action);
                }
            } else {
                // No closing bracket found, skip this position
                pos += 1;
            }
        } else {
            break;
        }
    }

    actions
}

fn parse_single_tag(content: &str) -> Option<ParsedAction> {
    let mut parts = content.split_whitespace();
    let tag_name = parts.next()?.to_string();

    let mut parameters = HashMap::new();
    let mut remaining = parts.collect::<Vec<_>>().join(" ");

    while !remaining.is_empty() {
        remaining = remaining.trim_start().to_string();
        if remaining.is_empty() {
            break;
        }

        // Find equals sign
        let eq_pos = match remaining.find('=') {
            Some(p) => p,
            None => break,
        };

        let key = remaining[..eq_pos].trim().to_string();
        remaining = remaining[eq_pos + 1..].to_string();

        // Find opening quote
        remaining = remaining.trim_start().to_string();
        if !remaining.starts_with('"') {
            break;
        }
        remaining = remaining[1..].to_string();

        // Parse value with escaped quotes
        let mut value = String::new();
        let mut escaped = false;
        let mut value_end = None;

        for (i, c) in remaining.chars().enumerate() {
            if escaped {
                value.push(c);
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                value_end = Some(i);
                break;
            } else {
                value.push(c);
            }
        }

        match value_end {
            Some(end) => {
                parameters.insert(key, value);
                remaining = remaining[end + 1..].to_string();
            }
            None => break,
        }
    }

    Some(ParsedAction {
        tag_name,
        parameters,
    })
}
