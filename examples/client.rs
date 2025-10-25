//! Example x402 client.
//!
//! This example demonstrates how to make requests to an x402-enabled server,
//! automatically handling payment requirements.
//!
//! Run with:
//! ```bash
//! cargo run --example client
//! ```
//!
//! Environment variables:
//! - PRIVATE_KEY: Your private key for signing payments
//! - RPC_URL: Blockchain RPC endpoint
//! - API_URL: The protected API endpoint to access

use base64::Engine;
use x402_rs::client::{get, X402ClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration from environment or use defaults
    let private_key = std::env::var("PRIVATE_KEY").unwrap_or_else(|_| {
        println!("⚠️  No PRIVATE_KEY set, using example key (DO NOT USE IN PRODUCTION)");
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
    });

    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://mainnet.base.org".to_string());

    let api_url = std::env::var("API_URL")
        .unwrap_or_else(|_| "http://localhost:3000/weather".to_string());

    println!("🔐 x402 Example Client");
    println!("   RPC: {}", rpc_url);
    println!("   API: {}", api_url);
    println!();

    // Create client configuration
    let config = X402ClientConfig::new(&private_key, &rpc_url)
        .with_scheme("exact")
        .with_network("8453"); // Base mainnet

    println!("📡 Making request to protected endpoint...");

    // Make the request - this will automatically handle 402 and payment
    match get(&config, &api_url).await {
        Ok(response) => {
            let status = response.status();
            println!("✅ Response status: {}", status);

            // Check for payment response header
            if let Some(payment_response) = response.headers().get("X-PAYMENT-RESPONSE") {
                if let Ok(encoded) = payment_response.to_str() {
                    if let Ok(decoded_bytes) = base64::engine::general_purpose::STANDARD
                        .decode(encoded.as_bytes())
                    {
                        if let Ok(json_str) = String::from_utf8(decoded_bytes) {
                            println!("💰 Payment settled: {}", json_str);
                        }
                    }
                }
            }

            // Parse and display response body
            if let Ok(body) = response.text().await {
                println!("\n📦 Response body:");
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    println!("{}", serde_json::to_string_pretty(&json)?);
                } else {
                    println!("{}", body);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            return Err(e.into());
        }
    }

    println!("\n✨ Done!");
    Ok(())
}

