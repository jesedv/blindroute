use std::sync::Arc;
use tokio::sync::RwLock;

use axum::{
    Router, routing::{get, post},
    extract::State,
    Json, response::Json as JsonResponse,
};
use tower_http::cors::CorsLayer;

use blindroute_core::scheme::{FheScheme, SchemeInfo};
use blindroute_ckks::{CkksScheme, CkksParams};

use crate::app::{BlindRouteServer, ComputeRequest, ComputeResponse};
use crate::keys::ServerConfig;

type SharedServer = Arc<RwLock<BlindRouteServer>>;

#[derive(serde::Serialize)]
struct PublicKeyResponse {
    p0: Vec<u64>,
    p1: Vec<u64>,
    n: usize,
    q: u64,
    scheme: String,
}

pub async fn serve(mut server: BlindRouteServer, config: ServerConfig) {
    server.generate_keys();
    let shared = Arc::new(RwLock::new(server));

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/info", get(info_handler))
        .route("/pubkey", get(pubkey_handler))
        .route("/compute", post(compute_handler))
        .layer(CorsLayer::permissive())
        .with_state(shared);

    let addr = format!("{}:{}", config.host, config.port);
    println!("BlindRoute server starting on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_handler() -> JsonResponse<serde_json::Value> {
    JsonResponse(serde_json::json!({ "status": "ok" }))
}

async fn info_handler(State(state): State<SharedServer>) -> JsonResponse<serde_json::Value> {
    let server = state.read().await;
    let info = <CkksScheme as FheScheme>::scheme_info(&CkksParams::from_scheme(&server.scheme));
    JsonResponse(serde_json::json!(info))
}

async fn pubkey_handler(State(state): State<SharedServer>) -> JsonResponse<serde_json::Value> {
    let server = state.read().await;
    match server.public_key() {
        Some(pk) => JsonResponse(serde_json::json!({
            "status": "ok",
            "p0": pk.p0,
            "p1": pk.p1,
            "n": pk.n,
            "q": pk.q,
        })),
        None => JsonResponse(serde_json::json!({
            "status": "error",
            "message": "keys not generated",
        })),
    }
}

async fn compute_handler(
    State(state): State<SharedServer>,
    Json(req): Json<ComputeRequest>,
) -> JsonResponse<ComputeResponse> {
    let server = state.read().await;
    let resp = server.compute(&req);
    JsonResponse(resp)
}
