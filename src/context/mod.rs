// src/context/mod.rs
// Context assembly module root and public interface definitions
//
// Responsibilities:
// - Module root for all context assembly functionality
// - Declare and export public submodules
// - Define public trait interfaces and type aliases
// - Expose minimal public API for context construction
//
// Dependencies:
// - Will depend on core types, database models, and tokenization libraries
//
// Created: 2026-04-20

// Submodule declarations
mod token_calculator;
mod history_trimmer;
mod static_formatter;
mod section_builder;
mod assembly_pipeline;
mod context_assembler;

// Public exports
pub use token_calculator::TokenCalculator;
pub use history_trimmer::HistoryTrimmer;
pub use static_formatter::StaticFormatter;
pub use section_builder::SectionBuilder;
pub use assembly_pipeline::AssemblyPipeline;
pub use context_assembler::ContextAssembler;

/// Trait for components that can calculate token counts for content
pub trait TokenCount {
    /// Calculate total tokens for given input text
    fn count_tokens(&self, input: &str) -> usize;
}

/// Trait for components that can trim history to fit within token limits
pub trait HistoryTrimming {
    /// Trim history entries while preserving priority and context boundaries
    fn trim_history(&self, entries: Vec<&str>, max_tokens: usize) -> Vec<String>;
}

/// Trait for building structured context sections
pub trait SectionBuilding {
    /// Build a formatted context section with header and content
    fn build_section(&self, title: &str, content: &str) -> String;
}

/// Trait for assembling complete context from multiple components
pub trait ContextAssembly {
    /// Assemble final context string from all configured sources
    fn assemble_context(&self) -> Result<String, ContextError>;
}

/// Error type for context assembly operations
#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("Token limit exceeded: {0} > {1}")]
    TokenLimitExceeded(usize, usize),

    #[error("Invalid context section: {0}")]
    InvalidSection(String),

    #[error("Assembly pipeline failure: {0}")]
    PipelineFailure(String),
}

// Type aliases for common types
pub type TokenCount = usize;
pub type ContextString = String;
pub type ContextResult<T> = Result<T, ContextError>;
