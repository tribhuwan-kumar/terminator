use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::LazyLock;
use std::time::SystemTime;
use terminator::UIElement;

// Precomputed set of null-like values for efficient O(1) lookups
static NULL_LIKE_VALUES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        // Standard null representations
        "null",
        "nil",
        "undefined",
        "(null)",
        "<null>",
        "n/a",
        "na",
        "",
        // Windows-specific null patterns
        "unknown",
        "<unknown>",
        "(unknown)",
        "none",
        "<none>",
        "(none)",
        "empty",
        "<empty>",
        "(empty)",
        // COM/Windows API specific
        "bstr()",
        "variant()",
        "variant(empty)",
    ]
    .into_iter()
    .collect()
});

// Helper function to filter empty strings and null-like values for serde skip_serializing_if
fn is_empty_string(s: &Option<String>) -> bool {
    match s {
        Some(s) => {
            // Fast path for completely empty strings
            if s.is_empty() {
                return true;
            }

            // Fast path for whitespace-only strings
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return true;
            }

            // Check against precomputed set (case-insensitive)
            // Only allocate lowercase string if we have a reasonable candidate
            if trimmed.len() <= 20 {
                // Reasonable max length for null-like values
                let lower = trimmed.to_lowercase();
                NULL_LIKE_VALUES.contains(lower.as_str())
            } else {
                false // Long strings are unlikely to be null-like values
            }
        }
        None => true,
    }
}

/// Represents a position on the screen
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Represents a rectangle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Represents the type of mouse button
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Represents the type of mouse event
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MouseEventType {
    Click,
    DoubleClick,
    RightClick,
    Down,
    Up,
    Move,
    Wheel,
    DragStart,
    DragEnd,
    Drop,
}

/// Represents a keyboard event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardEvent {
    /// The key code
    pub key_code: u32,

    /// Whether the key was pressed or released
    pub is_key_down: bool,

    /// Whether the Ctrl key was pressed
    pub ctrl_pressed: bool,

    /// Whether the Alt key was pressed
    pub alt_pressed: bool,

    /// Whether the Shift key was pressed
    pub shift_pressed: bool,

    /// Whether the Win key was pressed
    pub win_pressed: bool,

    /// Character representation of the key (if printable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<char>,

    /// Raw scan code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_code: Option<u32>,

    /// Event metadata (UI element, application, etc.)
    pub metadata: EventMetadata,
}

/// Represents a mouse event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEvent {
    /// The type of mouse event
    pub event_type: MouseEventType,

    /// The mouse button
    pub button: MouseButton,

    /// The position of the mouse
    pub position: Position,

    /// Scroll delta for wheel events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_delta: Option<(i32, i32)>,

    /// Drag start position (for drag events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drag_start: Option<Position>,

    /// Event metadata (UI element, application, etc.)
    pub metadata: EventMetadata,
}

/// Represents clipboard actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClipboardAction {
    Copy,
    Cut,
    Paste,
    Clear,
}

/// Represents a clipboard event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEvent {
    /// The clipboard action
    pub action: ClipboardAction,

    /// The content that was copied/cut/pasted (truncated if too long)
    #[serde(skip_serializing_if = "is_empty_string")]
    pub content: Option<String>,

    /// The size of the content in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_size: Option<usize>,

    /// The format of the clipboard data
    #[serde(skip_serializing_if = "is_empty_string")]
    pub format: Option<String>,

    /// Whether the content was truncated due to size
    pub truncated: bool,

    /// Event metadata (UI element, application, etc.)
    pub metadata: EventMetadata,
}

/// Represents text selection events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSelectionEvent {
    /// The selected text content
    pub selected_text: String,

    /// The start position of the selection (screen coordinates)
    pub start_position: Position,

    /// The end position of the selection (screen coordinates)
    pub end_position: Position,

    /// The selection method (mouse drag, keyboard shortcuts, etc.)
    pub selection_method: SelectionMethod,

    /// The length of the selection in characters
    pub selection_length: usize,

    /// Event metadata (UI element, application, etc.)
    pub metadata: EventMetadata,
}

