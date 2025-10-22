use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::recorder_manager::RecorderManager;
use crate::types::WebSocketMessage;

#[derive(Deserialize)]
pub struct WsQuery {
    pub session: String,
}

/// WebSocket handler for streaming recording events
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsQuery>,
    State(manager): State<Arc<RecorderManager>>,
) -> Response {
    info!(
        "🔌 WebSocket connection request for session: {}",
        params.session
    );

    ws.on_upgrade(move |socket| handle_socket(socket, params.session, manager))
}

async fn handle_socket(socket: WebSocket, session_id: String, manager: Arc<RecorderManager>) {
    info!("✅ WebSocket connected: session={}", session_id);

    let (mut sender, mut receiver) = socket.split();

    // Subscribe to recorder events
    let mut event_receiver = match manager.subscribe_to_events().await {
        Ok(rx) => rx,
        Err(e) => {
            error!("❌ Failed to subscribe to events: {}", e);
            let error_msg = WebSocketMessage::Error {
                message: format!("Failed to subscribe: {}", e),
            };
            let _ = sender
                .send(Message::Text(
                    serde_json::to_string(&error_msg).unwrap().into(),
                ))
                .await;
            return;
        }
    };

    info!("📡 Subscribed to event stream for session: {}", session_id);

    // Forward events to WebSocket
    let mut event_count = 0;
    let mut last_status_update = tokio::time::Instant::now();

    loop {
        tokio::select! {
            // Receive event from recorder
            event_result = event_receiver.recv() => {
                    match event_result {
                        Ok(event) => {
                            event_count += 1;

                            // Serialize event to JSON
                            let event_json = match serde_json::to_value(&event) {
                                Ok(json) => json,
                                Err(e) => {
                                    warn!("⚠️ Failed to serialize event: {}", e);
                                    continue;
                                }
                            };

                            // Get timestamp
                            let timestamp = event.timestamp().unwrap_or_else(|| {
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis() as u64
                            });

                            // Create WebSocket message
                            let ws_msg = WebSocketMessage::Event {
                                session_id: session_id.clone(),
                                event: event_json,
                                timestamp,
                            };

                            // Send to WebSocket client
                            let msg_text = match serde_json::to_string(&ws_msg) {
                                Ok(text) => text,
                                Err(e) => {
                                    error!("❌ Failed to serialize WebSocket message: {}", e);
                                    continue;
                                }
                            };

                            if let Err(e) = sender.send(Message::Text(msg_text.into())).await {
                                warn!("❌ Failed to send WebSocket message: {}", e);
                                break;
                            }

                            // Send periodic status updates
                            if last_status_update.elapsed() >= tokio::time::Duration::from_secs(5) {
                                let status_msg = WebSocketMessage::Status {
                                    session_id: session_id.clone(),
                                    status: "recording".to_string(),
                                    event_count,
                                };

                                if let Ok(status_text) = serde_json::to_string(&status_msg) {
                                    let _ = sender.send(Message::Text(status_text.into())).await;
                                }

                                last_status_update = tokio::time::Instant::now();
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!("⚠️ WebSocket lagged, skipped {} events", skipped);
                        }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        info!("📡 Event channel closed, ending WebSocket");
                        break;
                    }
                }
            }

            // Handle incoming messages (ping/close)
            msg_result = receiver.next() => {
                match msg_result {
                    Some(Ok(Message::Text(text))) => {
                        if text.to_string() == "ping" {
                            let _ = sender.send(Message::Text("pong".into())).await;
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("🔌 WebSocket close received");
                        break;
                    }
                    Some(Err(e)) => {
                        warn!("❌ WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        info!("🔌 WebSocket stream ended");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    info!("📊 WebSocket forwarded {} events total", event_count);
    info!("🔌 WebSocket disconnected: session={}", session_id);
}
