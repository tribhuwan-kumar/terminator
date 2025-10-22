use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use terminator::HighlightHandle;
use terminator_workflow_recorder::{WorkflowEvent, WorkflowRecorder};
use futures::StreamExt;
use tracing::{info, warn};

use crate::types::HighlightingConfig;

pub struct EventHighlighter {
    config: HighlightingConfig,
    active_highlights: Arc<Mutex<Vec<HighlightHandle>>>,
    task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl EventHighlighter {
    pub fn new(config: HighlightingConfig) -> Self {
        Self {
            config,
            active_highlights: Arc::new(Mutex::new(Vec::new())),
            task_handle: None,
        }
    }

    /// Start highlighting task that listens to recorder event stream
    pub fn start(&mut self, recorder: &WorkflowRecorder) {
        if !self.config.enabled {
            info!("🎨 Highlighting disabled by config");
            return;
        }

        let mut event_stream = recorder.event_stream();
        let config = self.config.clone();
        let active_highlights = self.active_highlights.clone();

        let task = tokio::spawn(async move {
            info!("🎨 Highlighting task started");

            while let Some(event) = event_stream.next().await {
                Self::highlight_event(
                    &event,
                    &config,
                    &active_highlights
                ).await;
            }

            info!("🎨 Highlighting task ended");
        });

        self.task_handle = Some(task);
        info!("🎨 Highlighting enabled with config: color={:X?}, duration={}ms, show_labels={}",
            self.config.color,
            self.config.duration_ms.unwrap_or(2000),
            self.config.show_labels
        );
    }

    /// Highlight a single event
    async fn highlight_event(
        event: &WorkflowEvent,
        config: &HighlightingConfig,
        active_highlights: &Arc<Mutex<Vec<HighlightHandle>>>,
    ) {
        // Extract UI element from event metadata
        let ui_element = match event {
            WorkflowEvent::Click(e) => e.metadata.ui_element.as_ref(),
            WorkflowEvent::TextInputCompleted(e) => e.metadata.ui_element.as_ref(),
            WorkflowEvent::Keyboard(e) => e.metadata.ui_element.as_ref(),
            WorkflowEvent::DragDrop(e) => e.metadata.ui_element.as_ref(),
            WorkflowEvent::ApplicationSwitch(e) => e.metadata.ui_element.as_ref(),
            WorkflowEvent::BrowserTabNavigation(e) => e.metadata.ui_element.as_ref(),
            WorkflowEvent::Mouse(e) => e.metadata.ui_element.as_ref(),
            _ => None,
        };

        let Some(ui_element) = ui_element else {
            return;
        };

        // Determine label text
        let label = if config.show_labels {
            Some(Self::event_to_label(event))
        } else {
            None
        };

        // Convert config types to platform types
        #[cfg(target_os = "windows")]
        let text_position = config.label_position.as_ref()
            .map(|pos| pos.into());
        #[cfg(not(target_os = "windows"))]
        let text_position: Option<()> = None;

        #[cfg(target_os = "windows")]
        let font_style = config.label_style.as_ref()
            .map(|style| style.into());
        #[cfg(not(target_os = "windows"))]
        let font_style: Option<()> = None;

        // Highlight the element
        match ui_element.highlight(
            config.color,
            config.duration_ms.map(Duration::from_millis),
            label.as_deref(),
            text_position,
            font_style,
        ) {
            Ok(handle) => {
                // Store handle
                {
                    let mut list = active_highlights.lock().await;
                    list.push(handle);
                }

                // Schedule cleanup
                let active_highlights_clone = active_highlights.clone();
                let duration = config.duration_ms.unwrap_or(2000);
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(duration)).await;
                    let mut list = active_highlights_clone.lock().await;
                    // Natural expiry: drop one handle (LIFO best-effort)
                    let _ = list.pop();
                });
            }
            Err(e) => {
                warn!("⚠️ Failed to highlight element: {}", e);
            }
        }
    }

    /// Convert event type to label text
    fn event_to_label(event: &WorkflowEvent) -> String {
        match event {
            WorkflowEvent::Click(_) => "CLICK".to_string(),
            WorkflowEvent::TextInputCompleted(_) => "TYPE".to_string(),
            WorkflowEvent::Keyboard(e) => format!("KEY: {}", e.key_code),
            WorkflowEvent::DragDrop(_) => "DRAG".to_string(),
            WorkflowEvent::ApplicationSwitch(_) => "SWITCH".to_string(),
            WorkflowEvent::BrowserTabNavigation(_) => "TAB".to_string(),
            WorkflowEvent::Mouse(e) => {
                use terminator_workflow_recorder::MouseButton;
                match e.button {
                    MouseButton::Right => "RCLICK".to_string(),
                    MouseButton::Middle => "MCLICK".to_string(),
                    _ => "MOUSE".to_string(),
                }
            }
            WorkflowEvent::BrowserClick(_) => "BROWSER CLICK".to_string(),
            WorkflowEvent::BrowserTextInput(_) => "BROWSER TYPE".to_string(),
            _ => "EVENT".to_string(),
        }
    }

    /// Stop highlighting and cleanup
    pub async fn stop(&mut self) {
        info!("🎨 Stopping highlighting task");

        // Abort highlighting task
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }

        // Clear all active highlights
        let mut list = self.active_highlights.lock().await;
        let count = list.len();
        while let Some(handle) = list.pop() {
            handle.close();
        }

        if count > 0 {
            info!("🎨 Cleared {} active highlights", count);
        }
    }
}
