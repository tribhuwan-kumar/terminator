use crate::events::{ButtonClickEvent, ButtonInteractionType};
use crate::{
    ApplicationSwitchEvent, ApplicationSwitchMethod, BrowserTabNavigationEvent, ClipboardAction,
    ClipboardEvent, EventMetadata, HotkeyEvent, KeyboardEvent, MouseButton, MouseEvent,
    MouseEventType, Position, Result, WorkflowEvent, WorkflowRecorderConfig,
};
use arboard::Clipboard;
use rdev::{Button, EventType, Key};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant, SystemTime},
};
use terminator::{convert_uiautomation_element_to_terminator, UIElement};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uiautomation::UIAutomation;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_MULTITHREADED,
};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostThreadMessageW, TranslateMessage, MSG, WM_QUIT,
};

/// Represents an input event that requires UI Automation processing.
enum UIAInputRequest {
    ButtonPress {
        button: MouseButton,
        position: Position,
    },
    ButtonRelease {
        button: MouseButton,
        position: Position,
    },
    MouseMove {
        position: Position,
    },
    Wheel {
        delta: (i32, i32),
        position: Position,
    },
}

/// Text input tracking state
#[derive(Debug, Clone)]
struct TextInputTracker {
    /// The UI element being tracked
    element: UIElement,
    /// When tracking started
    start_time: Instant,
    /// Number of typing keystrokes (excludes navigation keys)
    keystroke_count: u32,
    /// Initial text value when tracking started (unused in current implementation)
    #[allow(dead_code)]
    initial_text: String,
    /// Whether we've detected any actual typing
    has_typing_activity: bool,
    /// Whether we're in the middle of autocomplete navigation (arrow keys active)
    in_autocomplete_navigation: bool,
    /// Last time we detected autocomplete navigation activity
    last_autocomplete_activity: Instant,
    /// Text value before autocomplete selection (for change detection)
    text_before_autocomplete: Option<String>,
}

impl TextInputTracker {
    fn new(element: UIElement) -> Self {
        // Don't try to get initial text to avoid potential access violations
        Self {
            element,
            start_time: Instant::now(),
            keystroke_count: 0,
            initial_text: String::new(), // Keep empty to avoid element access issues
            has_typing_activity: false,
            in_autocomplete_navigation: false,
            last_autocomplete_activity: Instant::now(),
            text_before_autocomplete: None,
        }
    }

    fn add_keystroke(&mut self, key_code: u32) {
        // Check for autocomplete navigation keys (arrow keys, escape)
        if Self::is_autocomplete_navigation_key(key_code) {
            self.in_autocomplete_navigation = true;
            self.last_autocomplete_activity = Instant::now();
            debug!(
                "🔽 Autocomplete navigation detected: key {} (Arrow/Escape)",
                key_code
            );

            // Capture current text value before potential autocomplete selection
            if self.text_before_autocomplete.is_none() {
                self.text_before_autocomplete = Self::get_element_text_value_safe(&self.element);
                debug!(
                    "📝 Captured text before autocomplete: {:?}",
                    self.text_before_autocomplete
                );
            }
            return;
        }

        // Only count actual typing keys, not navigation/modifier keys
        if Self::is_typing_key(key_code) {
            self.keystroke_count += 1;
            self.has_typing_activity = true;
            // Reset autocomplete state on new typing
            self.in_autocomplete_navigation = false;
        }
    }

    fn is_autocomplete_navigation_key(key_code: u32) -> bool {
        matches!(
            key_code,
            0x26 |  // Up arrow
            0x28 |  // Down arrow  
            0x25 |  // Left arrow (less common in autocomplete but possible)
            0x27 |  // Right arrow (less common in autocomplete but possible)
            0x1B // Escape (cancel autocomplete)
        )
    }

    fn is_typing_key(key_code: u32) -> bool {
        // Letters, numbers, space, punctuation - actual content input
        matches!(key_code,
            0x30..=0x39 |  // Numbers 0-9
            0x41..=0x5A |  // Letters A-Z
            0x20 |         // Space
            0x08 |         // Backspace
            0x2E |         // Delete
            // Common punctuation and symbols
            0xBA..=0xC0 |  // ;=,-./`
            0xDB..=0xDE    // [\]'
        )
    }

    fn handle_enter_key(&mut self) -> bool {
        // If we're in autocomplete navigation, Enter likely selects a suggestion
        if self.in_autocomplete_navigation {
            let time_since_nav = self.last_autocomplete_activity.elapsed();
            if time_since_nav < Duration::from_millis(5000) {
                // 5 second window
                debug!("🔥 Enter pressed during autocomplete navigation - suggestion selection detected!");
                self.has_typing_activity = true;
                self.keystroke_count += 1; // Count as one interaction
                self.in_autocomplete_navigation = false; // Reset state
                return true; // Indicates this is a suggestion selection
            }
        }
        false
    }

    fn should_emit_completion(&self, reason: &str) -> bool {
        // For trigger keys (Enter/Tab), require both activity and keystrokes
        if reason == "trigger_key" || reason == "suggestion_enter" {
            return self.has_typing_activity && self.keystroke_count > 0;
        }

        // For focus changes, be more lenient - emit if there was any activity
        if reason == "focus_change" {
            return self.has_typing_activity || self.keystroke_count > 0;
        }

        // For suggestion clicks, check if we have activity
        if reason == "suggestion_click" {
            return self.has_typing_activity || self.keystroke_count > 0;
        }

        // Default: require activity
        self.has_typing_activity && self.keystroke_count > 0
    }

    #[allow(dead_code)]
    fn text_changed(&self) -> bool {
        // Always return false to avoid accessing element properties
        // We'll rely on keystroke counting instead
        false
    }

    fn get_completion_event(
        &self,
        input_method: Option<crate::TextInputMethod>,
    ) -> Option<crate::TextInputCompletedEvent> {
        // Only proceed if we have typing activity
        if !self.has_typing_activity && self.keystroke_count == 0 {
            debug!("❌ No typing activity or keystrokes");
            return None;
        }

        let typing_duration_ms = self.start_time.elapsed().as_millis() as u64;

        // Use safe fallbacks for element properties
        let field_name = self.element.name();
        let field_type = self.element.role();

        // Try to get actual text value from the element
        let text_value = match Self::get_element_text_value_safe(&self.element) {
            Some(actual_text) if !actual_text.trim().is_empty() => {
                debug!("✅ Got actual text value: '{}'", actual_text);
                actual_text
            }
            Some(empty_text) => {
                debug!(
                    "⚠️ Got empty text value: '{}', using placeholder",
                    empty_text
                );
                format!(
                    "(text input completed, {} keystrokes)",
                    self.keystroke_count
                )
            }
            None => {
                debug!("⚠️ Could not get text value, using placeholder");
                format!(
                    "(text input completed, {} keystrokes)",
                    self.keystroke_count
                )
            }
        };

        // Determine input method
        let final_input_method = input_method.unwrap_or(crate::TextInputMethod::Typed);

        Some(crate::TextInputCompletedEvent {
            text_value,
            field_name,
            field_type,
            input_method: final_input_method,
            typing_duration_ms,
            keystroke_count: self.keystroke_count,
            metadata: EventMetadata::with_ui_element_and_timestamp(Some(self.element.clone())),
        })
    }

    // Safe wrapper for getting element text value
    fn get_element_text_value_safe(element: &UIElement) -> Option<String> {
        WindowsRecorder::get_element_text_value(element)
    }
}

/// The Windows-specific recorder
pub struct WindowsRecorder {
    /// The event sender
    event_tx: broadcast::Sender<WorkflowEvent>,

    /// The configuration
    config: WorkflowRecorderConfig,

    /// The last mouse position
    last_mouse_pos: Arc<Mutex<Option<(i32, i32)>>>,

    /// Signal to stop the listener thread
    stop_indicator: Arc<AtomicBool>,

    /// Modifier key states
    modifier_states: Arc<Mutex<ModifierStates>>,

    /// Last clipboard content hash for change detection
    last_clipboard_hash: Arc<Mutex<Option<u64>>>,

    /// Last mouse move time for throttling
    last_mouse_move_time: Arc<Mutex<Instant>>,

    /// Known hotkey patterns
    hotkey_patterns: Arc<Vec<HotkeyPattern>>,

    /// UI Automation thread ID for proper cleanup
    ui_automation_thread_id: Arc<Mutex<Option<u32>>>,

    /// Current application tracking for switch detection
    current_application: Arc<Mutex<Option<ApplicationState>>>,

    /// Browser tab navigation tracking
    browser_tab_tracker: Arc<Mutex<BrowserTabTracker>>,

    /// Rate limiting for performance modes
    last_event_time: Arc<Mutex<std::time::Instant>>,

    /// Event counter for rate limiting
    events_this_second: Arc<Mutex<(u32, std::time::Instant)>>,

    /// Currently focused text input element tracking with keystroke counting
    current_text_input: Arc<Mutex<Option<TextInputTracker>>>,
}

#[derive(Debug, Clone)]
struct ModifierStates {
    ctrl: bool,
    alt: bool,
    shift: bool,
    win: bool,
}

#[derive(Debug, Clone)]
struct HotkeyPattern {
    action: String,
    keys: Vec<u32>,
}

/// Tracks the current application state for switch detection
#[derive(Debug, Clone)]
struct ApplicationState {
    /// Application name/title
    name: String,
    /// Process ID
    process_id: u32,
    /// When the application became active
    start_time: Instant,
}

/// Tracks browser tab navigation state
#[derive(Debug, Clone)]
struct BrowserTabTracker {
    /// Current browser application
    current_browser: Option<String>,
    /// Current URL (best effort detection)
    current_url: Option<String>,
    /// Current page title
    current_title: Option<String>,
    /// Known browser process names
    known_browsers: Vec<String>,
    /// When the current page was last accessed
    last_navigation_time: Instant,
}

