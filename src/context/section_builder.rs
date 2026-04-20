// src/context/section_builder.rs
// Individual prompt section builders for context assembly pipeline
//
// Responsibilities:
// - Implement SectionBuilder struct for constructing each of the 8 standard prompt sections
// - Provide dedicated method for every context section type
// - Use StaticFormatter for consistent standardized section formatting
// - Maintain strict boundary between section construction and final assembly
//
// Dependencies:
// - StaticFormatter from src/context/static_formatter.rs
// - Context assembly specification from pb_context_assembly.md
//
// Created: 2026-04-20

use super::StaticFormatter;

/// Section builder for constructing individual prompt context sections
///
/// Provides dedicated methods for each of the 8 standard context sections
/// as defined in the context assembly specification. All sections are
/// formatted using the standardized StaticFormatter.
#[derive(Debug, Clone, Default)]
pub struct SectionBuilder {
    formatter: StaticFormatter,
}

impl SectionBuilder {
    /// Create a new SectionBuilder instance
    pub fn new() -> Self {
        Self {
            formatter: StaticFormatter::new(),
        }
    }

    /// Build Core Values section (Section 1)
    pub fn build_core_values(&self, content: &str) -> String {
        self.formatter.build_section("Core Values", content)
    }

    /// Build Personality section (Section 2)
    pub fn build_personality(&self, content: &str) -> String {
        self.formatter.build_section("Personality", content)
    }

    /// Build Action Registry section (Section 3)
    pub fn build_action_registry(&self, content: &str) -> String {
        self.formatter.build_section("Action Registry", content)
    }

    /// Build Current Time section (Section 4)
    pub fn build_current_time(&self, datetime: &str) -> String {
        let content = format!("Current time: {}", datetime);
        self.formatter.build_section("Current Time", &content)
    }

    /// Build Memory Summary section (Section 5)
    pub fn build_memory_summary(&self, content: &str) -> String {
        self.formatter.build_section("Relevant Memories", content)
    }

    /// Build Chat History section (Section 6)
    pub fn build_chat_history(&self, content: &str) -> String {
        self.formatter.build_section("Chat History", content)
    }

    /// Build Tool Results section (Section 7)
    pub fn build_tool_results(&self, content: &str) -> String {
        self.formatter.build_section("Tool Results", content)
    }

    /// Build User Message section (Section 8)
    pub fn build_user_message(&self, message: &str) -> String {
        let content = format!("User: {}", message);
        self.formatter.build_section("User Message", &content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_builder_creation() {
        let builder = SectionBuilder::new();
        assert!(true);
    }

    #[test]
    fn test_all_section_methods_exist() {
        let builder = SectionBuilder::new();
        
        // Verify all 8 builder methods are present and compile
        let _cv = builder.build_core_values("test");
        let _p = builder.build_personality("test");
        let _ar = builder.build_action_registry("test");
        let _ct = builder.build_current_time("2026-04-20T17:00:00Z");
        let _ms = builder.build_memory_summary("test");
        let _ch = builder.build_chat_history("test");
        let _tr = builder.build_tool_results("test");
        let _um = builder.build_user_message("test");
    }
}
