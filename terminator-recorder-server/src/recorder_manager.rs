use futures::StreamExt;
use std::sync::Arc;
use terminator::Desktop;
use terminator_workflow_recorder::{WorkflowEvent, WorkflowRecorder, WorkflowRecorderConfig};
use tokio::sync::{broadcast, Mutex};
use tracing::info;

use crate::highlighting::EventHighlighter;
use crate::types::StartRecordingRequest;

pub struct RecorderSession {
    pub session_id: String,
    pub workflow_name: String,
    pub recorder: WorkflowRecorder,
    pub highlighter: Option<EventHighlighter>,
    pub event_broadcaster: broadcast::Sender<WorkflowEvent>,
}

pub struct RecorderManager {
    desktop: Arc<Desktop>,
    current_session: Arc<Mutex<Option<RecorderSession>>>,
}

impl RecorderManager {
    pub fn new() -> Result<Self, String> {
        info!("🔧 Initializing RecorderManager");

        #[cfg(target_os = "windows")]
        let desktop = Desktop::new(false, false)
            .map_err(|e| format!("Failed to initialize Desktop: {}", e))?;

        #[cfg(target_os = "macos")]
        let desktop =
            Desktop::new(true, true).map_err(|e| format!("Failed to initialize Desktop: {}", e))?;

        info!("✅ Desktop initialized successfully");

        Ok(Self {
            desktop: Arc::new(desktop),
            current_session: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn start_recording(
        &self,
        request: StartRecordingRequest,
    ) -> Result<(String, bool), String> {
        let mut session_lock = self.current_session.lock().await;

        if session_lock.is_some() {
            return Err("Recording already in progress".to_string());
        }

        info!("🎬 Starting recording: {}", request.workflow_name);

        // Build recorder config
        let config = if let Some(config_opts) = &request.config {
            config_opts.to_workflow_recorder_config()
        } else {
            // Use default config
            WorkflowRecorderConfig {
                record_mouse: true,
                record_keyboard: true,
                record_text_input_completion: true,
                record_clipboard: true,
                record_hotkeys: true,
                record_application_switches: true,
                capture_ui_elements: true,
                enable_multithreading: true,
                filter_mouse_noise: true,
                ..Default::default()
            }
        };

        info!(
            "🔧 Recorder config: performance_mode={:?}",
            config.performance_mode
        );

        // Create recorder
        let mut recorder = WorkflowRecorder::new(request.workflow_name.clone(), config);

        // Start recorder
        recorder
            .start()
            .await
            .map_err(|e| format!("Failed to start recorder: {}", e))?;

        info!("✅ Recorder started successfully");

        // Create broadcast channel for WebSocket streaming
        let (event_tx, _) = broadcast::channel(100);

        // Setup highlighting if enabled
        let mut highlighter = None;
        let highlighting_enabled = request
            .highlighting
            .as_ref()
            .map(|h| h.enabled)
            .unwrap_or(false);

        if highlighting_enabled {
            let highlight_config = request.highlighting.unwrap_or_default();
            let mut event_highlighter = EventHighlighter::new(highlight_config);
            event_highlighter.start(&recorder);
            highlighter = Some(event_highlighter);
            info!("✅ Highlighting enabled");
        } else {
            info!("ℹ️  Highlighting disabled");
        }

        // Generate session ID
        let session_id = uuid::Uuid::new_v4().to_string();

        // Store session
        *session_lock = Some(RecorderSession {
            session_id: session_id.clone(),
            workflow_name: request.workflow_name,
            recorder,
            highlighter,
            event_broadcaster: event_tx,
        });

        info!("✅ Recording session created: {}", session_id);

        Ok((session_id, highlighting_enabled))
    }

    pub async fn stop_recording(&self) -> Result<(String, Vec<serde_json::Value>), String> {
        let mut session_lock = self.current_session.lock().await;

        let Some(mut session) = session_lock.take() else {
            return Err("No recording in progress".to_string());
        };

        info!("⏹️  Stopping recording session: {}", session.session_id);

        // Stop highlighting first
        if let Some(mut highlighter) = session.highlighter.take() {
            highlighter.stop().await;
        }

        // Stop recorder
        session
            .recorder
            .stop()
            .await
            .map_err(|e| format!("Failed to stop recorder: {}", e))?;

        info!("✅ Recorder stopped");

        // Get events as JSON (for API response)
        let events: Vec<serde_json::Value> = {
            let workflow = session.recorder.workflow.lock().unwrap();
            workflow
                .events
                .iter()
                .map(|e| serde_json::to_value(&e.event).unwrap_or(serde_json::Value::Null))
                .collect()
        };

        let event_count = events.len();
        info!("📊 Captured {} events", event_count);

        Ok((session.session_id, events))
    }

    pub async fn get_session_info(&self) -> Option<(String, String)> {
        let session_lock = self.current_session.lock().await;
        session_lock
            .as_ref()
            .map(|s| (s.session_id.clone(), s.workflow_name.clone()))
    }

    pub async fn get_event_broadcaster(&self) -> Option<broadcast::Sender<WorkflowEvent>> {
        let session_lock = self.current_session.lock().await;
        session_lock.as_ref().map(|s| s.event_broadcaster.clone())
    }

    /// Subscribe to recorder events for WebSocket streaming
    pub async fn subscribe_to_events(&self) -> Result<broadcast::Receiver<WorkflowEvent>, String> {
        let session_lock = self.current_session.lock().await;

        let Some(session) = session_lock.as_ref() else {
            return Err("No active recording session".to_string());
        };

        // Get event stream from recorder
        let mut event_stream = session.recorder.event_stream();
        let broadcaster = session.event_broadcaster.clone();

        // Spawn task to forward events from recorder to broadcaster
        tokio::spawn(async move {
            while let Some(event) = event_stream.next().await {
                // Broadcast to all WebSocket clients
                let _ = broadcaster.send(event);
            }
        });

        Ok(session.event_broadcaster.subscribe())
    }
}
