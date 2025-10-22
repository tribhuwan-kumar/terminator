mod api;
mod highlighting;
mod recorder_manager;
mod types;
mod websocket;

use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber;

use recorder_manager::RecorderManager;

#[derive(Parser, Debug)]
#[command(name = "terminator-recorder-server")]
#[command(about = "HTTP/WebSocket server for workflow recording with highlighting support")]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "8082")]
    port: u16,

    /// Enable CORS for all origins
    #[arg(long)]
    cors: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("🚀 Starting terminator-recorder-server v{}", env!("CARGO_PKG_VERSION"));
    info!("🔧 Port: {}", args.port);
    info!("🔧 CORS: {}", if args.cors { "enabled" } else { "disabled" });

    // Store port in environment for WebSocket URL generation
    std::env::set_var("PORT", args.port.to_string());

    // Initialize recorder manager
    let manager = Arc::new(
        RecorderManager::new()
            .expect("Failed to initialize RecorderManager")
    );

    info!("✅ RecorderManager initialized");

    // Build router
    let mut app = Router::new()
        // Health check
        .route("/api/health", get(api::health))

        // Recording endpoints
        .route("/api/recording/start", post(api::start_recording))
        .route("/api/recording/stop", post(api::stop_recording))
        .route("/api/recording/status", get(api::get_status))

        // WebSocket for event streaming
        .route("/api/recording/events", get(websocket::websocket_handler))

        // Shared state
        .with_state(manager);

    // Add CORS if enabled
    if args.cors {
        app = app.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
    }

    // Start server
    let addr = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("✅ Server listening on http://{}", addr);
    info!("📡 WebSocket endpoint: ws://{}/api/recording/events", addr);
    info!("🎬 Ready to record workflows!");

    axum::serve(listener, app).await?;

    Ok(())
}
