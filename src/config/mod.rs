//! config/mod.rs
//!
//! Configuration module root for PolyBox application settings.
//!
//! Responsibilities:
//! - Export configuration module public interface
//! - Re-export configuration models for external use
//! - Define module structure for configuration handling
//!
//! Dependencies:
//! - config/models.rs module
//!
//! Last updated: 2026-04-20

pub mod models;

pub use models::*;
