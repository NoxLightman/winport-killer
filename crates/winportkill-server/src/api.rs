use axum::{
    Json, Router,
    extract::{
        Path, Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use winportkill_core::{
    PortViewEntry, PortViewStats, ProcessViewEntry, ProcessViewStats, filter_ports,
    filter_processes, kill, port_stats, process_stats, scan_ports, scan_processes,
};

#[derive(Clone)]
pub struct AppState {
    tx: broadcast::Sender<String>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct VersionResponse {
    name: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
struct KillResult {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct PortsResponse {
    entries: Vec<PortViewEntry>,
    stats: PortViewStats,
}

#[derive(Serialize)]
struct ProcessesResponse {
    entries: Vec<ProcessViewEntry>,
    stats: ProcessViewStats,
}

#[derive(Deserialize)]
struct FilterQuery {
    filter: Option<String>,
}

pub fn create_app() -> Router<()> {
    let (tx, _) = broadcast::channel(100);
    let state = AppState { tx };

    Router::new()
        .route("/health", get(get_health))
        .route("/version", get(get_version))
        .route("/ports", get(get_ports))
        .route("/processes", get(get_processes))
        .route("/stats/ports", get(get_port_stats))
        .route("/stats/processes", get(get_process_stats))
        .route("/kill/{pid}", post(kill_process))
        .route("/ws", get(ws_handler))
        .route("/ports/filter/{keyword}", get(get_filtered_ports_legacy))
        .route("/stats", get(get_port_stats_legacy))
        .with_state(state)
}

async fn get_health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn get_version() -> Json<VersionResponse> {
    Json(VersionResponse {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn get_ports(Query(query): Query<FilterQuery>) -> Json<PortsResponse> {
    let entries = match query.filter {
        Some(filter) if !filter.is_empty() => filter_ports(&scan_ports(), &filter),
        _ => scan_ports(),
    };
    let stats = port_stats(&entries);
    Json(PortsResponse { entries, stats })
}

async fn get_processes(Query(query): Query<FilterQuery>) -> Json<ProcessesResponse> {
    let entries = match query.filter {
        Some(filter) if !filter.is_empty() => filter_processes(&scan_processes(), &filter),
        _ => scan_processes(),
    };
    let stats = process_stats(&entries);
    Json(ProcessesResponse { entries, stats })
}

async fn get_port_stats(Query(query): Query<FilterQuery>) -> Json<PortViewStats> {
    let entries = match query.filter {
        Some(filter) if !filter.is_empty() => filter_ports(&scan_ports(), &filter),
        _ => scan_ports(),
    };
    Json(port_stats(&entries))
}

async fn get_process_stats(Query(query): Query<FilterQuery>) -> Json<ProcessViewStats> {
    let entries = match query.filter {
        Some(filter) if !filter.is_empty() => filter_processes(&scan_processes(), &filter),
        _ => scan_processes(),
    };
    Json(process_stats(&entries))
}

async fn kill_process(Path(pid): Path<u32>, State(state): State<AppState>) -> Json<KillResult> {
    match kill(pid) {
        Ok(name) => {
            let msg = format!("Killed PID {} ({})", pid, name);
            let _ = state.tx.send(msg.clone());
            Json(KillResult {
                success: true,
                message: msg,
            })
        }
        Err(msg) => Json(KillResult {
            success: false,
            message: msg,
        }),
    }
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
    loop {
        interval.tick().await;
        let entries = scan_ports();
        let payload = serde_json::to_string(&PortsResponse {
            stats: port_stats(&entries),
            entries,
        })
        .unwrap_or_default();

        if socket.send(Message::Text(payload.into())).await.is_err() {
            break;
        }
    }
}

async fn get_filtered_ports_legacy(Path(keyword): Path<String>) -> Json<Vec<PortViewEntry>> {
    Json(filter_ports(&scan_ports(), &keyword))
}

async fn get_port_stats_legacy() -> Json<PortViewStats> {
    let entries = scan_ports();
    Json(port_stats(&entries))
}
