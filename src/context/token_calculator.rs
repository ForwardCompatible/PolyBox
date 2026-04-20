// src/context/token_calculator.rs
// Token counting and budget calculation for context assembly
//
// Responsibilities:
// - Implement token counting functionality for text content
// - Track token budget allocation and consumption
// - Calculate remaining available tokens
// - Provide budget validation checks
//
// Dependencies:
// - TokenCount trait from src/context/mod.rs
// - Standard library only
//
// Created: 2026-04-20

use super::TokenCount;

/// Token calculator with budget tracking capabilities
#[derive(Debug, Clone, Copy)]
pub struct TokenCalculator {
    total_budget: usize,
    used_tokens: usize,
}

impl TokenCalculator {
    /// Create new TokenCalculator with specified total token budget
    pub fn new(total_budget: usize) -> Self {
        Self {
            total_budget,
            used_tokens: 0,
        }
    }

    /// Get total allocated token budget
    pub fn total_budget(&self) -> usize {
        self.total_budget
    }

    /// Get number of tokens already consumed
    pub fn used_tokens(&self) -> usize {
        self.used_tokens
    }

    /// Calculate remaining available tokens
    pub fn remaining_tokens(&self) -> usize {
        self.total_budget.saturating_sub(self.used_tokens)
    }

    /// Check if given token count fits within remaining budget
    pub fn fits_in_budget(&self, token_count: usize) -> bool {
        token_count <= self.remaining_tokens()
    }

    /// Consume specified number of tokens from budget
    /// Returns Ok(remaining) on success, Err(required, remaining) if insufficient
    pub fn consume_tokens(&mut self, token_count: usize) -> Result<usize, (usize, usize)> {
        if self.fits_in_budget(token_count) {
            self.used_tokens += token_count;
            Ok(self.remaining_tokens())
        } else {
            Err((token_count, self.remaining_tokens()))
        }
    }

    /// Reset used token counter to zero
    pub fn reset_budget(&mut self) {
        self.used_tokens = 0;
    }

    /// Update total budget while preserving current usage
    pub fn set_total_budget(&mut self, new_budget: usize) {
        self.total_budget = new_budget;
    }
}

impl TokenCount for TokenCalculator {
    /// Count tokens in input string
    /// Current implementation uses character-based estimation
    /// Will be replaced with proper tokenizer in future iteration
    fn count_tokens(&self, input: &str) -> usize {
        // Simple estimation: ~4 characters per token as standard baseline
        input.chars().count().div_ceil(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_calculator_budget_tracking() {
        let mut calc = TokenCalculator::new(1000);
        assert_eq!(calc.total_budget(), 1000);
        assert_eq!(calc.used_tokens(), 0);
        assert_eq!(calc.remaining_tokens(), 1000);

        assert!(calc.fits_in_budget(500));
        assert_eq!(calc.consume_tokens(500), Ok(500));
        assert_eq!(calc.used_tokens(), 500);
        assert_eq!(calc.remaining_tokens(), 500);

        assert!(!calc.fits_in_budget(600));
        assert_eq!(calc.consume_tokens(600), Err((600, 500)));

        calc.reset_budget();
        assert_eq!(calc.used_tokens(), 0);
        assert_eq!(calc.remaining_tokens(), 1000);
    }

    #[test]
    fn token_count_estimation() {
        let calc = TokenCalculator::new(1000);
        assert_eq!(calc.count_tokens(""), 0);
        assert_eq!(calc.count_tokens("test"), 1);
        assert_eq!(calc.count_tokens("testing 123"), 3);
        assert_eq!(calc.count_tokens("four character tokens"), 6);
    }
}
