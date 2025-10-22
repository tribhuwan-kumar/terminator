use serde::{Deserialize, Serialize};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct StartRecordingRequest {
    pub workflow_name: String,

    #[serde(default)]
    pub config: Option<RecorderConfigOptions>,

    #[serde(default)]
    pub highlighting: Option<HighlightingConfig>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartRecordingResponse {
    pub status: String,
    pub session_id: String,
    pub websocket_url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlighting: Option<HighlightingStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HighlightingStatus {
    pub enabled: bool,
    pub task_started: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StopRecordingRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StopRecordingResponse {
    pub status: String,
    pub session_id: String,
    pub event_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

// ============================================================================
// Recorder Configuration
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct RecorderConfigOptions {
    pub performance_mode: Option<String>, // "Normal", "Balanced", "LowEnergy"
    pub record_mouse: Option<bool>,
    pub record_keyboard: Option<bool>,
    pub record_text_input_completion: Option<bool>,
    pub record_clipboard: Option<bool>,
    pub record_hotkeys: Option<bool>,
}

impl RecorderConfigOptions {
    pub fn to_workflow_recorder_config(
        &self,
    ) -> terminator_workflow_recorder::WorkflowRecorderConfig {
        let performance_mode = match self.performance_mode.as_deref() {
            Some("LowEnergy") => terminator_workflow_recorder::PerformanceMode::LowEnergy,
            Some("Balanced") => terminator_workflow_recorder::PerformanceMode::Balanced,
            _ => terminator_workflow_recorder::PerformanceMode::Normal,
        };

        terminator_workflow_recorder::WorkflowRecorderConfig {
            record_mouse: self.record_mouse.unwrap_or(true),
            record_keyboard: self.record_keyboard.unwrap_or(true),
            capture_ui_elements: true,
            record_clipboard: self.record_clipboard.unwrap_or(true),
            record_hotkeys: self.record_hotkeys.unwrap_or(true),
            record_text_input_completion: self.record_text_input_completion.unwrap_or(true),
            record_application_switches: true,
            enable_multithreading: true,
            performance_mode,
            filter_mouse_noise: true,
            ..Default::default()
        }
    }
}

// ============================================================================
// Highlighting Configuration
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HighlightingConfig {
    pub enabled: bool,

    #[serde(default = "default_highlight_color")]
    pub color: Option<u32>, // BGR format

    #[serde(default = "default_highlight_duration")]
    pub duration_ms: Option<u64>,

    #[serde(default = "default_true")]
    pub show_labels: bool,

    pub label_position: Option<TextPosition>,
    pub label_style: Option<FontStyle>,
}

fn default_highlight_color() -> Option<u32> {
    Some(0x0000FF) // Red in BGR
}

fn default_highlight_duration() -> Option<u64> {
    Some(2000) // 2 seconds
}

fn default_true() -> bool {
    true
}

impl Default for HighlightingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            color: default_highlight_color(),
            duration_ms: default_highlight_duration(),
            show_labels: true,
            label_position: Some(TextPosition::Top),
            label_style: Some(FontStyle {
                size: 14,
                bold: true,
                color: 0xFFFFFF, // White
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum TextPosition {
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
    Inside,
}

#[cfg(target_os = "windows")]
impl From<&TextPosition> for terminator::platforms::windows::TextPosition {
    fn from(pos: &TextPosition) -> Self {
        use terminator::platforms::windows::TextPosition as WinPos;
        match pos {
            TextPosition::Top => WinPos::Top,
            TextPosition::TopRight => WinPos::TopRight,
            TextPosition::Right => WinPos::Right,
            TextPosition::BottomRight => WinPos::BottomRight,
            TextPosition::Bottom => WinPos::Bottom,
            TextPosition::BottomLeft => WinPos::BottomLeft,
            TextPosition::Left => WinPos::Left,
            TextPosition::TopLeft => WinPos::TopLeft,
            TextPosition::Inside => WinPos::Inside,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FontStyle {
    pub size: u32,
    pub bold: bool,
    pub color: u32, // BGR format
}

#[cfg(target_os = "windows")]
impl From<&FontStyle> for terminator::platforms::windows::FontStyle {
    fn from(style: &FontStyle) -> Self {
        terminator::platforms::windows::FontStyle {
            size: style.size,
            bold: style.bold,
            color: style.color,
        }
    }
}

// ============================================================================
// WebSocket Events
// ============================================================================

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "event")]
    Event {
        session_id: String,
        event: serde_json::Value,
        timestamp: u64,
    },

    #[serde(rename = "status")]
    Status {
        session_id: String,
        status: String,
        event_count: usize,
    },

    #[serde(rename = "error")]
    Error { message: String },
}
