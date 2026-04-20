// src/context/static_formatter.rs
// Static prompt section formatting implementation for standardized context assembly
//
// Responsibilities:
// - Implement standardized formatting for prompt context sections
// - Provide consistent header styling, section boundaries, and indentation
// - Implement the SectionBuilding trait interface
// - Ensure uniform presentation across all context sections
//
// Dependencies:
// - SectionBuilding trait from src/context/mod.rs
//
// Created: 2026-04-20

use super::SectionBuilding;

/// Static formatter for prompt context sections
/// 
/// Provides standardized, consistent formatting for all context sections
/// with uniform headers, boundaries, and indentation rules.
#[derive(Debug, Clone, Default)]
pub struct StaticFormatter;

impl StaticFormatter {
    /// Create a new StaticFormatter instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Indent all lines in content by specified number of spaces
    fn indent_content(&self, content: &str, indent: usize) -> String {
        let indent_str = " ".repeat(indent);
        content
            .lines()
            .map(|line| format!("{}{}", indent_str, line))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl SectionBuilding for StaticFormatter {
    /// Build a formatted context section with standardized header and boundaries
    ///
    /// Format structure:
    /// ┌────────────────────────────────────────
    /// │ TITLE
    /// ├────────────────────────────────────────
    ///   content lines properly indented
    /// └────────────────────────────────────────
    fn build_section(&self, title: &str, content: &str) -> String {
        const BOUNDARY_WIDTH: usize = 48;
        let boundary = "─".repeat(BOUNDARY_WIDTH);
        
        let indented_content = self.indent_content(content, 2);
        
        format!(
            "┌{}\n│ {}\n├{}\n{}\n└{}",
            boundary,
            title.to_uppercase(),
            boundary,
            indented_content,
            boundary
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_formatting() {
        let formatter = StaticFormatter::new();
        let result = formatter.build_section("Test Section", "line 1\nline 2\nline 3");
        
        assert!(result.contains("TEST SECTION"));
        assert!(result.contains("  line 1"));
        assert!(result.contains("┌────────────────────────────────────────"));
        assert!(result.contains("└────────────────────────────────────────"));
    }
}
