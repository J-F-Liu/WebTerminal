use axum::{
    Json,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Path, State},
    http::StatusCode,
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::process::Command;
use tracing::*;

#[derive(Clone)]
pub struct AppState {
    pub shell: crate::shell::Shell,
    pub work_dir: std::path::PathBuf,
}

pub async fn get_available() -> Result<Json<Vec<String>>, StatusCode> {
    let available = vec!["cmd".to_string(), "sh".to_string(), "nu".to_string()];
    Ok(Json(available))
}

pub async fn execute_command(state: State<AppState>, command: String) -> String {
    execute_command_inner(&state, &command).await
}

async fn execute_command_inner(state: &AppState, command: &str) -> String {
    let shell = &state.shell;
    match Command::new(shell.program)
        .current_dir(&state.work_dir)
        .arg(shell.argument)
        .arg(command)
        .output()
    {
        Ok(output) => {
            let mut text = shell.encoding.decode(&output.stdout).0.to_string();
            if output.stderr.len() > 0 {
                text.push_str(shell.encoding.decode(&output.stderr).0.as_ref());
            }
            text
        }
        Err(err) => {
            error!("Failed to execute command: {}", err);
            err.to_string()
        }
    }
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
    let mut state = AppState {
        shell: crate::shell::Shell::from_name(&name),
        work_dir,
    };
    let (mut sender, mut receiver) = socket.split();
    if let Some(version) = state.shell.get_version() {
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
                let work_dir = std::path::PathBuf::from(path.trim());
                if work_dir.is_absolute() {
                    state.work_dir = work_dir;
                } else {
                    state.work_dir = state.work_dir.join(work_dir);
                }
                let message = if state.work_dir.exists() {
                    "".to_string()
                } else {
                    format!("Directory does not exist: {}", state.work_dir.display())
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
