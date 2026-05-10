use axum::{
    Json, Router,
    extract::{Path, State, WebSocketUpgrade, ws::{Message, WebSocket}},
    response::IntoResponse,
    routing::{get, post},
};
use serde::Serialize;
use winportkill_core::{scan, stats, kill, filter, Entry, Stats};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    tx: broadcast::Sender<String>,
}

#[derive(Serialize)]
struct KillResult {
    success: bool,
    message: String,
}

/// 创建 axum Router（Router<()>），可直接用于 axum::serve
pub fn create_app() -> Router<()> {
    let (tx, _) = broadcast::channel(100);
    let state = AppState { tx };
    Router::new()
        .route("/ports", get(get_ports))
        .route("/ports/filter/{keyword}", get(get_filtered_ports))
        .route("/kill/{pid}", post(kill_process))
        .route("/stats", get(get_stats))
        .route("/ws", get(ws_handler))
        .with_state(state)
}

async fn get_ports() -> Json<Vec<Entry>> {
    Json(scan())
}

async fn get_filtered_ports(Path(keyword): Path<String>) -> Json<Vec<Entry>> {
    let entries = scan();
    Json(filter(&entries, &keyword))
}

async fn kill_process(
    Path(pid): Path<u32>,
    State(state): State<AppState>,
) -> Json<KillResult> {
    match kill(pid) {
        Ok(name) => {
            let msg = format!("Killed PID {} ({})", pid, name);
            let _ = state.tx.send(msg.clone());
            Json(KillResult { success: true, message: msg })
        }
        Err(msg) => Json(KillResult { success: false, message: msg }),
    }
}

async fn get_stats() -> Json<Stats> {
    let entries = scan();
    Json(stats(&entries))
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
    loop {
        interval.tick().await;
        let entries = scan();
        let json = serde_json::to_string(&entries).unwrap_or_default();
        if socket.send(Message::Text(json.into())).await.is_err() {
            break;
        }
    }
}