/// Represents how text was selected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionMethod {
    MouseDrag,
    DoubleClick,      // Word selection
    TripleClick,      // Line/paragraph selection
    KeyboardShortcut, // Ctrl+A, Shift+arrows, etc.
    ContextMenu,
}

/// Represents drag and drop operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragDropEvent {
    /// The start position of the drag
    pub start_position: Position,

    /// The end position of the drop
    pub end_position: Position,

    /// The UI element being dragged (source)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_element: Option<UIElement>,

    /// The type of data being dragged
    #[serde(skip_serializing_if = "is_empty_string")]
    pub data_type: Option<String>,

    /// The dragged content (if text)
    #[serde(skip_serializing_if = "is_empty_string")]
    pub content: Option<String>,

    /// Whether the drag was successful
    pub success: bool,

    /// Event metadata (target UI element, application, etc.)
    pub metadata: EventMetadata,
}

/// Represents hotkey/shortcut events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyEvent {
    /// The key combination (e.g., "Ctrl+C", "Alt+Tab")
    pub combination: String,

    /// The action performed by the hotkey
    #[serde(skip_serializing_if = "is_empty_string")]
    pub action: Option<String>,

    /// Whether this was a global or application-specific hotkey
    pub is_global: bool,

    /// Event metadata (UI element, application, etc.)
    pub metadata: EventMetadata,
}

/// Represents the type of button interaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ButtonInteractionType {
    /// Simple button click
    Click,
    /// Toggle button (on/off)
    Toggle,
    /// Dropdown button expand/collapse
    DropdownToggle,
    /// Submit button
    Submit,
    /// Cancel/close button
    Cancel,
}

/// Represents a high-level button click event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonClickEvent {
    /// The text/label of the button
    pub button_text: String,

    /// The type of button interaction
    pub interaction_type: ButtonInteractionType,

    /// The role/type of the button element
    pub button_role: String,

    /// Whether the button was enabled when clicked
    pub was_enabled: bool,

    /// The position where the button was clicked
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_position: Option<Position>,

    /// Additional context about the button's function
    #[serde(skip_serializing_if = "is_empty_string")]
    pub button_description: Option<String>,

    /// Event metadata with UI element context
    pub metadata: EventMetadata,
}

/// Represents a workflow event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEvent {
    /// A mouse event
    Mouse(MouseEvent),

    /// A keyboard event
    Keyboard(KeyboardEvent),

    /// A clipboard event
    Clipboard(ClipboardEvent),

    /// A text selection event
    TextSelection(TextSelectionEvent),

    /// A drag and drop event
    DragDrop(DragDropEvent),

    /// A hotkey event
    Hotkey(HotkeyEvent),

    /// High-level text input completion event
    TextInputCompleted(TextInputCompletedEvent),

    /// High-level application switch event
    ApplicationSwitch(ApplicationSwitchEvent),

    /// High-level browser tab navigation event
    BrowserTabNavigation(BrowserTabNavigationEvent),

    /// High-level button click event
    ButtonClick(ButtonClickEvent),
}

impl WorkflowEvent {
    /// Returns a reference to the metadata of the event.
    pub fn metadata(&self) -> &EventMetadata {
        match self {
            WorkflowEvent::Mouse(e) => &e.metadata,
            WorkflowEvent::Keyboard(e) => &e.metadata,
            WorkflowEvent::Clipboard(e) => &e.metadata,
            WorkflowEvent::TextSelection(e) => &e.metadata,
            WorkflowEvent::DragDrop(e) => &e.metadata,
            WorkflowEvent::Hotkey(e) => &e.metadata,
            WorkflowEvent::TextInputCompleted(e) => &e.metadata,
            WorkflowEvent::ApplicationSwitch(e) => &e.metadata,
            WorkflowEvent::BrowserTabNavigation(e) => &e.metadata,
            WorkflowEvent::ButtonClick(e) => &e.metadata,
        }
    }