impl Default for BrowserTabTracker {
    fn default() -> Self {
        Self {
            current_browser: None,
            current_url: None,
            current_title: None,
            known_browsers: vec![
                // Executable names
                "chrome.exe".to_string(),
                "firefox.exe".to_string(),
                "msedge.exe".to_string(),
                "brave.exe".to_string(),
                "opera.exe".to_string(),
                "vivaldi.exe".to_string(),
                "iexplore.exe".to_string(),
                // Display names
                "google chrome".to_string(),
                "chrome".to_string(),
                "firefox".to_string(),
                "mozilla firefox".to_string(),
                "microsoft edge".to_string(),
                "edge".to_string(),
                "brave".to_string(),
                "opera".to_string(),
                "vivaldi".to_string(),
                "internet explorer".to_string(),
            ],
            last_navigation_time: Instant::now(),
        }
    }
}

impl WindowsRecorder {
    /// Capture the current timestamp in milliseconds since epoch
    fn capture_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Creates a UIAutomation instance with the configured threading model for a new thread.
    fn create_configured_automation_instance(
        config: &WorkflowRecorderConfig,
    ) -> std::result::Result<UIAutomation, String> {
        unsafe {
            let threading_model = if config.enable_multithreading {
                COINIT_MULTITHREADED
            } else {
                COINIT_APARTMENTTHREADED
            };
            let hr = CoInitializeEx(None, threading_model);
            if hr.is_err() && hr != windows::Win32::Foundation::RPC_E_CHANGED_MODE {
                let err_msg = format!("Failed to initialize COM for new thread: {:?}", hr);
                error!("{}", err_msg);
                return Err(err_msg);
            }
        }
        UIAutomation::new_direct().map_err(|e| {
            let err_msg = format!("Failed to create UIAutomation instance directly: {}", e);
            error!("{}", err_msg);
            err_msg
        })
    }

    /// Create a new Windows recorder
    pub async fn new(
        config: WorkflowRecorderConfig,
        event_tx: broadcast::Sender<WorkflowEvent>,
    ) -> Result<Self> {
        info!("Initializing comprehensive Windows recorder");
        debug!("Recorder config: {:?}", config);

        let last_mouse_pos = Arc::new(Mutex::new(None));
        let stop_indicator = Arc::new(AtomicBool::new(false));
        let modifier_states = Arc::new(Mutex::new(ModifierStates {
            ctrl: false,
            alt: false,
            shift: false,
            win: false,
        }));
        let last_clipboard_hash = Arc::new(Mutex::new(None));
        let last_mouse_move_time = Arc::new(Mutex::new(Instant::now()));

        // Initialize hotkey patterns
        let hotkey_patterns = Arc::new(Self::initialize_hotkey_patterns());

        let mut recorder = Self {
            event_tx,
            config,
            last_mouse_pos,
            stop_indicator,
            modifier_states,
            last_clipboard_hash,
            last_mouse_move_time,
            hotkey_patterns,
            ui_automation_thread_id: Arc::new(Mutex::new(None)),
            current_application: Arc::new(Mutex::new(None)),
            browser_tab_tracker: Arc::new(Mutex::new(BrowserTabTracker::default())),
            last_event_time: Arc::new(Mutex::new(Instant::now())),
            events_this_second: Arc::new(Mutex::new((0, Instant::now()))),
            current_text_input: Arc::new(Mutex::new(None)),
        };

        let handle = tokio::runtime::Handle::current();

        // Set up comprehensive event listeners
        recorder.setup_comprehensive_listeners(handle).await?;

        Ok(recorder)
    }

    /// Check for application switch and emit event if detected
    fn check_and_emit_application_switch(
        current_app: &Arc<Mutex<Option<ApplicationState>>>,
        event_tx: &broadcast::Sender<WorkflowEvent>,
        new_element: &Option<UIElement>,
        switch_method: ApplicationSwitchMethod,
        config: &WorkflowRecorderConfig,
    ) {
        if !config.record_application_switches {
            return;
        }

        if let Some(element) = new_element {
            let app_name = element.application_name();
            if let Ok(process_id) = element.process_id() {
                if !app_name.is_empty() {
                    let mut current = current_app.lock().unwrap();

                    // Check if this is a new application
                    let is_switch = if let Some(ref current_state) = *current {
                        current_state.process_id != process_id || current_state.name != app_name
                    } else {
                        true // First app detection
                    };

                    if is_switch {
                        let now = Instant::now();

                        // Calculate dwell time for previous app
                        let dwell_time = if let Some(ref current_state) = *current {
                            let duration = now.duration_since(current_state.start_time);
                            if duration.as_millis()
                                >= config.app_switch_dwell_time_threshold_ms as u128
                            {
                                Some(duration.as_millis() as u64)
                            } else {
                                None // Too short, probably just UI noise
                            }
                        } else {
                            None
                        };

                        // Only emit if we have meaningful dwell time or this is first app
                        if dwell_time.is_some() || current.is_none() {
                            let switch_event = ApplicationSwitchEvent {
                                from_application: current.as_ref().map(|s| s.name.clone()),
                                to_application: app_name.clone(),
                                from_process_id: current.as_ref().map(|s| s.process_id),
                                to_process_id: process_id,
                                switch_method,
                                dwell_time_ms: dwell_time,
                                switch_count: None, // TODO: Track Alt+Tab cycles
                                metadata: EventMetadata::with_ui_element_and_timestamp(Some(
                                    element.clone(),
                                )),
                            };

                            if let Err(e) =
                                event_tx.send(WorkflowEvent::ApplicationSwitch(switch_event))
                            {
                                debug!("Failed to send application switch event: {}", e);
                            }
                        }

                        // Update current application state
                        *current = Some(ApplicationState {
                            name: app_name,
                            process_id,
                            start_time: now,
                        });
                    }
                }
            }
        }
    }

    /// Initialize common hotkey patterns
    fn initialize_hotkey_patterns() -> Vec<HotkeyPattern> {
        vec![
            HotkeyPattern {
                action: "Copy".to_string(),
                keys: vec![162, 67], // Ctrl + C
            },
            HotkeyPattern {
                action: "Paste".to_string(),
                keys: vec![162, 86], // Ctrl + V
            },
            HotkeyPattern {
                action: "Cut".to_string(),
                keys: vec![162, 88], // Ctrl + X
            },
            HotkeyPattern {
                action: "Undo".to_string(),
                keys: vec![162, 90], // Ctrl + Z
            },
            HotkeyPattern {
                action: "Redo".to_string(),
                keys: vec![162, 89], // Ctrl + Y
            },
            HotkeyPattern {
                action: "Save".to_string(),
                keys: vec![162, 83], // Ctrl + S
            },
            HotkeyPattern {
                action: "Switch Window".to_string(),
                keys: vec![164, 9], // Alt + Tab
            },
            HotkeyPattern {
                action: "Show Desktop".to_string(),
                keys: vec![91, 68], // Win + D
            },
            HotkeyPattern {
                action: "Task Manager".to_string(),
                keys: vec![162, 160, 27], // Ctrl + Shift + Esc
            },
        ]
    }

    /// Set up comprehensive event listeners
    async fn setup_comprehensive_listeners(
        &mut self,
        handle: tokio::runtime::Handle,
    ) -> Result<()> {
        // Main input event listener (enhanced from original)
        self.setup_enhanced_input_listener().await?;

        // Clipboard monitoring
        if self.config.record_clipboard {
            self.setup_clipboard_monitor()?;
        }

        // UI Automation event monitoring
        self.setup_ui_automation_events(
            Arc::clone(&self.current_application),
            Arc::clone(&self.browser_tab_tracker),
            Arc::clone(&self.current_text_input),
            handle,
        )?;

        Ok(())
    }

