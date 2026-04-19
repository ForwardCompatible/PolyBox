//! Hardware monitoring — CPU, RAM, and GPU/VRAM via NVML

use serde::Serialize;
use sysinfo::System;

#[derive(Debug, Clone, Serialize)]
pub struct HardwareStats {
    pub cpu_percent: f32,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub vram_used_gb: Option<f64>,
    pub vram_total_gb: Option<f64>,
}

pub struct NvmlHandle {
    inner: nvml_wrapper::Nvml,
}

impl std::fmt::Debug for NvmlHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NvmlHandle").finish()
    }
}

impl NvmlHandle {
    pub fn new() -> Option<Self> {
        match nvml_wrapper::Nvml::init() {
            Ok(inner) => Some(Self { inner }),
            Err(e) => {
                tracing::warn!("NVML init failed (GPU monitoring disabled): {:?}", e);
                None
            }
        }
    }

    pub fn gpu_stats(&self) -> (Option<f64>, Option<f64>) {
        match self.inner.device_by_index(0) {
            Ok(device) => {
                match device.memory_info() {
                    Ok(mem) => {
                        let used = mem.used as f64 / 1_073_741_824.0;
                        let total = mem.total as f64 / 1_073_741_824.0;
                        (Some(used), Some(total))
                    }
                    Err(e) => {
                        tracing::warn!("Failed to read GPU memory info: {:?}", e);
                        (None, None)
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to get GPU device: {:?}", e);
                (None, None)
            }
        }
    }
}

pub fn get_stats(nvml: Option<&NvmlHandle>) -> HardwareStats {
    let mut sys = System::new_all();
    sys.refresh_cpu_all();
    let cpu = sys.global_cpu_usage();
    let total_mem = sys.total_memory() as f64 / 1_073_741_824.0;
    let used_mem = sys.used_memory() as f64 / 1_073_741_824.0;

    let (vram_used, vram_total) = nvml.map(|n| n.gpu_stats()).unwrap_or((None, None));

    HardwareStats {
        cpu_percent: cpu,
        ram_used_gb: used_mem,
        ram_total_gb: total_mem,
        vram_used_gb: vram_used,
        vram_total_gb: vram_total,
    }
}
