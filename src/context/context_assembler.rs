// src/context/context_assembler.rs
// Main orchestrator component that coordinates all context assembly operations
//
// Responsibilities:
// - Orchestrate the full context assembly workflow
// - Coordinate all lower-level context components
// - Implement the public ContextAssembly trait interface
// - Manage execution order of assembly pipeline stages
//
// Dependencies:
// - TokenCalculator for token counting operations
// - HistoryTrimmer for history trimming logic
// - StaticFormatter for static content formatting
// - SectionBuilder for structured section construction
// - AssemblyPipeline for final assembly execution
// - ContextAssembly trait from module root
//
// Created: 2026-04-20

use super::{
    TokenCalculator,
    HistoryTrimmer,
    StaticFormatter,
    SectionBuilder,
    AssemblyPipeline,
    ContextAssembly,
    ContextError,
    ContextResult,
};

/// Main context assembly orchestrator
///
/// Coordinates all components required to build complete context strings.
/// This struct holds references to all required lower-level components
/// and implements the top-level assembly workflow.
#[derive(Debug, Clone)]
pub struct ContextAssembler {
    token_calculator: TokenCalculator,
    history_trimmer: HistoryTrimmer,
    static_formatter: StaticFormatter,
    section_builder: SectionBuilder,
    assembly_pipeline: AssemblyPipeline,
}

impl ContextAssembler {
    /// Create a new ContextAssembler with default component instances
    pub fn new() -> Self {
        Self {
            token_calculator: TokenCalculator::new(),
            history_trimmer: HistoryTrimmer::new(),
            static_formatter: StaticFormatter::new(),
            section_builder: SectionBuilder::new(),
            assembly_pipeline: AssemblyPipeline::new(),
        }
    }

    /// Create a ContextAssembler with pre-configured component instances
    pub fn with_components(
        token_calculator: TokenCalculator,
        history_trimmer: HistoryTrimmer,
        static_formatter: StaticFormatter,
        section_builder: SectionBuilder,
        assembly_pipeline: AssemblyPipeline,
    ) -> Self {
        Self {
            token_calculator,
            history_trimmer,
            static_formatter,
            section_builder,
            assembly_pipeline,
        }
    }
}

impl Default for ContextAssembler {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextAssembly for ContextAssembler {
    /// Assemble complete context string by coordinating all components
    ///
    /// Executes the full assembly workflow in correct order:
    /// 1. Format static context sections
    /// 2. Calculate available token budget
    /// 3. Trim history to fit within limits
    /// 4. Build structured context sections
    /// 5. Execute final assembly pipeline
    fn assemble_context(&self) -> ContextResult<String> {
        // Format static system context
        let static_context = self.static_formatter.format_static_sections()?;

        // Calculate remaining token budget for dynamic content
        let static_tokens = self.token_calculator.count_tokens(&static_context);
        let available_tokens = self.token_calculator.get_available_budget(static_tokens);

        // Trim history entries to fit remaining token budget
        let trimmed_history = self.history_trimmer.trim_history(available_tokens)?;

        // Build structured sections from trimmed content
        let history_section = self.section_builder.build_section("History", &trimmed_history);
        let system_section = self.section_builder.build_section("System", &static_context);

        // Execute final assembly pipeline
        let final_context = self.assembly_pipeline.assemble(vec![
            system_section,
            history_section,
        ])?;

        Ok(final_context)
    }
}
