//! models.rs
//! Model discovery and configuration operations
//!
//! Responsibilities:
//! - ModelInfo data structure
//! - ModelConfigUpdate request structure
//! - Discover installed models on filesystem
//!
//! Dependencies:
//! - std::fs
//! - std::path
//! - rusqlite
//! - serde

use std::path::PathBuf;
use rusqlite::Result;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelConfigUpdate {
    pub backend_type: Option<String>,
    pub model_path: Option<String>,
    pub api_base_url: Option<String>,
    pub model_name: Option<String>,
    pub port: Option<i64>,
    pub auto_start: Option<bool>,
    pub ctx_size: Option<i64>,
    pub output_token_reserve: Option<i64>,
    pub n_gpu_layers: Option<i64>,
    pub temperature: Option<f64>,
    pub repeat_penalty: Option<f64>,
    pub cache_type_k: Option<String>,
    pub cache_type_v: Option<String>,
    pub flash_attn: Option<bool>,
    pub cache_ram: Option<bool>,
    pub embedding_ctx_size: Option<i64>,
    pub dim: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelInfo {
    pub repo_name: String,
    pub filename: String,
    pub full_path: String,
    pub file_size_bytes: i64,
}

pub fn discover_models(model_type: &str) -> Result<Vec<ModelInfo>, String> {
    // Models are at ./models/<type>/<repo>/<file.gguf
    let model_base = PathBuf::from("models").join(model_type);
    if !model_base.exists() {
        return Ok(vec![]);
    }

    let mut models = Vec::new();
    let entries = std::fs::read_dir(&model_base)
        .map_err(|e| format!("Failed to read models directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let repo_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Check if folder contains any mmproj*.gguf files
            // If so, skip ALL mmproj*.gguf files (they're projector files, not standalone models)
            let has_mmproj = if let Ok(gguf_entries) = std::fs::read_dir(&path) {
                gguf_entries.flatten().any(|e| {
                    let binding = e.file_name();
                    let name = binding.to_str().unwrap_or("");
                    name.to_lowercase().contains("mmproj") && name.to_lowercase().ends_with(".gguf")
                })
            } else {
                false
            };

            // Look for .gguf files in the subdirectory
            if let Ok(gguf_entries) = std::fs::read_dir(&path) {
                for gguf in gguf_entries.flatten() {
                    let gguf_path = gguf.path();
                    if gguf_path.extension().and_then(|e| e.to_str()) == Some("gguf") {
                        let filename = gguf_path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();

                        // Skip mmproj*.gguf files if folder has any mmproj file
                        if has_mmproj && filename.to_lowercase().contains("mmproj") {
                            continue;
                        }

                        let file_size = std::fs::metadata(&gguf_path)
                            .map(|m| m.len())
                            .unwrap_or(0);

                        models.push(ModelInfo {
                            repo_name: repo_name.clone(),
                            filename,
                            full_path: gguf_path.to_string_lossy().to_string(),
                            file_size_bytes: file_size as i64,
                        });
                    }
                }
            }
        }
    }

    Ok(models)
}
