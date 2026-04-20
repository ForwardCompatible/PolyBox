// src/context/assembly_pipeline.rs
// Linear 8-section context assembly pipeline with token budget enforcement
//
// Responsibilities:
// - Execute context assembly sections in fixed priority order
// - Validate token budget after each section execution
// - Track accumulated token usage during assembly
// - Halt assembly immediately when token budget is exceeded
//
// Dependencies:
// - TokenCalculator for real-time token counting
// - Section builders for individual context sections
//
// Created: 2026-04-20

use crate::context::token_calculator::TokenCalculator;
use crate::context::section_builder::SectionBuilder;

/// Assembly pipeline execution result
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineResult {
    /// Pipeline completed successfully within token budget
    Completed,
    /// Pipeline halted due to token budget exceeded
    TokenBudgetExceeded,
    /// Pipeline failed during section execution
    SectionFailed,
}

/// Linear 8-section context assembly pipeline
pub struct AssemblyPipeline {
    token_calculator: TokenCalculator,
    token_budget: u32,
    accumulated_tokens: u32,
}

impl AssemblyPipeline {
    /// Create new assembly pipeline with specified token budget
    pub fn new(token_budget: u32) -> Self {
        Self {
            token_calculator: TokenCalculator::new(),
            token_budget,
            accumulated_tokens: 0,
        }
    }

    /// Execute full pipeline in fixed section order
    pub fn execute(&mut self, builder: &mut SectionBuilder) -> PipelineResult {
        // Fixed priority execution order - 8 sections
        let sections = [
            Self::execute_system_prompt,
            Self::execute_core_memory,
            Self::execute_recent_actions,
            Self::execute_context_seeds,
            Self::execute_tool_results,
            Self::execute_working_memory,
            Self::execute_conversation_history,
            Self::execute_user_prompt,
        ];

        for section_fn in sections {
            match section_fn(self, builder) {
                PipelineResult::Completed => continue,
                other => return other,
            }
        }

        PipelineResult::Completed
    }

    /// Validate token budget after section execution
    fn validate_budget(&mut self, section_tokens: u32) -> PipelineResult {
        self.accumulated_tokens += section_tokens;

        if self.accumulated_tokens > self.token_budget {
            return PipelineResult::TokenBudgetExceeded;
        }

        PipelineResult::Completed
    }

    // Section 1: System Prompt
    fn execute_system_prompt(&mut self, builder: &mut SectionBuilder) -> PipelineResult {
        let tokens = builder.build_system_prompt();
        self.validate_budget(tokens)
    }

    // Section 2: Core Memory
    fn execute_core_memory(&mut self, builder: &mut SectionBuilder) -> PipelineResult {
        let tokens = builder.build_core_memory();
        self.validate_budget(tokens)
    }

    // Section 3: Recent Actions
    fn execute_recent_actions(&mut self, builder: &mut SectionBuilder) -> PipelineResult {
        let tokens = builder.build_recent_actions();
        self.validate_budget(tokens)
    }

    // Section 4: Context Seeds
    fn execute_context_seeds(&mut self, builder: &mut SectionBuilder) -> PipelineResult {
        let tokens = builder.build_context_seeds();
        self.validate_budget(tokens)
    }

    // Section 5: Tool Results
    fn execute_tool_results(&mut self, builder: &mut SectionBuilder) -> PipelineResult {
        let tokens = builder.build_tool_results();
        self.validate_budget(tokens)
    }

    // Section 6: Working Memory
    fn execute_working_memory(&mut self, builder: &mut SectionBuilder) -> PipelineResult {
        let tokens = builder.build_working_memory();
        self.validate_budget(tokens)
    }

    // Section 7: Conversation History
    fn execute_conversation_history(&mut self, builder: &mut SectionBuilder) -> PipelineResult {
        let tokens = builder.build_conversation_history();
        self.validate_budget(tokens)
    }

    // Section 8: User Prompt
    fn execute_user_prompt(&mut self, builder: &mut SectionBuilder) -> PipelineResult {
        let tokens = builder.build_user_prompt();
        self.validate_budget(tokens)
    }

    /// Get current accumulated token count
    pub fn accumulated_tokens(&self) -> u32 {
        self.accumulated_tokens
    }

    /// Get configured token budget
    pub fn token_budget(&self) -> u32 {
        self.token_budget
    }
}