    /// Returns a mutable reference to the metadata of the event.
    pub fn metadata_mut(&mut self) -> &mut EventMetadata {
        match self {
            WorkflowEvent::Mouse(e) => &mut e.metadata,
            WorkflowEvent::Keyboard(e) => &mut e.metadata,
            WorkflowEvent::Clipboard(e) => &mut e.metadata,
            WorkflowEvent::TextSelection(e) => &mut e.metadata,
            WorkflowEvent::DragDrop(e) => &mut e.metadata,
            WorkflowEvent::Hotkey(e) => &mut e.metadata,
            WorkflowEvent::TextInputCompleted(e) => &mut e.metadata,
            WorkflowEvent::ApplicationSwitch(e) => &mut e.metadata,
            WorkflowEvent::BrowserTabNavigation(e) => &mut e.metadata,
            WorkflowEvent::ButtonClick(e) => &mut e.metadata,
        }
    }

    /// Returns the timestamp of the event, if available.
    pub fn timestamp(&self) -> Option<u64> {
        self.metadata().timestamp
    }

    /// Returns a reference to the UI element of the event, if available.
    pub fn ui_element(&self) -> Option<&UIElement> {
        self.metadata().ui_element.as_ref()
    }
}

/// Represents a recorded event with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedEvent {
    /// The timestamp of the event (milliseconds since epoch)
    pub timestamp: u64,

    /// The event
    pub event: WorkflowEvent,
}

/// Represents a recorded workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedWorkflow {
    /// The name of the workflow
    pub name: String,

    /// The timestamp when the recording started
    pub start_time: u64,

    /// The timestamp when the recording ended
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u64>,

    /// The recorded events
    pub events: Vec<RecordedEvent>,
}

impl RecordedWorkflow {
    /// Create a new recorded workflow
    pub fn new(name: String) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            name,
            start_time: now,
            end_time: None,
            events: Vec::new(),
        }
    }

    /// Add an event to the workflow
    pub fn add_event(&mut self, event: WorkflowEvent) {
        // Use the event's timestamp if available in its metadata, otherwise generate current timestamp
        let timestamp = event.timestamp().unwrap_or_else(|| {
            // Fallback: generate timestamp now if not present in event metadata
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        });

        self.events.push(RecordedEvent { timestamp, event });
    }

    /// Finish the recording
    pub fn finish(&mut self) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        self.end_time = Some(now);
    }

    /// Serialize the workflow to JSON string
    /// This converts UIElement instances to serializable form
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let serializable: SerializableRecordedWorkflow = self.into();
        serde_json::to_string_pretty(&serializable)
    }

    /// Serialize the workflow to JSON bytes
    /// This converts UIElement instances to serializable form
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let serializable: SerializableRecordedWorkflow = self.into();
        serde_json::to_vec_pretty(&serializable)
    }

    /// Deserialize a workflow from JSON string
    /// Note: This creates a workflow with serializable UI elements,
    /// not the original UIElement instances
    pub fn from_json(json: &str) -> Result<SerializableRecordedWorkflow, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Deserialize a workflow from JSON bytes
    /// Note: This creates a workflow with serializable UI elements,
    /// not the original UIElement instances
    pub fn from_json_bytes(
        bytes: &[u8],
    ) -> Result<SerializableRecordedWorkflow, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Save the workflow to a JSON file
    pub fn save_to_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load a workflow from a JSON file
    /// Note: This creates a workflow with serializable UI elements,
    /// not the original UIElement instances
    pub fn load_from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<SerializableRecordedWorkflow, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let workflow = Self::from_json(&json)?;
        Ok(workflow)
    }
}

/// Method used to input text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TextInputMethod {
    /// Text was typed character by character
    Typed,
    /// Text was likely pasted (large amount added quickly)
    Pasted,
    /// Text was likely auto-filled or auto-completed
    AutoFilled,
    /// Text was selected from autocomplete/suggestion dropdown
    Suggestion,
    /// Mixed input methods
    Mixed,
}

