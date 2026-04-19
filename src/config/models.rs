//! Model configuration structs and file I/O

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use serde::{Deserialize, Serialize};
use tracing::{info, debug};

/// Orchestrator-specific model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub backend_type: Option<String>,
    pub model_path: Option<String>,
    pub api_base_url: Option<String>,
    pub model_name: Option<String>,
    pub port: i64,
    pub auto_start: bool,
    pub ctx_size: Option<i64>,
    pub output_token_reserve: Option<i64>,
    pub n_gpu_layers: Option<i64>,
    pub temperature: Option<f64>,
    pub repeat_penalty: Option<f64>,
    pub cache_type_k: Option<String>,
    pub cache_type_v: Option<String>,
    pub flash_attn: Option<bool>,
    pub cache_ram: Option<bool>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            backend_type: Some("llama.cpp".to_string()),
            model_path: Some(String::new()),
            api_base_url: None,
            model_name: None,
            port: 11434,
            auto_start: false,
            ctx_size: Some(32000),
            output_token_reserve: Some(0),
            n_gpu_layers: Some(-1),
            temperature: Some(0.9),
            repeat_penalty: Some(1.1),
            cache_type_k: Some("f16".to_string()),
            cache_type_v: Some("f16".to_string()),
            flash_attn: Some(false),
            cache_ram: Some(false),
        }
    }
}

/// Embedding-specific model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub backend_type: Option<String>,
    pub model_path: Option<String>,
    pub api_base_url: Option<String>,
    pub model_name: Option<String>,
    pub port: i64,
    pub auto_start: bool,
    pub embedding_ctx_size: Option<i64>,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            backend_type: None,
            model_path: Some(String::new()),
            api_base_url: None,
            model_name: None,
            port: 11435,
            auto_start: false,
            embedding_ctx_size: Some(512),
        }
    }
}

/// Returns the path to the orchestrator config JSON file
fn orchestrator_path(data_dir: &Path) -> PathBuf {
    data_dir.join("orchestrator.json")
}

/// Returns the path to the embedding config JSON file
fn embedding_path(data_dir: &Path) -> PathBuf {
    data_dir.join("embedding.json")
}

/// Load orchestrator config from {data_dir}/orchestrator.json
/// If the file doesn't exist, creates it with defaults and returns the default
pub fn load_orchestrator_config(data_dir: &Path) -> std::io::Result<OrchestratorConfig> {
    let path = orchestrator_path(data_dir);
    load_config_from_path(&path, OrchestratorConfig::default())
}

/// Save orchestrator config to {data_dir}/orchestrator.json
pub fn save_orchestrator_config(data_dir: &Path, config: &OrchestratorConfig) -> std::io::Result<()> {
    let path = orchestrator_path(data_dir);
    save_config_to_path(&path, config)
}

/// Load embedding config from {data_dir}/embedding.json
/// If the file doesn't exist, creates it with defaults and returns the default
pub fn load_embedding_config(data_dir: &Path) -> std::io::Result<EmbeddingConfig> {
    let path = embedding_path(data_dir);
    load_config_from_path(&path, EmbeddingConfig::default())
}

/// Save embedding config to {data_dir}/embedding.json
pub fn save_embedding_config(data_dir: &Path, config: &EmbeddingConfig) -> std::io::Result<()> {
    let path = embedding_path(data_dir);
    save_config_to_path(&path, config)
}

/// Generic load — reads JSON from path, falls back to default if file missing
fn load_config_from_path<T: serde::de::DeserializeOwned + Default + serde::Serialize>(
    path: &Path,
    _default_marker: T,
) -> std::io::Result<T> {
    if !path.exists() {
        let default = T::default();
        debug!("Config file {:?} not found, creating with defaults", path);
        // Ensure parent dir exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        save_config_to_path(path, &default)?;
        info!("Created default config at {:?}", path);
        return Ok(default);
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config: T = serde_json::from_reader(reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(config)
}

/// Generic save — writes JSON to path with pretty formatting
fn save_config_to_path<T: serde::Serialize>(path: &Path, config: &T) -> std::io::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(())
}
