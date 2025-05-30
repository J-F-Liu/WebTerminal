use anyhow::Context;
use std::env;
use tower_http::services::ServeDir;
use tracing::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod service;
mod shell;
use service::*;
use shell::Shell;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let host = env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or("8000".to_string());
    let shell = env::var("SHELL").unwrap_or("cmd".to_string());
    let work_dir = env::var("WORK_DIR")
        .map(|dir| std::path::PathBuf::from(dir))
        .unwrap_or(env::current_dir().unwrap());
    let logs_dir = work_dir.join("logs");
    let server_url = format!("{host}:{port}");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Listening on {}", &server_url);
    info!("Work directory: {}", work_dir.display());
    info!("Command shell: {}", &shell);

    let state = AppState {
        shell: Shell::from_name(&shell),
        work_dir,
    };

    // build our application with some routes
    let router = create_router(state)
        .nest_service("/logs", ServeDir::new(logs_dir))
        .fallback_service(ServeDir::new("public").precompressed_br());

    // run it
    let listener = tokio::net::TcpListener::bind(server_url)
        .await
        .context("failed to bind TCP listener")?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("axum::serve failed")?;
    Ok(())
}

pub async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.expect("Listen for Ctrl+C");
    info!("Shutdown server...");
}