/// High-level text input completion event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextInputCompletedEvent {
    /// The text that was entered in the field
    pub text_value: String,
    /// The name/label of the input field
    #[serde(skip_serializing_if = "is_empty_string")]
    pub field_name: Option<String>,
    /// The type of input field (e.g., "TextBox", "PasswordBox", "SearchBox")
    pub field_type: String,
    /// Whether the text was likely typed vs pasted/auto-filled
    pub input_method: TextInputMethod,
    /// Duration of the typing session in milliseconds
    pub typing_duration_ms: u64,
    /// Number of individual keystroke events that contributed to this input
    pub keystroke_count: u32,
    /// Event metadata with UI element context
    pub metadata: EventMetadata,
}

/// Method used to switch applications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApplicationSwitchMethod {
    /// Alt+Tab keyboard shortcut
    AltTab,
    /// Clicking on taskbar icon
    TaskbarClick,
    /// Windows key + number shortcut
    WindowsKeyShortcut,
    /// Start menu or app launcher
    StartMenu,
    /// Direct window click
    WindowClick,
    /// Other/unknown method
    Other,
}

/// High-level application switch event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationSwitchEvent {
    /// The application being switched from
    #[serde(skip_serializing_if = "is_empty_string")]
    pub from_application: Option<String>,
    /// The application being switched to
    pub to_application: String,
    /// Process ID of the source application
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_process_id: Option<u32>,
    /// Process ID of the target application
    pub to_process_id: u32,
    /// Method used to switch applications
    pub switch_method: ApplicationSwitchMethod,
    /// Time spent in the previous application (milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dwell_time_ms: Option<u64>,
    /// Number of rapid application switches (Alt+Tab cycling)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switch_count: Option<u32>,
    /// Event metadata
    pub metadata: EventMetadata,
}

/// Browser tab navigation action type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TabAction {
    /// New tab created
    Created,
    /// Switched to existing tab
    Switched,
    /// Tab closed
    Closed,
    /// Tab moved/reordered
    Moved,
    /// Tab duplicated
    Duplicated,
    /// Tab pinned/unpinned
    Pinned,
    /// Tab refreshed/reloaded
    Refreshed,
}

/// Method used for tab navigation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TabNavigationMethod {
    /// Keyboard shortcut (Ctrl+T, Ctrl+W, Ctrl+Tab, etc.)
    KeyboardShortcut,
    /// Mouse click on tab
    TabClick,
    /// Mouse click on new tab button
    NewTabButton,
    /// Mouse click on close button
    CloseButton,
    /// Context menu action
    ContextMenu,
    /// Address bar navigation
    AddressBar,
    /// Link click that opens in new tab
    LinkNewTab,
    /// Other/unknown method
    Other,
}

/// High-level browser tab navigation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserTabNavigationEvent {
    /// The action performed on the tab
    pub action: TabAction,
    /// Method used for the navigation
    pub method: TabNavigationMethod,
    /// URL being navigated TO (destination URL)
    #[serde(skip_serializing_if = "is_empty_string")]
    pub to_url: Option<String>,
    /// URL being navigated FROM (source URL)
    #[serde(skip_serializing_if = "is_empty_string")]
    pub from_url: Option<String>,
    /// Title of the page being navigated TO (destination title)
    #[serde(skip_serializing_if = "is_empty_string")]
    pub to_title: Option<String>,
    /// Title of the page being navigated FROM (source title)
    #[serde(skip_serializing_if = "is_empty_string")]
    pub from_title: Option<String>,
    /// Browser application (Chrome, Firefox, Edge, etc.)
    pub browser: String,
    /// Current tab index in the window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_index: Option<u32>,
    /// Total number of tabs in the window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tabs: Option<u32>,
    /// Time spent on previous URL (for navigation events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_dwell_time_ms: Option<u64>,
    /// Whether this was a back/forward navigation
    pub is_back_forward: bool,
    /// Event metadata
    pub metadata: EventMetadata,
}