    /// Set up enhanced input event listener
    async fn setup_enhanced_input_listener(&mut self) -> Result<()> {
        let event_tx = self.event_tx.clone();
        let last_mouse_pos = Arc::clone(&self.last_mouse_pos);
        let stop_indicator_clone = Arc::clone(&self.stop_indicator);
        let modifier_states = Arc::clone(&self.modifier_states);
        let last_mouse_move_time = Arc::clone(&self.last_mouse_move_time);
        let hotkey_patterns = Arc::clone(&self.hotkey_patterns);
        let config = self.config.clone();
        let performance_last_event_time = Arc::clone(&self.last_event_time);
        let performance_events_counter = Arc::clone(&self.events_this_second);
        let current_text_input = Arc::clone(&self.current_text_input);

        // --- UIA Processor Thread ---
        // Create a channel for rdev events that need UIA processing
        let (uia_event_tx, uia_event_rx) = std::sync::mpsc::channel::<UIAInputRequest>();

        // Spawn the UI Automation processing thread for rdev events
        let uia_processor_event_tx = self.event_tx.clone();
        let uia_processor_config = self.config.clone();
        let uia_processor_text_input = Arc::clone(&self.current_text_input);
        let uia_processor_last_event_time = Arc::clone(&self.last_event_time);
        let uia_processor_events_counter = Arc::clone(&self.events_this_second);
        let capture_ui_elements = self.config.capture_ui_elements;

        thread::spawn(move || {
            if !capture_ui_elements {
                return; // Don't start this thread if UI elements are not needed.
            }

            // Initialize COM and UIAutomation for this dedicated thread
            let automation = match Self::create_configured_automation_instance(
                &uia_processor_config,
            ) {
                Ok(auto) => auto,
                Err(e) => {
                    error!(
                        "Could not create UIAutomation instance in processor thread: {}. UI context for input events will be missing.",
                        e
                    );
                    return;
                }
            };
            info!("✅ UIA processor thread for input events started.");

            // Process events from the rdev listener
            for event_request in uia_event_rx {
                match event_request {
                    UIAInputRequest::ButtonPress { button, position } => {
                        Self::handle_button_press_request(
                            button,
                            &position,
                            &automation,
                            &uia_processor_config,
                            &uia_processor_text_input,
                            &uia_processor_event_tx,
                            &uia_processor_last_event_time,
                            &uia_processor_events_counter,
                        );
                    }
                    UIAInputRequest::ButtonRelease { button, position } => {
                        Self::handle_button_release_request(
                            button,
                            &position,
                            &automation,
                            &uia_processor_config,
                            &uia_processor_event_tx,
                            &uia_processor_last_event_time,
                            &uia_processor_events_counter,
                        );
                    }
                    UIAInputRequest::MouseMove { position } => {
                        Self::handle_mouse_move_request(
                            &position,
                            &automation,
                            &uia_processor_config,
                            &uia_processor_event_tx,
                            &uia_processor_last_event_time,
                            &uia_processor_events_counter,
                        );
                    }
                    UIAInputRequest::Wheel { delta, position } => {
                        Self::handle_wheel_request(
                            delta,
                            &position,
                            &automation,
                            &uia_processor_config,
                            &uia_processor_event_tx,
                            &uia_processor_last_event_time,
                            &uia_processor_events_counter,
                        );
                    }
                }
            }
        });

        // --- Rdev Input Listener Thread ---
        thread::spawn(move || {
            let track_modifiers = config.track_modifier_states;
            let record_hotkeys = config.record_hotkeys;
            let mouse_move_throttle = config.mouse_move_throttle_ms;
            let capture_ui_elements_rdev = config.capture_ui_elements;

            let mut active_keys: HashMap<u32, bool> = HashMap::new();

            if let Err(error) = rdev::listen(move |event: rdev::Event| {
                if stop_indicator_clone.load(Ordering::SeqCst) {
                    return;
                }

                match event.event_type {
                    EventType::KeyPress(key) => {
                        let key_code = key_to_u32(&key);
                        active_keys.insert(key_code, true);

                        // Track keystrokes for text input completion
                        if config.record_text_input_completion {
                            if let Ok(mut tracker) = current_text_input.try_lock() {
                                if let Some(ref mut text_input) = tracker.as_mut() {
                                    text_input.add_keystroke(key_code);
                                    // Don't log here to avoid spam

                                    // Check for completion trigger keys (Enter, Tab)
                                    if key_code == 0x0D || key_code == 0x09 {
                                        let is_suggestion_enter = if key_code == 0x0D {
                                            text_input.handle_enter_key()
                                        } else {
                                            false
                                        };
                                        let completion_reason = if is_suggestion_enter {
                                            "suggestion_enter"
                                        } else {
                                            "trigger_key"
                                        };

                                        if text_input.should_emit_completion(completion_reason) {
                                            let input_method = if is_suggestion_enter {
                                                Some(crate::TextInputMethod::Suggestion)
                                            } else {
                                                None
                                            };
                                            if let Some(text_event) =
                                                text_input.get_completion_event(input_method)
                                            {
                                                let _ = event_tx.send(
                                                    WorkflowEvent::TextInputCompleted(text_event),
                                                );
                                                if let Some(element) =
                                                    &tracker.as_ref().map(|t| t.element.clone())
                                                {
                                                    *tracker = Some(TextInputTracker::new(
                                                        element.clone(),
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Update modifier states
                        if track_modifiers {
                            Self::update_modifier_states(&modifier_states, key_code, true);
                        }

                        // Check for hotkeys
                        if record_hotkeys {
                            if let Some(hotkey) =
                                Self::detect_hotkey(&hotkey_patterns, &active_keys)
                            {
                                let _ = event_tx.send(WorkflowEvent::Hotkey(hotkey));
                            }
                        }

                        let modifiers = if track_modifiers {
                            modifier_states.lock().unwrap().clone()
                        } else {
                            ModifierStates {
                                ctrl: false,
                                alt: false,
                                shift: false,
                                win: false,
                            }
                        };
                        let character = if (32..=126).contains(&key_code) {
                            Some(key_code as u8 as char)
                        } else {
                            None
                        };

                        let keyboard_event = KeyboardEvent {
                            key_code,
                            is_key_down: true,
                            ctrl_pressed: modifiers.ctrl,
                            alt_pressed: modifiers.alt,
                            shift_pressed: modifiers.shift,
                            win_pressed: modifiers.win,
                            character,
                            scan_code: None,
                            metadata: EventMetadata {
                                ui_element: None,
                                timestamp: Some(Self::capture_timestamp()),
                            },
                        };

                        Self::send_filtered_event_static(
                            &event_tx,
                            &config,
                            &performance_last_event_time,
                            &performance_events_counter,
                            WorkflowEvent::Keyboard(keyboard_event),
                        );
                    }
                    EventType::KeyRelease(key) => {
                        let key_code = key_to_u32(&key);
                        active_keys.remove(&key_code);

                        if track_modifiers {
                            Self::update_modifier_states(&modifier_states, key_code, false);
                        }

                        let modifiers = if track_modifiers {
                            modifier_states.lock().unwrap().clone()
                        } else {
                            ModifierStates {
                                ctrl: false,
                                alt: false,
                                shift: false,
                                win: false,
                            }
                        };

                        let keyboard_event = KeyboardEvent {
                            key_code,
                            is_key_down: false,
                            ctrl_pressed: modifiers.ctrl,
                            alt_pressed: modifiers.alt,
                            shift_pressed: modifiers.shift,
                            win_pressed: modifiers.win,
                            character: None,
                            scan_code: None,
                            metadata: EventMetadata {
                                ui_element: None,
                                timestamp: Some(Self::capture_timestamp()),
                            },
                        };
                        Self::send_filtered_event_static(
                            &event_tx,
                            &config,
                            &performance_last_event_time,
                            &performance_events_counter,
                            WorkflowEvent::Keyboard(keyboard_event),
                        );
                    }
                    EventType::ButtonPress(button) => {
                        if let Some((x, y)) = *last_mouse_pos.lock().unwrap() {
                            let mouse_button = match button {
                                Button::Left => MouseButton::Left,
                                Button::Right => MouseButton::Right,
                                Button::Middle => MouseButton::Middle,
                                _ => return,
                            };
                            let position = Position { x, y };

                            if capture_ui_elements_rdev {
                                let request = UIAInputRequest::ButtonPress {
                                    button: mouse_button,
                                    position,
                                };
                                let _ = uia_event_tx.send(request);
                            } else {
                                let mouse_event = MouseEvent {
                                    event_type: MouseEventType::Down,
                                    button: mouse_button,
                                    position,
                                    scroll_delta: None,
                                    drag_start: None,
                                    metadata: EventMetadata {
                                        ui_element: None,
                                        timestamp: Some(Self::capture_timestamp()),
                                    },
                                };
                                Self::send_filtered_event_static(
                                    &event_tx,
                                    &config,
                                    &performance_last_event_time,
                                    &performance_events_counter,
                                    WorkflowEvent::Mouse(mouse_event),
                                );
                            }
                        }
                    }
                    EventType::ButtonRelease(button) => {
                        if let Some((x, y)) = *last_mouse_pos.lock().unwrap() {
                            let mouse_button = match button {
                                Button::Left => MouseButton::Left,
                                Button::Right => MouseButton::Right,
                                Button::Middle => MouseButton::Middle,
                                _ => return,
                            };
                            let position = Position { x, y };

                            if capture_ui_elements_rdev {
                                let request = UIAInputRequest::ButtonRelease {
                                    button: mouse_button,
                                    position,
                                };
                                let _ = uia_event_tx.send(request);
                            } else {
                                let mouse_event = MouseEvent {
                                    event_type: MouseEventType::Up,
                                    button: mouse_button,
                                    position,
                                    scroll_delta: None,
                                    drag_start: None,
                                    metadata: EventMetadata {
                                        ui_element: None,
                                        timestamp: Some(Self::capture_timestamp()),
                                    },
                                };
                                Self::send_filtered_event_static(
                                    &event_tx,
                                    &config,
                                    &performance_last_event_time,
                                    &performance_events_counter,
                                    WorkflowEvent::Mouse(mouse_event),
                                );
                            }
                        }
                    }
                    EventType::MouseMove { x, y } => {
                        let x = x as i32;
                        let y = y as i32;
                        *last_mouse_pos.lock().unwrap() = Some((x, y));

                        let now = Instant::now();
                        let should_record = {
                            let mut last_time = last_mouse_move_time.lock().unwrap();
                            if now.duration_since(*last_time).as_millis()
                                >= mouse_move_throttle as u128
                            {
                                *last_time = now;
                                true
                            } else {
                                false
                            }
                        };

                        if should_record {
                            let position = Position { x, y };
                            if capture_ui_elements_rdev {
                                let request = UIAInputRequest::MouseMove { position };
                                let _ = uia_event_tx.send(request);
                            } else {
                                let mouse_event = MouseEvent {
                                    event_type: MouseEventType::Move,
                                    button: MouseButton::Left,
                                    position,
                                    scroll_delta: None,
                                    drag_start: None,
                                    metadata: EventMetadata {
                                        ui_element: None,
                                        timestamp: Some(Self::capture_timestamp()),
                                    },
                                };
                                Self::send_filtered_event_static(
                                    &event_tx,
                                    &config,
                                    &performance_last_event_time,
                                    &performance_events_counter,
                                    WorkflowEvent::Mouse(mouse_event),
                                );
                            }
                        }
                    }
                    EventType::Wheel { delta_x, delta_y } => {
                        if let Some((x, y)) = *last_mouse_pos.lock().unwrap() {
                            let position = Position { x, y };
                            if capture_ui_elements_rdev {
                                let request = UIAInputRequest::Wheel {
                                    delta: (delta_x as i32, delta_y as i32),
                                    position,
                                };
                                if uia_event_tx.send(request).is_err() {
                                    debug!("Failed to send wheel event to UIA processor thread");
                                }
                            } else {
                                let mouse_event = MouseEvent {
                                    event_type: MouseEventType::Wheel,
                                    button: MouseButton::Middle, // Common for wheel
                                    position,
                                    scroll_delta: Some((delta_x as i32, delta_y as i32)),
                                    drag_start: None,
                                    metadata: EventMetadata {
                                        ui_element: None,
                                        timestamp: Some(Self::capture_timestamp()),
                                    },
                                };
                                Self::send_filtered_event_static(
                                    &event_tx,
                                    &config,
                                    &performance_last_event_time,
                                    &performance_events_counter,
                                    WorkflowEvent::Mouse(mouse_event),
                                );
                            }
                        }
                    }
                }
            }) {
                error!("Failed to listen for events: {:?}", error);
            }
        });

        Ok(())
    }

    /// Update modifier key states
    fn update_modifier_states(states: &Arc<Mutex<ModifierStates>>, key_code: u32, pressed: bool) {
        let mut states = states.lock().unwrap();
        match key_code {
            162 | 163 => states.ctrl = pressed,  // Left/Right Ctrl
            164 | 165 => states.alt = pressed,   // Left/Right Alt
            160 | 161 => states.shift = pressed, // Left/Right Shift
            91 | 92 => states.win = pressed,     // Left/Right Win
            _ => {}
        }
    }

    /// Detect hotkey combinations
    fn detect_hotkey(
        patterns: &[HotkeyPattern],
        active_keys: &HashMap<u32, bool>,
    ) -> Option<HotkeyEvent> {
        for pattern in patterns {
            if pattern
                .keys
                .iter()
                .all(|&key| active_keys.get(&key).copied().unwrap_or(false))
            {
                return Some(HotkeyEvent {
                    combination: format!("{:?}", pattern.keys), // TODO: Format properly
                    action: Some(pattern.action.clone()),
                    is_global: true,
                    metadata: EventMetadata {
                        ui_element: None,
                        timestamp: Some(Self::capture_timestamp()),
                    }, // TODO: Pass UI element context from caller
                });
            }
        }
        None
    }

    /// Set up clipboard monitoring
    fn setup_clipboard_monitor(&self) -> Result<()> {
        let event_tx = self.event_tx.clone();
        let stop_indicator = Arc::clone(&self.stop_indicator);
        let last_hash = Arc::clone(&self.last_clipboard_hash);
        let config = self.config.clone();
        let capture_ui_elements = self.config.capture_ui_elements;

        thread::spawn(move || {
            let mut clipboard = match Clipboard::new() {
                Ok(cb) => cb,
                Err(e) => {
                    error!("Failed to initialize clipboard: {}", e);
                    return;
                }
            };

            // Use configured automation instance
            let automation = if capture_ui_elements {
                match Self::create_configured_automation_instance(&config) {
                    Ok(auto) => Some(auto),
                    Err(e) => {
                        warn!("⚠️ Failed to create UIAutomation for clipboard monitor: {}. UI context will be missing.", e);
                        None
                    }
                }
            } else {
                None
            };

            // Initialize the clipboard hash with current content to avoid false initial events
            if let Ok(initial_content) = clipboard.get_text() {
                let initial_hash = Self::calculate_hash(&initial_content);
                *last_hash.lock().unwrap() = Some(initial_hash);
                debug!("Initialized clipboard monitoring with existing content hash");
            }

            while !stop_indicator.load(Ordering::SeqCst) {
                if let Ok(content) = clipboard.get_text() {
                    let hash = Self::calculate_hash(&content);
                    let mut last_hash_guard = last_hash.lock().unwrap();

                    if last_hash_guard.as_ref() != Some(&hash) {
                        *last_hash_guard = Some(hash);
                        drop(last_hash_guard);

                        let (truncated_content, truncated) =
                            if content.len() > config.max_clipboard_content_length {
                                (
                                    content[..config.max_clipboard_content_length].to_string(),
                                    true,
                                )
                            } else {
                                (content.clone(), false)
                            };

                        // Capture UI element if enabled
                        let ui_element = if capture_ui_elements {
                            automation
                                .as_ref()
                                .and_then(Self::get_focused_ui_element_with_timeout)
                        } else {
                            None
                        };

                        let clipboard_event = ClipboardEvent {
                            action: ClipboardAction::Copy, // Assume copy for content changes
                            content: Some(truncated_content),
                            content_size: Some(content.len()),
                            format: Some("text".to_string()),
                            truncated,
                            metadata: EventMetadata {
                                ui_element,
                                timestamp: Some(Self::capture_timestamp()),
                            },
                        };

                        let _ = event_tx.send(WorkflowEvent::Clipboard(clipboard_event));
                    }
                }

                thread::sleep(Duration::from_millis(200)); // PERFORMANCE: Check clipboard every 200ms (was 100ms)
            }
        });

        Ok(())
    }

    /// Calculate hash for content comparison
    fn calculate_hash(content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Get focused UI element with timeout protection to prevent hanging
    fn get_focused_ui_element_with_timeout(automation: &UIAutomation) -> Option<UIElement> {
        // Use panic catching to handle any COM/threading issues gracefully
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            automation.get_focused_element()
        })) {
            Ok(Ok(element)) => {
                // Successfully got element, now safely convert it
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    convert_uiautomation_element_to_terminator(element)
                })) {
                    Ok(ui_element) => Some(ui_element),
                    Err(_) => {
                        debug!("Failed to convert UI element safely");
                        None
                    }
                }
            }
            Ok(Err(e)) => {
                debug!("UI Automation call failed: {}", e);
                None
            }
            Err(_) => {
                debug!("UI Automation call panicked, handled gracefully");
                None
            }
        }
    }

    /// Set up UI Automation event handlers
    fn setup_ui_automation_events(
        &self,
        current_application: Arc<Mutex<Option<ApplicationState>>>,
        browser_tab_tracker: Arc<Mutex<BrowserTabTracker>>,
        current_text_input: Arc<Mutex<Option<TextInputTracker>>>,
        handle: tokio::runtime::Handle,
    ) -> Result<()> {
        let event_tx = self.event_tx.clone();
        let stop_indicator = Arc::clone(&self.stop_indicator);
        let ui_automation_thread_id = Arc::clone(&self.ui_automation_thread_id);

        // Clone filtering configuration
        let ignore_focus_patterns = self.config.ignore_focus_patterns.clone();
        let ignore_window_titles = self.config.ignore_window_titles.clone();
        let ignore_applications = self.config.ignore_applications.clone();
        let config_clone = self.config.clone();

        thread::spawn(move || {
            info!("Starting UI Automation event monitoring thread");

            // Initialize COM apartment for UI Automation events
            // Threading model controlled by configuration
            let threading_model = if config_clone.enable_multithreading {
                COINIT_MULTITHREADED
            } else {
                COINIT_APARTMENTTHREADED
            };

            let com_initialized = unsafe {
                let hr = CoInitializeEx(None, threading_model);
                if hr.is_ok() {
                    let threading_name = if config_clone.enable_multithreading {
                        "multithreaded (MTA)"
                    } else {
                        "apartment threaded (STA)"
                    };
                    info!(
                        "✅ Successfully initialized COM apartment as {} for UI Automation events",
                        threading_name
                    );
                    true
                } else if hr == windows::Win32::Foundation::RPC_E_CHANGED_MODE {
                    warn!("⚠️  COM apartment already initialized with different threading model");
                    // This is expected if the main process already initialized COM differently
                    false
                } else {
                    error!(
                        "❌ Failed to initialize COM apartment for UI Automation: {:?}",
                        hr
                    );
                    return;
                }
            };

            // Store the thread ID for cleanup
            let thread_id = unsafe { GetCurrentThreadId() };
            *ui_automation_thread_id.lock().unwrap() = Some(thread_id);

            info!(
                "UI Automation event thread started (Thread ID: {})",
                thread_id
            );

            // Use new_direct() to avoid COM initialization conflicts
            // The uiautomation library's new() method tries to initialize COM as MTA which conflicts with our STA setup
            let automation = match uiautomation::UIAutomation::new_direct() {
                Ok(auto) => {
                    info!("✅ Successfully created UIAutomation instance using new_direct()");
                    auto
                }
                Err(e) => {
                    error!("❌ Failed to create UIAutomation instance: {}", e);
                    warn!(
                        "UI Automation events will be disabled, but other recording will continue"
                    );

                    // Still run message pump for potential future use
                    Self::run_message_pump(&stop_indicator);

                    // Clean up COM if we initialized it
                    if com_initialized {
                        unsafe {
                            CoUninitialize();
                        }
                    }
                    return;
                }
            };

            info!("UI Automation instance created successfully, setting up event handlers");

            // Set up focus change event handler if enabled
            info!("Setting up focus change event handler");
            let focus_event_tx = event_tx.clone();

            // Create a channel to signal focus changes, without sending data to avoid blocking.
            let (focus_tx, focus_rx) = std::sync::mpsc::channel::<()>();

            struct FocusHandler {
                sender: std::sync::mpsc::Sender<()>,
            }

            impl uiautomation::events::CustomFocusChangedEventHandler for FocusHandler {
                fn handle(&self, _sender: &uiautomation::UIElement) -> uiautomation::Result<()> {
                    // This handler is on a critical UIA thread.
                    // DO NOT perform any blocking operations here.
                    // Just send a signal to the processor thread to do the work.
                    self.sender.send(()).ok(); // Disregard error if receiver is gone.
                    Ok(())
                }
            }

            let focus_handler = FocusHandler { sender: focus_tx };

            let focus_event_handler =
                uiautomation::events::UIFocusChangedEventHandler::from(focus_handler);

            // Register the focus change event handler
            match automation.add_focus_changed_event_handler(None, &focus_event_handler) {
                Ok(_) => info!("✅ Focus change event handler registered successfully"),
                Err(e) => error!("❌ Failed to register focus change event handler: {}", e),
            }

            // This thread receives signals and performs the blocking UI Automation work safely.
            let focus_event_tx_clone = focus_event_tx.clone();
            let focus_current_app = Arc::clone(&current_application);
            let focus_browser_tracker = Arc::clone(&browser_tab_tracker);
            let focus_current_text_input = Arc::clone(&current_text_input);

            let focus_processing_config = config_clone.clone();
            let focus_processing_ignore_patterns = ignore_focus_patterns.clone();
            let focus_processing_ignore_window_titles = ignore_window_titles.clone();
            let focus_processing_ignore_applications = ignore_applications.clone();
            let processing_handle = handle;

            std::thread::spawn(move || {
                // This thread needs its own UIA instance.
                let automation = match Self::create_configured_automation_instance(
                    &focus_processing_config,
                ) {
                    Ok(auto) => auto,
                    Err(e) => {
                        error!("❌ Failed to create UIA instance in focus processor thread: {}. Focus events will not be processed.", e);
                        return;
                    }
                };

                while focus_rx.recv().is_ok() {
                    // Received a signal. Now, get the currently focused element.
                    // This is the main blocking call, now safely on a dedicated thread.
                    let ui_element = Self::get_focused_ui_element_with_timeout(&automation);

                    if let Some(element) = ui_element {
                        let element_name = element.name_or_empty();
                        let element_role = element.role().to_lowercase();
                        debug!(
                            "Focus event received for element: '{}', role: '{}'",
                            element_name, element_role
                        );

                        // Task for button focus check
                        let button_focus_event_tx = focus_event_tx_clone.clone();
                        let button_element = element.clone();
                        processing_handle.spawn(async move {
                            WindowsRecorder::handle_button_focus_event(
                                &button_element,
                                &button_focus_event_tx,
                            );
                        });

                        // Task for application switch check
                        let app_switch_current_app = Arc::clone(&focus_current_app);
                        let app_switch_event_tx_clone = focus_event_tx_clone.clone();
                        let app_switch_element_name = element_name.clone();
                        let app_switch_ui_element = Some(element.clone());
                        let app_switch_config_clone = focus_processing_config.clone();
                        let app_switch_ignore_focus_patterns =
                            focus_processing_ignore_patterns.clone();
                        let app_switch_ignore_window_titles =
                            focus_processing_ignore_window_titles.clone();
                        let app_switch_ignore_applications =
                            focus_processing_ignore_applications.clone();

                        processing_handle.spawn(async move {
                            if WindowsRecorder::should_ignore_focus_event(
                                &app_switch_element_name,
                                &app_switch_ui_element,
                                &app_switch_ignore_focus_patterns,
                                &app_switch_ignore_window_titles,
                                &app_switch_ignore_applications,
                            ) {
                                debug!(
                                    "Ignoring focus change event for app switch check: {}",
                                    app_switch_element_name
                                );
                                return;
                            }

                            Self::check_and_emit_application_switch(
                                &app_switch_current_app,
                                &app_switch_event_tx_clone,
                                &app_switch_ui_element,
                                ApplicationSwitchMethod::WindowClick,
                                &app_switch_config_clone,
                            );
                        });

                        // Task for browser navigation check
                        let browser_nav_tracker = Arc::clone(&focus_browser_tracker);
                        let browser_nav_event_tx_clone = focus_event_tx_clone.clone();
                        let browser_nav_ui_element = Some(element.clone());
                        let browser_nav_config_clone = focus_processing_config.clone();

                        processing_handle.spawn(async move {
                            Self::check_and_emit_browser_navigation(
                                &browser_nav_tracker,
                                &browser_nav_event_tx_clone,
                                &browser_nav_ui_element,
                                &browser_nav_config_clone,
                            )
                            .await;
                        });

                        // Task for text input completion check
                        let text_input_tracker = Arc::clone(&focus_current_text_input);
                        let text_input_event_tx = focus_event_tx_clone.clone();
                        let text_input_element = Some(element);
                        let text_input_config = focus_processing_config.clone();

                        processing_handle.spawn(async move {
                            WindowsRecorder::handle_text_input_focus_change(
                                &text_input_tracker,
                                &text_input_event_tx,
                                &text_input_element,
                                &text_input_config,
                            );
                        });
                    }
                }
            });

            info!("✅ UI Automation event handlers setup complete, starting message pump");

            // CRITICAL: Start Windows message pump for COM/UI Automation events
            Self::run_message_pump(&stop_indicator);

            info!("UI Automation event monitoring stopped");

            // Clean up COM if we initialized it
            if com_initialized {
                unsafe {
                    CoUninitialize();
                }
                debug!("COM uninitialized");
            }
        });

        Ok(())
    }

    /// Run the Windows message pump for UI Automation events
    fn run_message_pump(stop_indicator: &Arc<AtomicBool>) {
        info!("Starting Windows message pump for UI Automation events");
        unsafe {
            let mut msg = MSG::default();
            while !stop_indicator.load(Ordering::SeqCst) {
                let result = GetMessageW(&mut msg, None, 0, 0);

                match result.0 {
                    -1 => {
                        // Error occurred
                        error!("Error in message pump: GetMessage failed");
                        break;
                    }
                    0 => {
                        // WM_QUIT received
                        debug!("WM_QUIT received in UI Automation message pump");
                        break;
                    }
                    _ => {
                        // Normal message - process it
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);

                        // Check for quit message
                        if msg.message == WM_QUIT {
                            debug!("WM_QUIT message processed");
                            break;
                        }
                    }
                }

                // Brief yield to check stop condition more frequently
                if msg.message == 0 {
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }
        info!("Windows message pump stopped");
    }

    /// Check if a focus change event should be ignored based on filtering patterns
    fn should_ignore_focus_event(
        element_name: &str,
        ui_element: &Option<UIElement>,
        ignore_patterns: &std::collections::HashSet<String>,
        ignore_window_titles: &std::collections::HashSet<String>,
        ignore_applications: &std::collections::HashSet<String>,
    ) -> bool {
        let element_name_lower = element_name.to_lowercase();

        // Check against focus-specific ignore patterns
        if ignore_patterns
            .iter()
            .any(|pattern| element_name_lower.contains(pattern))
        {
            return true;
        }

        // Check against window title patterns
        if ignore_window_titles
            .iter()
            .any(|title| element_name_lower.contains(title))
        {
            return true;
        }

        // Check against application patterns
        if let Some(ui_elem) = ui_element {
            let app_name = ui_elem.application_name();
            if !app_name.is_empty() {
                let app_name_lower = app_name.to_lowercase();
                if ignore_applications
                    .iter()
                    .any(|app| app_name_lower.contains(app))
                {
                    return true;
                }
            }
        }

        false
    }

    /// Stop recording
    pub fn stop(&self) -> Result<()> {
        debug!("Stopping comprehensive Windows recorder...");
        self.stop_indicator.store(true, Ordering::SeqCst);

        // Signal the UI Automation thread to stop by posting a quit message
        if let Some(thread_id) = *self.ui_automation_thread_id.lock().unwrap() {
            unsafe {
                let result = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
                if result.is_ok() {
                    debug!(
                        "Posted WM_QUIT message to UI Automation thread {}",
                        thread_id
                    );
                } else {
                    warn!(
                        "Failed to post WM_QUIT message to UI Automation thread {}",
                        thread_id
                    );
                }
            }
        }

        info!("Windows recorder stop signal sent");
        Ok(())
    }

    /// Determine the type of button interaction based on element characteristics
    fn determine_button_interaction_type(
        name: &str,
        description: &str,
        role: &str,
    ) -> ButtonInteractionType {
        let name_lower = name.to_lowercase();
        let desc_lower = description.to_lowercase();
        let role_lower = role.to_lowercase();

        // Check for hyperlinks/links first
        if role_lower.contains("hyperlink") || role_lower.contains("link") {
            return ButtonInteractionType::Click; // Hyperlinks are just clicks
        }

        // Check for dropdown indicators
        if name_lower.contains("dropdown")
            || name_lower.contains("▼")
            || name_lower.contains("⏷")
            || desc_lower.contains("dropdown")
            || desc_lower.contains("expand")
            || desc_lower.contains("collapse")
        {
            return ButtonInteractionType::DropdownToggle;
        }

        // Check for submit buttons
        if name_lower.contains("submit")
            || name_lower.contains("save")
            || name_lower.contains("ok")
            || name_lower.contains("apply")
            || name_lower.contains("confirm")
        {
            return ButtonInteractionType::Submit;
        }

        // Check for cancel buttons
        if name_lower.contains("cancel")
            || name_lower.contains("close")
            || name_lower.contains("×")
            || name_lower.contains("dismiss")
        {
            return ButtonInteractionType::Cancel;
        }

        // Check for toggle elements
        if role_lower.contains("toggle")
            || role_lower.contains("checkbox")
            || role_lower.contains("radiobutton")
            || name_lower.contains("toggle")
            || desc_lower.contains("toggle")
        {
            return ButtonInteractionType::Toggle;
        }

        // Default to simple click
        ButtonInteractionType::Click
    }

    /// Static version for use in event listeners where self is not available
    fn send_filtered_event_static(
        event_tx: &broadcast::Sender<WorkflowEvent>,
        config: &WorkflowRecorderConfig,
        last_event_time: &Arc<Mutex<Instant>>,
        events_this_second: &Arc<Mutex<(u32, Instant)>>,
        event: WorkflowEvent,
    ) {
        // Apply rate limiting first
        if let Some(max_events) = config.effective_max_events_per_second() {
            let mut counter = events_this_second.lock().unwrap();
            let now = Instant::now();

            // Reset counter if a new second has started
            if now.duration_since(counter.1).as_secs() >= 1 {
                counter.0 = 0;
                counter.1 = now;
            }

            if counter.0 >= max_events {
                return; // Rate limit exceeded
            }

            counter.0 += 1;
        }

        // Apply processing delay
        let processing_delay = config.effective_processing_delay_ms();
        if processing_delay > 0 {
            let mut last_time = last_event_time.lock().unwrap();
            let now = Instant::now();
            if now.duration_since(*last_time).as_millis() < processing_delay as u128 {
                return; // Filter out if within delay window
            }
            *last_time = now;
        }

        // Apply event-specific filtering
        let should_filter = match &event {
            WorkflowEvent::Mouse(mouse_event) => {
                if config.should_filter_mouse_noise() {
                    matches!(
                        mouse_event.event_type,
                        MouseEventType::Move | MouseEventType::Wheel
                    )
                } else {
                    false
                }
            }
            WorkflowEvent::Keyboard(keyboard_event) => {
                if config.should_filter_keyboard_noise() {
                    // Filter key-down events and non-printable keys
                    if keyboard_event.is_key_down {
                        // Keep printable characters (32-126) and common editing keys
                        !((keyboard_event.key_code >= 32 && keyboard_event.key_code <= 126)
                            || matches!(
                                keyboard_event.key_code,
                                0x08 | // Backspace
                            0x2E | // Delete
                            0x20 | // Space  
                            0x0D | // Enter
                            0x09 // Tab
                            ))
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            // Never filter high-value events
            WorkflowEvent::ApplicationSwitch(_)
            | WorkflowEvent::ButtonClick(_)
            | WorkflowEvent::Clipboard(_) => false,

            // Other events can be filtered in LowEnergy mode
            _ => matches!(config.performance_mode, crate::PerformanceMode::LowEnergy),
        };

        if !should_filter {
            let _ = event_tx.send(event);
        }
    }

    /// Check and emit browser navigation events with improved filtering
    async fn check_and_emit_browser_navigation(
        tracker: &Arc<Mutex<BrowserTabTracker>>,
        event_tx: &broadcast::Sender<WorkflowEvent>,
        ui_element: &Option<UIElement>,
        config: &WorkflowRecorderConfig,
    ) {
        if !config.record_browser_tab_navigation {
            return;
        }

        if let Some(element) = ui_element {
            let app_name = element.application_name();
            let app_name_lower = app_name.to_lowercase();

            let is_browser = {
                let tracker_guard = tracker.lock().unwrap();
                tracker_guard
                    .known_browsers
                    .iter()
                    .any(|b| app_name_lower.contains(b))
            };

            debug!(
                "Checking browser navigation for app: '{}', is_browser: {}",
                app_name, is_browser
            );

            if !is_browser {
                return;
            }

            // Try multiple methods to get URL information
            let url_info = element
                .url()
                .or_else(|| {
                    // Try to get URL from element attributes or text
                    if let Ok(text) = element.text(0) {
                        if text.starts_with("http") {
                            Some(text)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .or_else(|| {
                    // Try to extract URL from window title (common in browsers)
                    let window_title = element.window_title();
                    if window_title.contains("http") {
                        // Extract URL from title
                        window_title
                            .split_whitespace()
                            .find(|s| s.starts_with("http"))
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                });

            if let Some(new_url) = url_info {
                // Use window title as page title, fallback to app name
                let new_title = {
                    let window_title = element.window_title();
                    if window_title.is_empty() {
                        app_name.clone()
                    } else {
                        window_title
                    }
                };

                debug!(
                    "Browser navigation detected - URL: '{}', Title: '{}'",
                    new_url, new_title
                );

                let mut tracker_guard = tracker.lock().unwrap();

                let is_switch = new_url != tracker_guard.current_url.clone().unwrap_or_default()
                    || new_title != tracker_guard.current_title.clone().unwrap_or_default();

                debug!(
                    "Is switch: {}, current_url: {:?}, current_title: {:?}",
                    is_switch, tracker_guard.current_url, tracker_guard.current_title
                );

                if is_switch {
                    let now = Instant::now();
                    let dwell_time = now
                        .duration_since(tracker_guard.last_navigation_time)
                        .as_millis() as u64;

                    let nav_event = BrowserTabNavigationEvent {
                        action: crate::TabAction::Switched,
                        method: crate::TabNavigationMethod::Other, // Updated from focus change
                        to_url: Some(new_url.clone()),
                        from_url: tracker_guard.current_url.clone(),
                        to_title: Some(new_title.clone()),
                        from_title: tracker_guard.current_title.clone(),
                        browser: app_name.clone(),
                        tab_index: None,
                        total_tabs: None,
                        page_dwell_time_ms: Some(dwell_time),
                        is_back_forward: false,
                        metadata: EventMetadata::with_ui_element_and_timestamp(Some(
                            element.clone(),
                        )),
                    };

                    debug!("Sending browser navigation event: {:?}", nav_event);

                    if event_tx
                        .send(WorkflowEvent::BrowserTabNavigation(nav_event))
                        .is_ok()
                    {
                        debug!("✅ Browser navigation event sent successfully");
                        tracker_guard.current_browser = Some(app_name.clone());
                        tracker_guard.current_url = Some(new_url);
                        tracker_guard.current_title = Some(new_title);
                        tracker_guard.last_navigation_time = now;
                    } else {
                        debug!("❌ Failed to send browser navigation event");
                    }
                }
            } else {
                debug!(
                    "No URL information found for browser element: '{}'",
                    element.name_or_empty()
                );
            }
        }
    }

    /// Handles a focus event to determine if it's a button-like interaction and sends an event.
    /// This is run in a separate task to avoid blocking the main focus event processing loop.
    fn handle_button_focus_event(element: &UIElement, event_tx: &broadcast::Sender<WorkflowEvent>) {
        let element_name = element.name_or_empty();
        let element_role = element.role().to_lowercase();

        debug!(
            "Button focus check - element: '{}', role: '{}'",
            element_name, element_role
        );

        // Check if the focused element is clickable (button, menu item, list item, hyperlink, etc.)
        if element_role.contains("button")
            || element_role.contains("menuitem")
            || element_role.contains("listitem")
            || element_role.contains("hyperlink")
            || element_role.contains("link")
            || element_role.contains("checkbox")
            || element_role.contains("radiobutton")
            || element_role.contains("togglebutton")
        {
            debug!(
                "✅ Detected clickable element: '{}' (role: '{}')",
                element_name, element_role
            );
            let element_desc = element.attributes().description.unwrap_or_default();

            let interaction_type = Self::determine_button_interaction_type(
                &element_name,
                &element_desc,
                &element_role,
            );
            let is_enabled = element.is_enabled().unwrap_or(true);
            let bounds = element.bounds().unwrap_or_default();

            let button_event = ButtonClickEvent {
                button_text: element_name.clone(),
                interaction_type,
                button_role: element_role.clone(),
                was_enabled: is_enabled,
                click_position: Some(Position {
                    x: bounds.0 as i32,
                    y: bounds.1 as i32,
                }),
                button_description: if element_desc.is_empty() {
                    None
                } else {
                    Some(element_desc.clone())
                },
                metadata: EventMetadata::with_ui_element_and_timestamp(Some(element.clone())),
            };

            let result = event_tx.send(WorkflowEvent::ButtonClick(button_event));

            if result.is_ok() {
                debug!("Successfully sent ButtonClickEvent for '{}'", element_name);
            } else {
                warn!(
                    "Failed to send ButtonClickEvent for '{}': {:?}",
                    element_name,
                    result.err()
                );
            }
        }
    }

    /// Check if a UI element is a text input field
    fn is_text_input_element(element: &UIElement) -> bool {
        let role = element.role().to_lowercase();

        // Only track actual input fields, not documents or other containers
        role.contains("edit")
            || role == "text"
            || (role.contains("combobox") && element.is_enabled().unwrap_or(false))
        // Only editable combobox
    }

    /// Get the text value from a UI element
    /// Try to find a recently active text input element for suggestion completion
    fn find_recent_text_input(automation: &UIAutomation) -> Option<UIElement> {
        // Try to find the currently focused element first
        if let Some(focused_element) = Self::get_focused_ui_element_with_timeout(automation) {
            if Self::is_text_input_element(&focused_element) {
                debug!(
                    "🎯 Found focused text input element: '{}'",
                    focused_element.name_or_empty()
                );
                return Some(focused_element);
            }
        }

        debug!("❌ Could not find any recent text input elements using focused element approach");
        None
    }

    fn get_element_text_value(element: &UIElement) -> Option<String> {
        // Try multiple methods to get the text value with multiple attempts for timing issues

        for attempt in 0..3 {
            if attempt > 0 {
                // Small delay between attempts to allow UI updates
                std::thread::sleep(std::time::Duration::from_millis(50));
            }

            // First try the value attribute (most reliable for input fields)
            if let Some(value) = element.attributes().value {
                if !value.trim().is_empty() {
                    debug!(
                        "✅ Got text value from 'value' attribute (attempt {}): '{}'",
                        attempt + 1,
                        value
                    );
                    return Some(value);
                }
            }

            // Then try the text() method which gets the actual text content
            if let Ok(text) = element.text(0) {
                if !text.trim().is_empty() {
                    debug!(
                        "✅ Got text value from text() method (attempt {}): '{}'",
                        attempt + 1,
                        text
                    );
                    return Some(text);
                }
            }

            // Try enhanced text methods for better coverage
            if let Ok(text) = element.text(1) {
                if !text.trim().is_empty() {
                    debug!(
                        "✅ Got text value from text(1) method (attempt {}): '{}'",
                        attempt + 1,
                        text
                    );
                    return Some(text);
                }
            }

            // Check for description which might contain the value
            if let Some(description) = element.attributes().description {
                if !description.trim().is_empty() && description.len() < 200 {
                    debug!(
                        "✅ Got text value from description (attempt {}): '{}'",
                        attempt + 1,
                        description
                    );
                    return Some(description);
                }
            }

            // Check the label which might contain the value
            if let Some(label) = element.attributes().label {
                if !label.trim().is_empty() && label.len() < 200 {
                    debug!(
                        "✅ Got text value from label (attempt {}): '{}'",
                        attempt + 1,
                        label
                    );
                    return Some(label);
                }
            }
        }

        // Finally try the name as last resort
        let name = element.name_or_empty();
        if !name.trim().is_empty() && name.len() < 200 {
            debug!("✅ Got text value from name (fallback): '{}'", name);
            Some(name)
        } else {
            debug!("❌ Could not extract text value from element after all attempts");
            None
        }
    }

    /// Handles text input focus changes to detect text input completion
    fn handle_text_input_focus_change(
        current_text_input: &Arc<Mutex<Option<TextInputTracker>>>,
        event_tx: &broadcast::Sender<WorkflowEvent>,
        new_element: &Option<UIElement>,
        config: &WorkflowRecorderConfig,
    ) {
        if !config.record_text_input_completion {
            return;
        }

        // Use centralized text input tracking logic
        Self::handle_text_input_transition(
            current_text_input,
            event_tx,
            new_element,
            "focus_change",
            config,
        );
    }

    /// Centralized text input tracking logic to avoid conflicts between mouse and focus handlers
    fn handle_text_input_transition(
        current_text_input: &Arc<Mutex<Option<TextInputTracker>>>,
        event_tx: &broadcast::Sender<WorkflowEvent>,
        new_element: &Option<UIElement>,
        trigger_reason: &str,
        config: &WorkflowRecorderConfig,
    ) {
        if !config.record_text_input_completion {
            return;
        }

        let mut tracker = match current_text_input.try_lock() {
            Ok(guard) => guard,
            Err(_) => {
                debug!("❌ Could not lock text input tracker for transition");
                return;
            }
        };

        // Check if the new element is a potential autocomplete suggestion
        let is_potential_autocomplete_element = if let Some(element) = new_element {
            let element_role = element.role().to_lowercase();
            let element_name = element.name_or_empty();

            element_role.contains("listitem")
                || element_role.contains("menuitem")
                || element_role.contains("option")
                || element_role.contains("comboboxitem")
                || element_role.contains("item")
                || element_role.contains("cell")
                || element_role == "text"
                || element_name.to_lowercase().contains("suggestion")
                || element_name.to_lowercase().contains("complete")
                || element_name.to_lowercase().contains("autocomplete")
                || element_name.to_lowercase().contains("dropdown")
                || (element_role == "pane" && !element_name.is_empty())
                || (element_role == "document" && element_name.len() < 100)
        } else {
            false
        };

        // If we're focusing on a potential autocomplete element, preserve the existing tracker
        if is_potential_autocomplete_element && tracker.is_some() {
            if let Some(element) = new_element {
                debug!(
                    "🔽 Focus moved to potential autocomplete element: '{}' (role: '{}') - PRESERVING text input tracker",
                    element.name_or_empty(), element.role()
                );
            }
            return; // Don't modify the tracker when focusing on autocomplete elements
        }

        // Check if we're leaving a text input field
        if let Some(existing_tracker) = tracker.as_ref() {
            let element_name = existing_tracker.element.name_or_empty();

            // Only remove tracker if we're truly leaving for a non-autocomplete element
            let should_remove_tracker = if let Some(element) = new_element {
                Self::is_text_input_element(element) || !is_potential_autocomplete_element
            } else {
                true // Remove if no focus
            };

            if should_remove_tracker {
                debug!(
                    "🔄 Leaving text input field: '{}' (reason: {})",
                    element_name, trigger_reason
                );

                // Take the tracker to check for completion
                let existing_tracker = tracker.take().unwrap();

                // Check if we should emit a completion event
                if existing_tracker.should_emit_completion(trigger_reason) {
                    debug!("✅ Should emit completion event for {}", trigger_reason);
                    if let Some(text_event) = existing_tracker.get_completion_event(None) {
                        debug!(
                            "🔥 Emitting text input completion event: '{}' (reason: {})",
                            text_event.text_value, trigger_reason
                        );
                        if let Err(e) = event_tx.send(WorkflowEvent::TextInputCompleted(text_event))
                        {
                            debug!("Failed to send text input completed event: {}", e);
                        }
                    } else {
                        debug!("❌ get_completion_event returned None");
                    }
                } else {
                    debug!("❌ Should NOT emit completion event for {}", trigger_reason);
                }
            } else {
                debug!(
                    "🔽 Staying in text input context: '{}' (reason: {})",
                    element_name, trigger_reason
                );
            }
        }

        // Check if the new element is a text input field (and we don't already have a tracker)
        if let Some(element) = new_element {
            let element_name = element.name_or_empty();
            let element_role = element.role();
            debug!(
                "🔍 Checking new element: '{}' (role: '{}') for text input",
                element_name, element_role
            );

            if Self::is_text_input_element(element) && tracker.is_none() {
                debug!(
                    "✅ New element is a text input field, starting tracking (reason: {})",
                    trigger_reason
                );
                // Store the new text input element with current time
                *tracker = Some(TextInputTracker::new(element.clone()));
                debug!(
                    "🎯 Started tracking text input: '{}' ({})",
                    element_name, element_role
                );
            } else if !Self::is_text_input_element(element) && !is_potential_autocomplete_element {
                debug!(
                    "❌ New element is NOT a text input field: '{}' ({})",
                    element_name, element_role
                );
            }
        } else {
            debug!("🔍 New element is None (no focus)");
        }
    }

    /// Handles a button press request from the input listener thread.
    /// This function performs the UI Automation calls and is expected to run on a dedicated UIA thread.
    #[allow(clippy::too_many_arguments)]
    fn handle_button_press_request(
        button: MouseButton,
        position: &Position,
        automation: &UIAutomation,
        config: &WorkflowRecorderConfig,
        current_text_input: &Arc<Mutex<Option<TextInputTracker>>>,
        event_tx: &broadcast::Sender<WorkflowEvent>,
        performance_last_event_time: &Arc<Mutex<Instant>>,
        performance_events_counter: &Arc<Mutex<(u32, Instant)>>,
    ) {
        let ui_element = Self::get_focused_ui_element_with_timeout(automation);

        // Debug: Log what UI element we captured at mouse down
        if let Some(ref element) = ui_element {
            debug!(
                "Mouse down captured element: name='{}', role='{}', position=({}, {})",
                element.name_or_empty(),
                element.role(),
                position.x,
                position.y
            );
        } else {
            debug!(
                "Mouse down: No UI element captured at position ({}, {})",
                position.x, position.y
            );
        }

        // Check if this is a click on a clickable element and emit button event immediately
        if let Some(ref element) = ui_element {
            if button == MouseButton::Left {
                let element_role = element.role().to_lowercase();
                let element_name = element.name_or_empty();

                // Debug: Log all mouse clicks on elements for debugging
                debug!(
                    "🖱️ Mouse click on element: '{}' (role: '{}') - checking if text input...",
                    element_name, element_role
                );

                // Check if this is a click on a text input element and start tracking
                let is_text_input = Self::is_text_input_element(element);
                debug!(
                    "🔍 is_text_input_element('{}', '{}') = {}",
                    element_name, element_role, is_text_input
                );

                if config.record_text_input_completion && is_text_input {
                    debug!(
                        "🎯 Detected mouse click on text input element: '{}' (role: '{}') - STARTING TRACKING",
                        element_name, element_role
                    );

                    // Use centralized text input tracking logic
                    Self::handle_text_input_transition(
                        current_text_input,
                        event_tx,
                        &Some(element.clone()),
                        "mouse_click",
                        config,
                    );
                }

                // Enhanced autocomplete/suggestion detection
                let is_suggestion_click = element_role.contains("listitem")
                    || element_role.contains("menuitem")
                    || element_role.contains("option")
                    || element_role.contains("comboboxitem")
                    || element_role.contains("item") // Generic item roles
                    || element_role.contains("cell") // Grid/table cells in dropdowns
                    || element_role == "text" // Plain text elements in dropdowns
                    || element_name.to_lowercase().contains("suggestion")
                    || element_name.to_lowercase().contains("complete")
                    || element_name.to_lowercase().contains("autocomplete")
                    || element_name.to_lowercase().contains("dropdown")
                    // Common autocomplete patterns
                    || (element_role == "pane" && !element_name.is_empty())
                    || (element_role == "document" && element_name.len() < 100); // Short text in documents could be suggestions

                // Debug logging for suggestion detection
                debug!(
                    "🔍 Checking suggestion click: element='{}', role='{}', is_suggestion={}, config_enabled={}",
                    element_name,
                    element_role,
                    is_suggestion_click,
                    config.record_text_input_completion
                );

                if is_suggestion_click && config.record_text_input_completion {
                    debug!(
                        "🎯 Detected potential autocomplete/suggestion click: '{}' (role: '{}') - SUGGESTION SELECTED",
                        element_name, element_role
                    );

                    // Check if we have an active text input tracker that might be affected
                    if let Ok(mut tracker) = current_text_input.try_lock() {
                        debug!(
                            "🔒 Successfully locked text input tracker, checking for active tracker..."
                        );
                        if let Some(ref mut text_input) = tracker.as_mut() {
                            debug!(
                                "✅ Found active text input tracker for element: '{}'",
                                text_input.element.name_or_empty()
                            );
                            // Mark as having activity (suggestion selection counts as significant input)
                            text_input.has_typing_activity = true;
                            text_input.keystroke_count += 1; // Count suggestion click as one interaction

                            debug!(
                                "📝 Marking text input as having suggestion selection activity (total keystrokes: {})",
                                text_input.keystroke_count
                            );

                            // Give the UI time to update after suggestion click
                            std::thread::sleep(std::time::Duration::from_millis(150));

                            // Emit suggestion completion with updated text value
                            if text_input.should_emit_completion("suggestion_click") {
                                let completion_time = Instant::now();
                                let typing_duration =
                                    completion_time.duration_since(text_input.start_time);

                                // Use the name of the clicked suggestion element as the final text.
                                let suggested_text = element_name.clone();

                                let text_event = crate::TextInputCompletedEvent {
                                    text_value: suggested_text,
                                    field_name: text_input.element.name(),
                                    field_type: text_input.element.role().to_string(),
                                    keystroke_count: text_input.keystroke_count,
                                    typing_duration_ms: typing_duration.as_millis() as u64,
                                    input_method: crate::TextInputMethod::Suggestion,
                                    metadata: EventMetadata::with_ui_element_and_timestamp(Some(
                                        text_input.element.clone(),
                                    )),
                                };

                                debug!(
                                    "🔥 Emitting text input completion for suggestion click: '{}'",
                                    text_event.text_value
                                );
                                if let Err(e) =
                                    event_tx.send(WorkflowEvent::TextInputCompleted(text_event))
                                {
                                    debug!("Failed to send text input completion event: {}", e);
                                } else {
                                    debug!("✅ Text input completion event sent successfully for suggestion");
                                }

                                // Reset tracker after emitting - clear but keep the element for potential continued typing
                                let element_for_continuation = text_input.element.clone();
                                *tracker = Some(TextInputTracker::new(element_for_continuation));
                                debug!("🔄 Reset text input tracker after suggestion completion but keep tracking the same element");
                            } else {
                                debug!("❌ Should not emit completion for suggestion click");
                            }
                        } else {
                            debug!(
                                "⚠️ Suggestion click detected but no active text input tracker found"
                            );
                            debug!(
                                "💡 Attempting to create temporary tracker for suggestion completion..."
                            );

                            // Try to find the text input element that was recently active
                            // Look for text input elements on the page
                            if let Some(text_element) = Self::find_recent_text_input(automation) {
                                debug!(
                                    "🔍 Found recent text input element: '{}'",
                                    text_element.name_or_empty()
                                );

                                // Create a temporary tracker for this suggestion completion
                                let temp_tracker = TextInputTracker::new(text_element.clone());

                                // Give the UI time to update after suggestion click
                                std::thread::sleep(std::time::Duration::from_millis(150));

                                let completion_time = Instant::now();
                                let typing_duration =
                                    completion_time.duration_since(temp_tracker.start_time);
                                let suggested_text = element_name.clone();

                                let text_event = crate::TextInputCompletedEvent {
                                    text_value: suggested_text,
                                    field_name: temp_tracker.element.name(),
                                    field_type: temp_tracker.element.role().to_string(),
                                    keystroke_count: 1, // just the click
                                    typing_duration_ms: typing_duration.as_millis() as u64,
                                    input_method: crate::TextInputMethod::Suggestion,
                                    metadata: EventMetadata::with_ui_element_and_timestamp(Some(
                                        temp_tracker.element.clone(),
                                    )),
                                };

                                debug!(
                                    "🔥 Emitting text input completion from temp tracker: '{}'",
                                    text_event.text_value
                                );
                                if let Err(e) =
                                    event_tx.send(WorkflowEvent::TextInputCompleted(text_event))
                                {
                                    debug!("Failed to send temp tracker completion event: {}", e);
                                } else {
                                    debug!("✅ Temp tracker completion event sent successfully");
                                }

                                // Create new tracker for potential continued typing
                                *tracker = Some(TextInputTracker::new(text_element));
                                debug!("🔄 Created new tracker after temp completion");
                            } else {
                                debug!("❌ Could not find recent text input element for suggestion completion");
                            }
                        }
                    } else {
                        debug!("❌ Could not lock text input tracker for suggestion click");
                    }
                }

                let is_clickable = element_role.contains("button")
                    || element_role.contains("menuitem")
                    || element_role.contains("listitem")
                    || element_role.contains("hyperlink")
                    || element_role.contains("link")
                    || element_role.contains("checkbox")
                    || element_role.contains("radiobutton")
                    || element_role.contains("togglebutton");

                if is_clickable {
                    debug!(
                        "🖱️ Mouse click on clickable element: '{}' (role: '{}')",
                        element_name, element_role
                    );

                    let element_desc = element.attributes().description.unwrap_or_default();
                    let interaction_type = Self::determine_button_interaction_type(
                        &element_name,
                        &element_desc,
                        &element_role,
                    );

                    let button_event = ButtonClickEvent {
                        button_text: element_name,
                        interaction_type,
                        button_role: element_role.clone(),
                        was_enabled: element.is_enabled().unwrap_or(true),
                        click_position: Some(position.clone()),
                        button_description: if element_desc.is_empty() {
                            None
                        } else {
                            Some(element_desc)
                        },
                        metadata: EventMetadata::with_ui_element_and_timestamp(Some(
                            element.clone(),
                        )),
                    };

                    if let Err(e) = event_tx.send(WorkflowEvent::ButtonClick(button_event)) {
                        debug!("Failed to send button click event: {}", e);
                    } else {
                        debug!("✅ Button click event sent successfully");
                    }
                }
            }
        }

        let mouse_event = MouseEvent {
            event_type: MouseEventType::Down,
            button,
            position: position.clone(),
            scroll_delta: None,
            drag_start: None,
            metadata: EventMetadata {
                ui_element,
                timestamp: Some(Self::capture_timestamp()),
            },
        };
        Self::send_filtered_event_static(
            event_tx,
            config,
            performance_last_event_time,
            performance_events_counter,
            WorkflowEvent::Mouse(mouse_event),
        );
    }

    /// Handles a button release request from the input listener thread.
    #[allow(clippy::too_many_arguments)]
    fn handle_button_release_request(
        button: MouseButton,
        position: &Position,
        automation: &UIAutomation,
        config: &WorkflowRecorderConfig,
        event_tx: &broadcast::Sender<WorkflowEvent>,
        performance_last_event_time: &Arc<Mutex<Instant>>,
        performance_events_counter: &Arc<Mutex<(u32, Instant)>>,
    ) {
        let ui_element = Self::get_focused_ui_element_with_timeout(automation);

        let mouse_event = MouseEvent {
            event_type: MouseEventType::Up,
            button,
            position: position.clone(),
            scroll_delta: None,
            drag_start: None,
            metadata: EventMetadata {
                ui_element,
                timestamp: Some(Self::capture_timestamp()),
            },
        };
        Self::send_filtered_event_static(
            event_tx,
            config,
            performance_last_event_time,
            performance_events_counter,
            WorkflowEvent::Mouse(mouse_event),
        );
    }

    /// Handles a mouse move request from the input listener thread.
    #[allow(clippy::too_many_arguments)]
    fn handle_mouse_move_request(
        position: &Position,
        _automation: &UIAutomation,
        config: &WorkflowRecorderConfig,
        event_tx: &broadcast::Sender<WorkflowEvent>,
        performance_last_event_time: &Arc<Mutex<Instant>>,
        performance_events_counter: &Arc<Mutex<(u32, Instant)>>,
    ) {
        // For performance, we don't get the UI element on every mouse move,
        // but it's an option if high-fidelity tracking is needed.
        // For now, we pass None to avoid high-frequency UIA calls.
        let ui_element = None;

        let mouse_event = MouseEvent {
            event_type: MouseEventType::Move,
            button: MouseButton::Left, // Inactive for move
            position: position.clone(),
            scroll_delta: None,
            drag_start: None,
            metadata: EventMetadata {
                ui_element,
                timestamp: Some(Self::capture_timestamp()),
            },
        };
        Self::send_filtered_event_static(
            event_tx,
            config,
            performance_last_event_time,
            performance_events_counter,
            WorkflowEvent::Mouse(mouse_event),
        );
    }

    /// Handles a mouse wheel request from the input listener thread.
    #[allow(clippy::too_many_arguments)]
    fn handle_wheel_request(
        delta: (i32, i32),
        position: &Position,
        automation: &UIAutomation,
        config: &WorkflowRecorderConfig,
        event_tx: &broadcast::Sender<WorkflowEvent>,
        performance_last_event_time: &Arc<Mutex<Instant>>,
        performance_events_counter: &Arc<Mutex<(u32, Instant)>>,
    ) {
        let ui_element = Self::get_focused_ui_element_with_timeout(automation);

        let mouse_event = MouseEvent {
            event_type: MouseEventType::Wheel,
            button: MouseButton::Middle, // Common for wheel
            position: position.clone(),
            scroll_delta: Some(delta),
            drag_start: None,
            metadata: EventMetadata {
                ui_element,
                timestamp: Some(Self::capture_timestamp()),
            },
        };
        Self::send_filtered_event_static(
            event_tx,
            config,
            performance_last_event_time,
            performance_events_counter,
            WorkflowEvent::Mouse(mouse_event),
        );
    }
}

/// Convert a Key to a u32
fn key_to_u32(key: &Key) -> u32 {
    match key {
        Key::KeyA => 0x41,
        Key::KeyB => 0x42,
        Key::KeyC => 0x43,
        Key::KeyD => 0x44,
        Key::KeyE => 0x45,
        Key::KeyF => 0x46,
        Key::KeyG => 0x47,
        Key::KeyH => 0x48,
        Key::KeyI => 0x49,
        Key::KeyJ => 0x4A,
        Key::KeyK => 0x4B,
        Key::KeyL => 0x4C,
        Key::KeyM => 0x4D,
        Key::KeyN => 0x4E,
        Key::KeyO => 0x4F,
        Key::KeyP => 0x50,
        Key::KeyQ => 0x51,
        Key::KeyR => 0x52,
        Key::KeyS => 0x53,
        Key::KeyT => 0x54,
        Key::KeyU => 0x55,
        Key::KeyV => 0x56,
        Key::KeyW => 0x57,
        Key::KeyX => 0x58,
        Key::KeyY => 0x59,
        Key::KeyZ => 0x5A,
        Key::Num0 => 0x30,
        Key::Num1 => 0x31,
        Key::Num2 => 0x32,
        Key::Num3 => 0x33,
        Key::Num4 => 0x34,
        Key::Num5 => 0x35,
        Key::Num6 => 0x36,
        Key::Num7 => 0x37,
        Key::Num8 => 0x38,
        Key::Num9 => 0x39,
        Key::Escape => 0x1B,
        Key::Backspace => 0x08,
        Key::Tab => 0x09,
        Key::Return => 0x0D,
        Key::Space => 0x20,
        Key::LeftArrow => 0x25,
        Key::UpArrow => 0x26,
        Key::RightArrow => 0x27,
        Key::DownArrow => 0x28,
        Key::Delete => 0x2E,
        Key::Home => 0x24,
        Key::End => 0x23,
        Key::PageUp => 0x21,
        Key::PageDown => 0x22,
        Key::F1 => 0x70,
        Key::F2 => 0x71,
        Key::F3 => 0x72,
        Key::F4 => 0x73,
        Key::F5 => 0x74,
        Key::F6 => 0x75,
        Key::F7 => 0x76,
        Key::F8 => 0x77,
        Key::F9 => 0x78,
        Key::F10 => 0x79,
        Key::F11 => 0x7A,
        Key::F12 => 0x7B,
        Key::ShiftLeft => 0xA0,
        Key::ShiftRight => 0xA1,
        Key::ControlLeft => 0xA2,
        Key::ControlRight => 0xA3,
        Key::Alt => 0xA4,
        Key::AltGr => 0xA5,
        Key::MetaLeft => 0x5B,
        Key::MetaRight => 0x5C,
        _ => 0,
    }
}
