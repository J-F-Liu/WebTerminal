use axum::{
    Json, Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Path, State},
    response::Response,
    routing::{any, get, post},
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use normpath::PathExt;
use std::process::Command;
use tracing::*;

#[derive(Clone)]
pub struct AppState {
    pub shell: crate::shell::Shell,
    pub work_dir: std::path::PathBuf,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/shells", get(get_available))
        .route("/socket/{shell}", any(connect_socket))
        .route("/execute", post(execute_command))
        .with_state(state)
}

pub async fn get_available() -> Json<Vec<String>> {
    Json(
        crate::shell::available_shells()
            .into_iter()
            .map(|shell| shell.program().to_string())
            .collect::<Vec<String>>(),
    )
}

pub async fn execute_command(state: State<AppState>, command: String) -> String {
    execute_command_inner(&state, &command).await
}

async fn execute_command_inner(state: &AppState, command: &str) -> String {
    let shell = &state.shell;
    match Command::new(shell.program())
        .current_dir(&state.work_dir)
        .arg(shell.argument())
        .arg(command)
        .output()
    {
        Ok(output) => {
            let mut text = decode_text(&output.stdout);
            if output.stderr.len() > 0 {
                text.push_str(&decode_text(&output.stderr));
            }
            strip_ansi_escapes::strip_str(text)
        }
        Err(err) => {
            error!("Failed to execute command: {}", err);
            err.to_string()
        }
    }
}

fn decode_text(bytes: &[u8]) -> String {
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);
    encoding.decode(bytes).0.to_string()
}

// WebSocketUpgrade 用于将 HTTP 请求升级为 WebSocket 连接
pub async fn connect_socket(
    ws: WebSocketUpgrade,
    Path(shell): Path<String>,
    state: State<AppState>,
) -> Response {
    let work_dir = state.work_dir.clone();
    ws.on_upgrade(|socket| handle_socket(socket, shell, work_dir))
}

async fn handle_socket(socket: WebSocket, name: String, work_dir: std::path::PathBuf) {
    info!("Command shell: {}", &name);
    let mut state = AppState {
        shell: crate::shell::Shell::from_name(&name),
        work_dir,
    };
    let (mut sender, mut receiver) = socket.split();
    if let Some(version) = state.shell.version() {
        sender.send(Message::text(version)).await.ok();
    } else {
        error!("Failed to get shell version");
        return;
    }
    while let Some(Ok(msg)) = receiver.next().await {
        if let Ok(command) = msg.to_text() {
            info!("> {}", command);
            if command == "exit" {
                info!("Exit command shell");
                break;
            } else if let Some(path) = command.strip_prefix("cd ") {
                let mut work_dir = std::path::PathBuf::from(path.trim());
                if work_dir.is_relative() {
                    work_dir = state.work_dir.join(work_dir);
                }
                let message = if let Ok(path) = work_dir.normalize() {
                    state.work_dir = path.into_path_buf();
                    format!("▻ {}", state.work_dir.display())
                } else {
                    format!("Failed to resolve path: {}", work_dir.display())
                };
                sender.send(Message::text(message)).await.ok();
                continue;
            }
            let output = execute_command_inner(&state, command).await;
            info!("{}", &output);
            match sender.send(Message::text(output)).await {
                Ok(_) => {}
                Err(err) => {
                    error!("{}", err);
                    break;
                }
            }
        } else {
            error!("Received non-text message: {:?}", msg);
        }
    }
}