/// Unified metadata for all workflow events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// The UI element associated with this event (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_element: Option<UIElement>,

    /// The exact timestamp when this event occurred (milliseconds since epoch)
    /// If None, the timestamp will be generated when the event is recorded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

// implement empty() constructor
impl EventMetadata {
    pub fn empty() -> Self {
        Self {
            ui_element: None,
            timestamp: None,
        }
    }

    /// Create EventMetadata with current timestamp
    pub fn with_timestamp() -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            ui_element: None,
            timestamp: Some(now),
        }
    }

    /// Create EventMetadata with UI element and current timestamp
    pub fn with_ui_element_and_timestamp(ui_element: Option<UIElement>) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            ui_element,
            timestamp: Some(now),
        }
    }
}

/// Serializable version of UIElement for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableUIElement {
    #[serde(skip_serializing_if = "is_empty_string")]
    pub id: Option<String>,
    pub role: String,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<(f64, f64, f64, f64)>,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub application: Option<String>,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub window_title: Option<String>,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_id: Option<u32>,
}

impl From<&UIElement> for SerializableUIElement {
    fn from(element: &UIElement) -> Self {
        let attrs = element.attributes();
        let bounds = element.bounds().ok();

        // Helper function to filter empty strings
        fn filter_empty(s: Option<String>) -> Option<String> {
            s.filter(|s| !s.is_empty())
        }

        Self {
            id: filter_empty(element.id()),
            role: element.role(),
            name: filter_empty(attrs.name),
            bounds,
            value: filter_empty(attrs.value),
            description: filter_empty(attrs.description),
            application: filter_empty(Some(element.application_name())),
            window_title: filter_empty(Some(element.window_title())),
            url: element.url(),
            process_id: element.process_id().ok(),
        }
    }
}

/// Serializable version of EventMetadata for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableEventMetadata {
    /// The UI element associated with this event (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_element: Option<SerializableUIElement>,

    /// The exact timestamp when this event occurred (milliseconds since epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

impl From<&EventMetadata> for SerializableEventMetadata {
    fn from(metadata: &EventMetadata) -> Self {
        Self {
            ui_element: metadata.ui_element.as_ref().map(|elem| elem.into()),
            timestamp: metadata.timestamp,
        }
    }
}

/// Serializable version of KeyboardEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableKeyboardEvent {
    pub key_code: u32,
    pub is_key_down: bool,
    pub ctrl_pressed: bool,
    pub alt_pressed: bool,
    pub shift_pressed: bool,
    pub win_pressed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<char>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_code: Option<u32>,
    pub metadata: SerializableEventMetadata,
}

