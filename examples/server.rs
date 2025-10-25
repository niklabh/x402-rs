//! Example x402 server using Axum.
//!
//! This example demonstrates how to create a web server that requires payment
//! for accessing protected endpoints.
//!
//! Run with:
//! ```bash
//! cargo run --example server
//! ```
//!
//! Environment variables:
//! - PAY_TO: Address to receive payments
//! - FACILITATOR_URL: URL of the facilitator service
//! - PORT: Server port (default: 3000)

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use base64::Engine;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use x402_rs::server::{
    create_payment_required_response, verify_and_settle_payment, PaymentConfig,
};
use x402_rs::types::PaymentResponse;

#[derive(Clone)]
struct AppState {
    payment_config: PaymentConfig,
}

/// Protected endpoint that requires payment.
async fn weather_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    // Check for X-PAYMENT header
    if let Some(payment_header) = headers.get("X-PAYMENT") {
        let payment_str = payment_header
            .to_str()
            .map_err(|_| AppError::InvalidPayment("Invalid payment header encoding".into()))?;

        // Verify and settle the payment
        let tx_hash = verify_and_settle_payment(
            payment_str,
            &state.payment_config,
            "/weather",
        )
        .await
        .map_err(|e| AppError::PaymentFailed(e.to_string()))?;

        // Create payment response
        let payment_response = PaymentResponse {
            tx_hash: tx_hash.clone(),
            settled_at: Some(chrono::Utc::now().to_rfc3339()),
            metadata: None,
        };

        // Encode payment response as Base64 JSON
        let payment_response_json = serde_json::to_string(&payment_response)
            .map_err(|e| AppError::ServerError(e.to_string()))?;
        let payment_response_encoded = base64::engine::general_purpose::STANDARD
            .encode(payment_response_json.as_bytes());

        // Return the weather data with payment response header
        let weather_data = json!({
            "location": "San Francisco",
            "temperature": 68,
            "conditions": "Sunny",
            "humidity": 65,
            "paid": true,
            "tx_hash": tx_hash,
        });

        Ok((
            StatusCode::OK,
            [(
                axum::http::header::HeaderName::from_static("x-payment-response"),
                payment_response_encoded,
            )],
            Json(weather_data),
        )
            .into_response())
    } else {
        // No payment header, return 402 with payment requirements
        let mut configs = HashMap::new();
        configs.insert("usdc".to_string(), state.payment_config.clone());

        let payment_required = create_payment_required_response(&configs, "/weather")
            .map_err(|e| AppError::ServerError(e.to_string()))?;

        Ok((StatusCode::PAYMENT_REQUIRED, Json(payment_required)).into_response())
    }
}

/// Health check endpoint (no payment required).
async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "protocol": "x402",
        "version": 1,
    }))
}

/// Root endpoint with information.
async fn root_handler() -> impl IntoResponse {
    Json(json!({
        "message": "x402 Example Server",
        "endpoints": {
            "/weather": "Weather data (requires $0.01 payment)",
            "/health": "Health check (free)"
        },
        "protocol": "x402",
        "version": 1,
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration from environment
    let pay_to = std::env::var("PAY_TO")
        .unwrap_or_else(|_| "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string());
    let facilitator_url = std::env::var("FACILITATOR_URL")
        .unwrap_or_else(|_| "http://localhost:3001".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()?;

    println!("🚀 Starting x402 example server");
    println!("   Pay to: {}", pay_to);
    println!("   Facilitator: {}", facilitator_url);
    println!("   Port: {}", port);

    // Create payment configuration
    // Using Base mainnet USDC, $0.01 per request
    let payment_config = PaymentConfig::new(
        pay_to,
        "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", // USDC on Base
        6,
        "8453", // Base mainnet
        "exact",
        0.01, // $0.01
        "Weather API access",
        facilitator_url,
    )
    .with_timeout(300)
    .with_token_metadata("USD Coin", "2");

    let state = Arc::new(AppState { payment_config });

    // Build router
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/weather", get(weather_handler))
        .route("/health", get(health_handler))
        .with_state(state);

    // Start server
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("✅ Server listening on http://{}", addr);
    println!("\nTry:");
    println!("  curl http://localhost:{}/", port);
    println!("  curl http://localhost:{}/weather", port);
    println!();

    axum::serve(listener, app).await?;

    Ok(())
}

// Error handling
enum AppError {
    InvalidPayment(String),
    PaymentFailed(String),
    ServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::InvalidPayment(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::PaymentFailed(msg) => (StatusCode::PAYMENT_REQUIRED, msg),
            AppError::ServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

