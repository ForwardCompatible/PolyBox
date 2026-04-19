//! PolyBox 2.0 - AI Agent Framework

mod config;
mod db;
mod web;
mod tools;
mod hardware;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug)]
pub struct AppState {
    pub data_dir: PathBuf,
    pub core_db: db::DbPool,
    pub logs_db: db::DbPool,
    pub memory_db: db::DbPool,
    pub service_manager: tools::ServiceManager,
    pub ws_broadcast: broadcast::Sender<String>,
    pub nvml: Option<hardware::NvmlHandle>,
}

impl AppState {
    pub fn is_orchestrator_running(&self) -> bool {
        self.service_manager.is_orchestrator_running()
    }

    pub fn is_embedding_running(&self) -> bool {
        self.service_manager.is_embedding_running()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("polybox=info".parse()?))
        .init();

    info!("PolyBox 2.0 starting...");

    dotenvy::dotenv().ok();

    let data_dir = std::env::var("POLYBOX_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data"));

    std::fs::create_dir_all(&data_dir)?;
    std::fs::create_dir_all(data_dir.join("backups"))?;
    std::fs::create_dir_all(data_dir.join("backups").join("db"))?;

    let (core_db, logs_db, memory_db) = db::init(&data_dir)?;

    let settings = db::core::get_app_settings(&core_db)?;
    let port = settings.web_server_port;

    let (tx, _rx) = broadcast::channel(100);

    let nvml = hardware::NvmlHandle::new();
    if nvml.is_some() {
        info!("NVML initialized — GPU monitoring enabled");
    } else {
        info!("NVML not available — GPU monitoring disabled");
    }

    let app_state = Arc::new(AppState {
        data_dir: data_dir.clone(),
        core_db,
        logs_db,
        memory_db,
        service_manager: tools::ServiceManager::new(),
        ws_broadcast: tx,
        nvml,
    });

    let app = web::create_app(app_state);

    let addr: SocketAddr = ([127, 0, 0, 1], port as u16).into();

    info!("Web server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("PolyBox initialized successfully");
    info!("  Data directory: {:?}", data_dir);
    info!("  Web server port: {}", port);
    info!("  Agent name: {}", settings.agent_name);
    info!("  User name: {}", settings.user_name);

    axum::serve(listener, app).await?;

    info!("PolyBox shutting down...");
    Ok(())
}