impl From<&KeyboardEvent> for SerializableKeyboardEvent {
    fn from(event: &KeyboardEvent) -> Self {
        Self {
            key_code: event.key_code,
            is_key_down: event.is_key_down,
            ctrl_pressed: event.ctrl_pressed,
            alt_pressed: event.alt_pressed,
            shift_pressed: event.shift_pressed,
            win_pressed: event.win_pressed,
            character: event.character,
            scan_code: event.scan_code,
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of MouseEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableMouseEvent {
    pub event_type: MouseEventType,
    pub button: MouseButton,
    pub position: Position,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_delta: Option<(i32, i32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drag_start: Option<Position>,
    pub metadata: SerializableEventMetadata,
}

impl From<&MouseEvent> for SerializableMouseEvent {
    fn from(event: &MouseEvent) -> Self {
        Self {
            event_type: event.event_type,
            button: event.button,
            position: event.position,
            scroll_delta: event.scroll_delta,
            drag_start: event.drag_start,
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of ClipboardEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableClipboardEvent {
    pub action: ClipboardAction,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_size: Option<usize>,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub format: Option<String>,
    pub truncated: bool,
    pub metadata: SerializableEventMetadata,
}

impl From<&ClipboardEvent> for SerializableClipboardEvent {
    fn from(event: &ClipboardEvent) -> Self {
        Self {
            action: event.action.clone(),
            content: event.content.clone(),
            content_size: event.content_size,
            format: event.format.clone(),
            truncated: event.truncated,
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of TextSelectionEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTextSelectionEvent {
    pub selected_text: String,
    pub start_position: Position,
    pub end_position: Position,
    pub selection_method: SelectionMethod,
    pub selection_length: usize,
    pub metadata: SerializableEventMetadata,
}

impl From<&TextSelectionEvent> for SerializableTextSelectionEvent {
    fn from(event: &TextSelectionEvent) -> Self {
        Self {
            selected_text: event.selected_text.clone(),
            start_position: event.start_position,
            end_position: event.end_position,
            selection_method: event.selection_method.clone(),
            selection_length: event.selection_length,
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of DragDropEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableDragDropEvent {
    pub start_position: Position,
    pub end_position: Position,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_element: Option<SerializableUIElement>,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub data_type: Option<String>,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub content: Option<String>,
    pub success: bool,
    pub metadata: SerializableEventMetadata,
}

impl From<&DragDropEvent> for SerializableDragDropEvent {
    fn from(event: &DragDropEvent) -> Self {
        Self {
            start_position: event.start_position,
            end_position: event.end_position,
            source_element: event.source_element.as_ref().map(|elem| elem.into()),
            data_type: event.data_type.clone(),
            content: event.content.clone(),
            success: event.success,
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of HotkeyEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableHotkeyEvent {
    pub combination: String,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub action: Option<String>,
    pub is_global: bool,
    pub metadata: SerializableEventMetadata,
}

impl From<&HotkeyEvent> for SerializableHotkeyEvent {
    fn from(event: &HotkeyEvent) -> Self {
        Self {
            combination: event.combination.clone(),
            action: event.action.clone(),
            is_global: event.is_global,
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of TextInputCompletedEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTextInputCompletedEvent {
    pub text_value: String,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub field_name: Option<String>,
    pub field_type: String,
    pub input_method: TextInputMethod,
    pub typing_duration_ms: u64,
    pub keystroke_count: u32,
    pub metadata: SerializableEventMetadata,
}

impl From<&TextInputCompletedEvent> for SerializableTextInputCompletedEvent {
    fn from(event: &TextInputCompletedEvent) -> Self {
        Self {
            text_value: event.text_value.clone(),
            field_name: event.field_name.clone(),
            field_type: event.field_type.clone(),
            input_method: event.input_method.clone(),
            typing_duration_ms: event.typing_duration_ms,
            keystroke_count: event.keystroke_count,
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of ApplicationSwitchEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableApplicationSwitchEvent {
    #[serde(skip_serializing_if = "is_empty_string")]
    pub from_application: Option<String>,
    pub to_application: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_process_id: Option<u32>,
    pub to_process_id: u32,
    pub switch_method: ApplicationSwitchMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dwell_time_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switch_count: Option<u32>,
    pub metadata: SerializableEventMetadata,
}

impl From<&ApplicationSwitchEvent> for SerializableApplicationSwitchEvent {
    fn from(event: &ApplicationSwitchEvent) -> Self {
        Self {
            from_application: event.from_application.clone(),
            to_application: event.to_application.clone(),
            from_process_id: event.from_process_id,
            to_process_id: event.to_process_id,
            switch_method: event.switch_method.clone(),
            dwell_time_ms: event.dwell_time_ms,
            switch_count: event.switch_count,
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of BrowserTabNavigationEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableBrowserTabNavigationEvent {
    pub action: TabAction,
    pub method: TabNavigationMethod,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub to_url: Option<String>,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub from_url: Option<String>,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub to_title: Option<String>,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub from_title: Option<String>,
    pub browser: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tabs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_dwell_time_ms: Option<u64>,
    pub is_back_forward: bool,
    pub metadata: SerializableEventMetadata,
}

impl From<&BrowserTabNavigationEvent> for SerializableBrowserTabNavigationEvent {
    fn from(event: &BrowserTabNavigationEvent) -> Self {
        Self {
            action: event.action.clone(),
            method: event.method.clone(),
            to_url: event.to_url.clone(),
            from_url: event.from_url.clone(),
            to_title: event.to_title.clone(),
            from_title: event.from_title.clone(),
            browser: event.browser.clone(),
            tab_index: event.tab_index,
            total_tabs: event.total_tabs,
            page_dwell_time_ms: event.page_dwell_time_ms,
            is_back_forward: event.is_back_forward,
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of ButtonClickEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableButtonClickEvent {
    pub button_text: String,
    pub interaction_type: ButtonInteractionType,
    pub button_role: String,
    pub was_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_position: Option<Position>,
    #[serde(skip_serializing_if = "is_empty_string")]
    pub button_description: Option<String>,
    pub metadata: SerializableEventMetadata,
}

impl From<&ButtonClickEvent> for SerializableButtonClickEvent {
    fn from(event: &ButtonClickEvent) -> Self {
        Self {
            button_text: event.button_text.clone(),
            interaction_type: event.interaction_type.clone(),
            button_role: event.button_role.clone(),
            was_enabled: event.was_enabled,
            click_position: event.click_position,
            button_description: event.button_description.clone(),
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of WorkflowEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableWorkflowEvent {
    Mouse(SerializableMouseEvent),
    Keyboard(SerializableKeyboardEvent),
    Clipboard(SerializableClipboardEvent),
    TextSelection(SerializableTextSelectionEvent),
    DragDrop(SerializableDragDropEvent),
    Hotkey(SerializableHotkeyEvent),
    TextInputCompleted(SerializableTextInputCompletedEvent),
    ApplicationSwitch(SerializableApplicationSwitchEvent),
    BrowserTabNavigation(SerializableBrowserTabNavigationEvent),
    ButtonClick(SerializableButtonClickEvent),
}

impl From<&WorkflowEvent> for SerializableWorkflowEvent {
    fn from(event: &WorkflowEvent) -> Self {
        match event {
            WorkflowEvent::Mouse(e) => SerializableWorkflowEvent::Mouse(e.into()),
            WorkflowEvent::Keyboard(e) => SerializableWorkflowEvent::Keyboard(e.into()),
            WorkflowEvent::Clipboard(e) => SerializableWorkflowEvent::Clipboard(e.into()),
            WorkflowEvent::TextSelection(e) => SerializableWorkflowEvent::TextSelection(e.into()),
            WorkflowEvent::DragDrop(e) => SerializableWorkflowEvent::DragDrop(e.into()),
            WorkflowEvent::Hotkey(e) => SerializableWorkflowEvent::Hotkey(e.into()),
            WorkflowEvent::TextInputCompleted(e) => {
                SerializableWorkflowEvent::TextInputCompleted(e.into())
            }
            WorkflowEvent::ApplicationSwitch(e) => {
                SerializableWorkflowEvent::ApplicationSwitch(e.into())
            }
            WorkflowEvent::BrowserTabNavigation(e) => {
                SerializableWorkflowEvent::BrowserTabNavigation(e.into())
            }
            WorkflowEvent::ButtonClick(e) => SerializableWorkflowEvent::ButtonClick(e.into()),
        }
    }
}

/// Serializable version of RecordedEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableRecordedEvent {
    pub timestamp: u64,
    pub event: SerializableWorkflowEvent,
}

impl From<&RecordedEvent> for SerializableRecordedEvent {
    fn from(event: &RecordedEvent) -> Self {
        Self {
            timestamp: event.timestamp,
            event: (&event.event).into(),
        }
    }
}

/// Serializable version of RecordedWorkflow for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableRecordedWorkflow {
    pub name: String,
    pub start_time: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u64>,
    pub events: Vec<SerializableRecordedEvent>,
}

impl From<&RecordedWorkflow> for SerializableRecordedWorkflow {
    fn from(workflow: &RecordedWorkflow) -> Self {
        Self {
            name: workflow.name.clone(),
            start_time: workflow.start_time,
            end_time: workflow.end_time,
            events: workflow.events.iter().map(|e| e.into()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string_helper() {
        // Test None values
        assert!(is_empty_string(&None));

        // Test empty strings
        assert!(is_empty_string(&Some("".to_string())));
        assert!(is_empty_string(&Some(" ".to_string())));
        assert!(is_empty_string(&Some("   ".to_string())));
        assert!(is_empty_string(&Some("\t\n".to_string())));

        // Test various null representations that might come from Windows APIs
        assert!(is_empty_string(&Some("null".to_string())));
        assert!(is_empty_string(&Some("NULL".to_string())));
        assert!(is_empty_string(&Some("Null".to_string())));
        assert!(is_empty_string(&Some("nil".to_string())));
        assert!(is_empty_string(&Some("NIL".to_string())));
        assert!(is_empty_string(&Some("undefined".to_string())));
        assert!(is_empty_string(&Some("UNDEFINED".to_string())));
        assert!(is_empty_string(&Some("(null)".to_string())));
        assert!(is_empty_string(&Some("<null>".to_string())));
        assert!(is_empty_string(&Some("n/a".to_string())));
        assert!(is_empty_string(&Some("N/A".to_string())));
        assert!(is_empty_string(&Some("na".to_string())));
        assert!(is_empty_string(&Some("NA".to_string())));

        // Test Windows-specific null patterns
        assert!(is_empty_string(&Some("unknown".to_string())));
        assert!(is_empty_string(&Some("UNKNOWN".to_string())));
        assert!(is_empty_string(&Some("<unknown>".to_string())));
        assert!(is_empty_string(&Some("(unknown)".to_string())));
        assert!(is_empty_string(&Some("none".to_string())));
        assert!(is_empty_string(&Some("NONE".to_string())));
        assert!(is_empty_string(&Some("<none>".to_string())));
        assert!(is_empty_string(&Some("(none)".to_string())));
        assert!(is_empty_string(&Some("empty".to_string())));
        assert!(is_empty_string(&Some("EMPTY".to_string())));
        assert!(is_empty_string(&Some("<empty>".to_string())));
        assert!(is_empty_string(&Some("(empty)".to_string())));

        // Test COM/Windows API specific patterns
        assert!(is_empty_string(&Some("BSTR()".to_string())));
        assert!(is_empty_string(&Some("variant()".to_string())));
        assert!(is_empty_string(&Some("VARIANT(EMPTY)".to_string())));
        assert!(is_empty_string(&Some("Variant(Empty)".to_string())));

        // Test with surrounding whitespace
        assert!(is_empty_string(&Some(" null ".to_string())));
        assert!(is_empty_string(&Some("\t(null)\n".to_string())));
        assert!(is_empty_string(&Some("  UNKNOWN  ".to_string())));

        // Test valid strings that should NOT be filtered
        assert!(!is_empty_string(&Some("test".to_string())));
        assert!(!is_empty_string(&Some("valid content".to_string())));
        assert!(!is_empty_string(&Some("0".to_string())));
        assert!(!is_empty_string(&Some("false".to_string())));
        assert!(!is_empty_string(&Some("Button".to_string())));

        // Test edge cases that might look like null but aren't
        assert!(!is_empty_string(&Some("not null".to_string())));
        assert!(!is_empty_string(&Some("nullify".to_string())));
        assert!(!is_empty_string(&Some("nullable".to_string())));
        assert!(!is_empty_string(&Some("unknown value".to_string())));
        assert!(!is_empty_string(&Some("something empty".to_string())));
        assert!(!is_empty_string(&Some("none selected".to_string())));
    }
}
