// src/actions/mod.rs
// Action tag system module
//
// Purpose: Module declaration for action tag parsing and handling
// Responsibilities:
// - Expose public interfaces for action tag system
// - Organize submodules for parsing, dispatching, and execution
//
// Dependencies:
// - parser.rs for tag parsing functionality
//
// Created: 2026-04-20

pub mod parser;
pub mod ordering;
pub mod dispatcher;
pub mod results;
pub mod stripper;

pub use parser::{ParsedAction, parse_action_tags};
pub use ordering::order_actions_for_execution;
pub use dispatcher::{Dispatcher, HandlerResult};
