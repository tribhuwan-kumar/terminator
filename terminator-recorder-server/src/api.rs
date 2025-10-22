use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use tracing::info;

use crate::recorder_manager::RecorderManager;
use crate::types::{
    HealthResponse, HighlightingStatus, StartRecordingRequest, StartRecordingResponse,
    StopRecordingRequest, StopRecordingResponse,
};

// ============================================================================
// Error Handling
// ============================================================================

pub struct ApiError(String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": self.0
            })),
        )
            .into_response()
    }
}

impl From<String> for ApiError {
    fn from(err: String) -> Self {
        ApiError(err)
    }
}

// ============================================================================
// Health Check
// ============================================================================

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

// ============================================================================
// Start Recording
// ============================================================================

pub async fn start_recording(
    State(manager): State<Arc<RecorderManager>>,
    Json(request): Json<StartRecordingRequest>,
) -> Result<Json<StartRecordingResponse>, ApiError> {
    info!("📥 POST /api/recording/start - workflow: {}", request.workflow_name);

    let highlighting_enabled = request.highlighting
        .as_ref()
        .map(|h| h.enabled)
        .unwrap_or(false);

    let (session_id, highlighting_started) = manager.start_recording(request).await?;

    let port = std::env::var("PORT").unwrap_or_else(|_| "8082".to_string());
    let websocket_url = format!("ws://127.0.0.1:{}/api/recording/events?session={}", port, session_id);

    let response = StartRecordingResponse {
        status: "started".to_string(),
        session_id: session_id.clone(),
        websocket_url,
        highlighting: if highlighting_enabled {
            Some(HighlightingStatus {
                enabled: true,
                task_started: highlighting_started,
            })
        } else {
            None
        },
    };

    info!("✅ Recording started: session={}, highlighting={}", session_id, highlighting_enabled);

    Ok(Json(response))
}

// ============================================================================
// Stop Recording
// ============================================================================

pub async fn stop_recording(
    State(manager): State<Arc<RecorderManager>>,
    Json(request): Json<StopRecordingRequest>,
) -> Result<Json<StopRecordingResponse>, ApiError> {
    info!("📥 POST /api/recording/stop - session: {}", request.session_id);

    let (session_id, events) = manager.stop_recording().await?;

    let response = StopRecordingResponse {
        status: "stopped".to_string(),
        session_id,
        event_count: events.len(),
    };

    info!("✅ Recording stopped: {} events captured", events.len());

    Ok(Json(response))
}

// ============================================================================
// Get Status
// ============================================================================

pub async fn get_status(
    State(manager): State<Arc<RecorderManager>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let status = if let Some((session_id, workflow_name)) = manager.get_session_info().await {
        serde_json::json!({
            "status": "recording",
            "session_id": session_id,
            "workflow_name": workflow_name
        })
    } else {
        serde_json::json!({
            "status": "idle"
        })
    };

    Ok(Json(status))
}
