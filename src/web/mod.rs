//! Web server module

pub mod routes;
pub mod health;
pub mod hardware;
pub mod services;
pub mod route_settings;
pub mod actions;
pub mod models;
pub mod backups;

use axum::{
    body::Body,
    routing::get,
    Router,
    response::{Html, IntoResponse, Response},
};
use std::sync::Arc;
use http::header;

use crate::AppState;

pub fn create_app(state: Arc<AppState>) -> Router {
    let static_files = Router::new()
        .route("/", get(index_handler))
        .route("/index.html", get(index_handler))
        .route("/styles.css", get(css_handler))
        .route("/pages/:page", get(page_handler))
        .route("/js/:file", get(js_handler));

    let api = routes::create_router(state);

    static_files.merge(api)
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../../web/index.html"))
}

async fn css_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        include_str!("../../web/styles.css"),
    )
}

fn get_web_root() -> std::path::PathBuf {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = std::path::PathBuf::from(manifest_dir).join("web");
        if path.exists() {
            return path;
        }
    }
    std::path::PathBuf::from("web")
}

async fn page_handler(axum::extract::Path(page): axum::extract::Path<String>) -> impl IntoResponse {
    let web_root = get_web_root();
    let path = web_root.join("pages").join(&page);
    match std::fs::read_to_string(&path) {
        Ok(content) => Html(content).into_response(),
        Err(_) => Response::builder()
            .status(404)
            .body(Body::from("<h1>Not found</h1>"))
            .unwrap()
            .into_response(),
    }
}

async fn js_handler(axum::extract::Path(file): axum::extract::Path<String>) -> impl IntoResponse {
    let web_root = get_web_root();
    let path = web_root.join("js").join(&file);
    match std::fs::read_to_string(&path) {
        Ok(content) => (
            [(header::CONTENT_TYPE, "application/javascript")],
            content,
        ).into_response(),
        Err(_) => Response::builder()
            .status(404)
            .body(Body::from("Not found"))
            .unwrap()
            .into_response(),
    }
}