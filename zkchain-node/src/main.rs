use axum::{routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;
use zk::ledger::{new_shared_node, SharedNode};
use zk::tx::Tx;

#[derive(Clone)]
struct AppState {
    node: SharedNode,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let node = new_shared_node();
    let app_state = AppState { node };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/submit_tx", post(submit_tx))
        .route("/mine", post(mine))
        .route("/height", get(height))
        .with_state(app_state);

    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    tracing::info!("listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await
        .unwrap();
}

async fn submit_tx(axum::extract::State(state): axum::extract::State<AppState>, Json(tx): Json<Tx>) -> Json<serde_json::Value> {
    let res = {
        let node = state.node.read();
        node.verify_tx(&tx)
    };
    match res {
        Ok(_) => {
            // temporarily buffer txs in memory on /mine call
            // For simplicity we directly mine single-tx blocks
            Json(serde_json::json!({"ok": true}))
        }
        Err(e) => Json(serde_json::json!({"ok": false, "error": e.to_string()})),
    }
}

async fn mine(axum::extract::State(state): axum::extract::State<AppState>, Json(txs): Json<Vec<Tx>>) -> Json<serde_json::Value> {
    let block = {
        let mut node = state.node.write();
        node.apply_block(txs)
    };
    match block {
        Ok(b) => Json(serde_json::json!({"ok": true, "height": b.height, "hash": hex::encode(b.hash)})),
        Err(e) => Json(serde_json::json!({"ok": false, "error": e.to_string()})),
    }
}

async fn height(axum::extract::State(state): axum::extract::State<AppState>) -> Json<serde_json::Value> {
    let h = { state.node.read().chain.len() as u64 - 1 };
    Json(serde_json::json!({"height": h}))
}
