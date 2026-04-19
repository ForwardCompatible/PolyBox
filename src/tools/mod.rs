//! Service process management
//! Handles launching and monitoring llama-server and embedding service processes

use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tracing::info;
use crate::config::{OrchestratorConfig, EmbeddingConfig};

#[derive(Debug)]
pub struct ServiceManager {
    orchestrator: Mutex<Option<Child>>,
    embedding: Mutex<Option<Child>>,
}

impl Clone for ServiceManager {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            orchestrator: Mutex::new(None),
            embedding: Mutex::new(None),
        }
    }

    pub fn start_orchestrator(&self, model_path: &str, port: u16, ctx_size: i64, config: &OrchestratorConfig) -> Result<(), String> {
        // Check if already running
        {
            let orch = self.orchestrator.lock().map_err(|e| e.to_string())?;
            if orch.is_some() {
                return Ok(()); // Already running
            }
        }

        info!("Starting orchestrator: {} on port {}", model_path, port);

        let mut cmd = Command::new("llama-server");
        cmd.arg("--model").arg(model_path)
           .arg("--ctx-size").arg(ctx_size.to_string())
           .arg("--port").arg(port.to_string())
           .arg("--host").arg("127.0.0.1");

        // GPU layers
        if let Some(n_gpu_layers) = config.n_gpu_layers {
            if n_gpu_layers != 0 {
                cmd.arg("--n-gpu-layers").arg(n_gpu_layers.to_string());
            }
        }

        // Temperature
        if let Some(temp) = config.temperature {
            cmd.arg("--temperature").arg(temp.to_string());
        }

        // Repeat penalty
        if let Some(repeat_penalty) = config.repeat_penalty {
            cmd.arg("--repeat-penalty").arg(repeat_penalty.to_string());
        }

        // Cache type K
        if let Some(ref cache_type_k) = config.cache_type_k {
            if !cache_type_k.is_empty() {
                cmd.arg("--cache-type-k").arg(cache_type_k);
            }
        }

        // Cache type V
        if let Some(ref cache_type_v) = config.cache_type_v {
            if !cache_type_v.is_empty() {
                cmd.arg("--cache-type-v").arg(cache_type_v);
            }
        }

        // Flash attention
        if config.flash_attn == Some(true) {
            cmd.arg("--flash-attn");
        }

        // Cache in RAM
        if config.cache_ram == Some(true) {
            cmd.arg("--cache-ram");
        }

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let child = cmd.spawn()
            .map_err(|e| format!("failed to start llama-server: {}", e))?;

        let mut orch = self.orchestrator.lock().map_err(|e| e.to_string())?;
        *orch = Some(child);

        Ok(())
    }

    pub fn stop_orchestrator(&self) -> Result<(), String> {
        // Take child out of lock first, then we can kill it
        let maybe_child = {
            let mut orch = self.orchestrator.lock().map_err(|e| e.to_string())?;
            orch.take()
        };
        if let Some(mut child) = maybe_child {
            info!("Stopping orchestrator");
            let _ = child.kill();
        }
        Ok(())
    }

    pub fn is_orchestrator_running(&self) -> bool {
        self.orchestrator.lock().map(|orch| orch.is_some()).unwrap_or(false)
    }

    pub fn start_embedding(&self, model_path: &str, port: u16, ctx_size: i64, _config: &EmbeddingConfig) -> Result<(), String> {
        // Check if already running
        {
            let emb = self.embedding.lock().map_err(|e| e.to_string())?;
            if emb.is_some() {
                return Ok(()); // Already running
            }
        }

        info!("Starting embedding service: {} on port {} (CPU-only)", model_path, port);

        let child = Command::new("llama-server")
            .arg("--model").arg(model_path)
            .arg("--ctx-size").arg(ctx_size.to_string())
            .arg("--port").arg(port.to_string())
            .arg("--host").arg("127.0.0.1")
            .arg("--embedding")
            // Embedding always runs CPU-only — --device none overrides CUDA init
            // to keep all computation (including KV cache) on CPU
            .arg("--device").arg("none")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to start embedding service: {}", e))?;

        let mut emb = self.embedding.lock().map_err(|e| e.to_string())?;
        *emb = Some(child);

        Ok(())
    }

    pub fn stop_embedding(&self) -> Result<(), String> {
        let maybe_child = {
            let mut emb = self.embedding.lock().map_err(|e| e.to_string())?;
            emb.take()
        };
        if let Some(mut child) = maybe_child {
            info!("Stopping embedding service");
            let _ = child.kill();
        }
        Ok(())
    }

    pub fn is_embedding_running(&self) -> bool {
        self.embedding.lock().map(|emb| emb.is_some()).unwrap_or(false)
    }

    pub fn shutdown(&self) {
        let _ = self.stop_orchestrator();
        let _ = self.stop_embedding();
        info!("All services stopped");
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}