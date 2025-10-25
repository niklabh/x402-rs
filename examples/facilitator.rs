//! Example x402 facilitator service.
//!
//! A facilitator is an intermediary service that verifies payment payloads
//! and settles transactions on-chain, paying the gas fees.
//!
//! Run with:
//! ```bash
//! cargo run --example facilitator
//! ```
//!
//! Environment variables:
//! - FACILITATOR_KEY: Private key for paying gas fees
//! - RPC_URL: Blockchain RPC endpoint
//! - PORT: Server port (default: 3001)

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;
use x402_rs::facilitator::{
    handle_settle, handle_supported, handle_verify, FacilitatorConfig,
};
use x402_rs::types::{SettlementRequest, VerificationRequest};

#[derive(Clone)]
struct AppState {
    config: FacilitatorConfig,
}

async fn verify_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<VerificationRequest>,
) -> impl IntoResponse {
    match handle_verify(request, &state.config).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn settle_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SettlementRequest>,
) -> impl IntoResponse {
    match handle_settle(request, &state.config).await {
        Ok(response) => {
            if response.error.is_some() {
                (StatusCode::BAD_REQUEST, Json(response)).into_response()
            } else {
                (StatusCode::OK, Json(response)).into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn supported_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match handle_supported(&state.config).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "service": "x402-facilitator",
        "version": 1,
    }))
}

async fn root_handler() -> impl IntoResponse {
    Json(json!({
        "service": "x402 Facilitator",
        "version": 1,
        "endpoints": {
            "/verify": "POST - Verify a payment payload",
            "/settle": "POST - Settle a payment on-chain",
            "/supported": "GET - List supported payment kinds",
            "/health": "GET - Health check"
        },
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration from environment
    let facilitator_key = std::env::var("FACILITATOR_KEY").unwrap_or_else(|_| {
        println!("⚠️  No FACILITATOR_KEY set, using example key (DO NOT USE IN PRODUCTION)");
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
    });

    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://mainnet.base.org".to_string());

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()?;

    println!("🔧 Starting x402 facilitator service");
    println!("   RPC: {}", rpc_url);
    println!("   Port: {}", port);

    // Create facilitator configuration
    let mut config = FacilitatorConfig::new(facilitator_key, rpc_url);
    
    // Add supported networks
    config.add_supported("exact", "8453"); // Base mainnet (already added by default)
    config.add_supported("exact", "84532"); // Base Sepolia
    config.add_supported("exact", "1"); // Ethereum mainnet
    config.add_supported("exact", "137"); // Polygon mainnet

    let state = Arc::new(AppState { config });

    // Build router
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/verify", post(verify_handler))
        .route("/settle", post(settle_handler))
        .route("/supported", get(supported_handler))
        .route("/health", get(health_handler))
        .with_state(state);

    // Start server
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("✅ Facilitator listening on http://{}", addr);
    println!("\nEndpoints:");
    println!("  POST   http://localhost:{}/verify", port);
    println!("  POST   http://localhost:{}/settle", port);
    println!("  GET    http://localhost:{}/supported", port);
    println!("  GET    http://localhost:{}/health", port);
    println!();

    axum::serve(listener, app).await?;

    Ok(())
}

