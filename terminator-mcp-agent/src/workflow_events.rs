// This module uses terminator-workflow-recorder types directly where possible
// and only creates wrapper types where needed for MCP conversion

use serde::{Deserialize, Serialize};

// Re-export types from recorder that we use directly
pub use terminator_workflow_recorder::{
    ApplicationSwitchEvent,
    ApplicationSwitchMethod,
    BrowserClickEvent,
    BrowserTabNavigationEvent,
    BrowserTextInputEvent,
    ButtonInteractionType,
    ClickEvent,
    ClipboardAction,
    ClipboardEvent,
    DomElementInfo,
    DragDropEvent,
    EnhancedUIElement,
    // Metadata
    EventMetadata,
    FieldFocusMethod,
    HotkeyEvent,
    // Context types
    InteractionContext,
    // Event types
    KeyboardEvent,
    MouseButton,
    MouseEvent,
    MouseEventType,
    // Basic types
    Position,
    RecordedEvent,
    // Workflow types
    RecordedWorkflow,
    Rect,
    SelectionMethod,
    SelectorCandidate,
    TabAction,
    TabNavigationMethod,
    TextInputCompletedEvent,
    TextInputMethod,
    TextSelectionEvent,
    UIElementInfo,
    WorkflowEvent,
};

// Basic types are now re-exported from terminator_workflow_recorder above

// KeyboardEvent is now re-exported from terminator_workflow_recorder above

// MouseEvent is now re-exported from terminator_workflow_recorder above

// ClipboardAction and ClipboardEvent are now re-exported from terminator_workflow_recorder above

// TextSelectionEvent and SelectionMethod are now re-exported from terminator_workflow_recorder above

// DragDropEvent is now re-exported from terminator_workflow_recorder above

// HotkeyEvent is now re-exported from terminator_workflow_recorder above

// ButtonInteractionType is now re-exported from terminator_workflow_recorder above

// ClickEvent is now re-exported from terminator_workflow_recorder above

// WorkflowEvent is now re-exported from terminator_workflow_recorder above

/// Represents an MCP tool step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolStep {
    /// The name of the tool to call
    pub tool_name: String,

    /// The arguments to pass to the tool
    pub arguments: serde_json::Value,

    /// Optional description of what this step does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Timeout in milliseconds for this step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,

    /// Whether to continue execution if this step fails
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_on_error: Option<bool>,

    /// Delay in milliseconds after this step completes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_ms: Option<u64>,
}

// InteractionContext is now re-exported from terminator_workflow_recorder above

// UIElementInfo is now re-exported from terminator_workflow_recorder above

// EnhancedUIElement is now re-exported from terminator_workflow_recorder above

// RecordedEvent is now re-exported from terminator_workflow_recorder above

// RecordedWorkflow is now re-exported from terminator_workflow_recorder above

// TextInputMethod is now re-exported from terminator_workflow_recorder above

// TextInputCompletedEvent is now re-exported from terminator_workflow_recorder above

// ApplicationSwitchMethod is now re-exported from terminator_workflow_recorder above

// ApplicationSwitchEvent is now re-exported from terminator_workflow_recorder above

// TabAction is now re-exported from terminator_workflow_recorder above

// TabNavigationMethod is now re-exported from terminator_workflow_recorder above

// BrowserTabNavigationEvent is now re-exported from terminator_workflow_recorder above

// EventMetadata is now re-exported from terminator_workflow_recorder above

// No conversion needed - using recorder types directly
// All From implementations have been removed since we're using recorder types directly
