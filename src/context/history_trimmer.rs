// src/context/history_trimmer.rs
// Chat history trimming strategy implementation for context budget management
//
// Responsibilities:
// - Implement oldest-first history trimming algorithm
// - Preserve complete message boundaries during trimming
// - Maintain priority order of remaining entries
// - Validate token budgets using token calculator
//
// Dependencies:
// - HistoryTrimming trait from src/context/mod.rs
// - TokenCalculator for token count validation
//
// Created: 2026-04-20

use super::{HistoryTrimming, TokenCalculator, TokenCount};

/// History trimmer implementation that removes oldest entries first while preserving message boundaries
#[derive(Debug, Clone, Default)]
pub struct HistoryTrimmer {
    token_calculator: TokenCalculator,
}

impl HistoryTrimmer {
    /// Create new HistoryTrimmer instance with default token calculator
    pub fn new() -> Self {
        Self::default()
    }

    /// Create HistoryTrimmer with custom token calculator
    pub fn with_calculator(calculator: TokenCalculator) -> Self {
        Self {
            token_calculator: calculator,
        }
    }
}

impl HistoryTrimming for HistoryTrimmer {
    /// Trim history entries to fit within max token budget
    /// Removes oldest entries first, always preserves complete messages
    /// Returns entries in original order (newest entries are retained)
    fn trim_history(&self, entries: Vec<&str>, max_tokens: usize) -> Vec<String> {
        if entries.is_empty() || max_tokens == 0 {
            return Vec::new();
        }

        // Calculate cumulative tokens starting from newest entry
        let mut total_tokens = 0;
        let mut kept_entries = Vec::new();

        // Iterate from newest to oldest (reverse order)
        for entry in entries.iter().rev() {
            let entry_tokens = self.token_calculator.count_tokens(entry);

            // If adding this entry would exceed budget, stop
            if total_tokens + entry_tokens > max_tokens {
                break;
            }

            total_tokens += entry_tokens;
            kept_entries.push(entry.to_string());
        }

        // Reverse back to original chronological order
        kept_entries.reverse();
        kept_entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_empty_entries_returns_empty() {
        let trimmer = HistoryTrimmer::new();
        let result = trimmer.trim_history(vec![], 100);
        assert!(result.is_empty());
    }

    #[test]
    fn trim_zero_max_tokens_returns_empty() {
        let trimmer = HistoryTrimmer::new();
        let entries = vec!["test message"];
        let result = trimmer.trim_history(entries, 0);
        assert!(result.is_empty());
    }
}
