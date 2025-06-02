

/*
/// Interface for platform-specific element implementations
pub(crate) trait UIElementImpl: Send + Sync + Debug {
    fn object_id(&self) -> usize;
    fn id(&self) -> Option<String>;
    fn role(&self) -> String;
    fn attributes(&self) -> UIElementAttributes;
    fn name(&self) -> Option<String> {
        self.attributes().name
    }
    fn children(&self) -> Result<Vec<UIElement>, AutomationError>;
    fn parent(&self) -> Result<Option<UIElement>, AutomationError>;
    fn bounds(&self) -> Result<(f64, f64, f64, f64), AutomationError>; // x, y, width, height
    fn click(&self) -> Result<ClickResult, AutomationError>;
    fn double_click(&self) -> Result<ClickResult, AutomationError>;
    fn right_click(&self) -> Result<(), AutomationError>;
    fn hover(&self) -> Result<(), AutomationError>;
    fn focus(&self) -> Result<(), AutomationError>;
    fn type_text(&self, text: &str, use_clipboard: bool) -> Result<(), AutomationError>;
    fn press_key(&self, key: &str) -> Result<(), AutomationError>;
    fn get_text(&self, max_depth: usize) -> Result<String, AutomationError>;
    fn set_value(&self, value: &str) -> Result<(), AutomationError>;
    fn is_enabled(&self) -> Result<bool, AutomationError>;
    fn is_visible(&self) -> Result<bool, AutomationError>;
    fn is_focused(&self) -> Result<bool, AutomationError>;
    fn perform_action(&self, action: &str) -> Result<(), AutomationError>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn create_locator(&self, selector: Selector) -> Result<Locator, AutomationError>;
    fn scroll(&self, direction: &str, amount: f64) -> Result<(), AutomationError>;

    // New method to activate the window containing the element
    fn activate_window(&self) -> Result<(), AutomationError>;

    // Add a method to clone the box
    fn clone_box(&self) -> Box<dyn UIElementImpl>;

    // New method for keyboard focusable
    fn is_keyboard_focusable(&self) -> Result<bool, AutomationError>;

    // New method for mouse drag
    fn mouse_drag(&self, start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> Result<(), AutomationError>;

    // New methods for mouse control
    fn mouse_click_and_hold(&self, x: f64, y: f64) -> Result<(), AutomationError>;
    fn mouse_move(&self, x: f64, y: f64) -> Result<(), AutomationError>;
    fn mouse_release(&self) -> Result<(), AutomationError>;

    // New methods to get containing application and window
    fn application(&self) -> Result<Option<UIElement>, AutomationError>;
    fn window(&self) -> Result<Option<UIElement>, AutomationError>;

    // New method to highlight the element
    fn highlight(&self, color: Option<u32>, duration: Option<std::time::Duration>) -> Result<(), AutomationError>;

    // New method to get the process ID of the element
    fn process_id(&self) -> Result<u32, AutomationError>;
}

pub trait AccessibilityEngine: Send + Sync {
    /// Get the root UI element
    fn get_root_element(&self) -> UIElement;

    fn get_element_by_id(&self, id: i32) -> Result<UIElement, AutomationError>;

    /// Get the currently focused element
    fn get_focused_element(&self) -> Result<UIElement, AutomationError>;

    /// Get all running applications
    fn get_applications(&self) -> Result<Vec<UIElement>, AutomationError>;

    /// Get application by name
    fn get_application_by_name(&self, name: &str) -> Result<UIElement, AutomationError>;

    /// Get application by process ID
    fn get_application_by_pid(&self, pid: i32, timeout: Option<Duration>) -> Result<UIElement, AutomationError>;

    /// Find elements using a selector
    fn find_element(
        &self,
        selector: &Selector,
        root: Option<&UIElement>,
        timeout: Option<Duration>,
    ) -> Result<UIElement, AutomationError>;

    /// Find all elements matching a selector
    /// Default implementation returns an UnsupportedOperation error,
    /// allowing platform-specific implementations to override as needed
    fn find_elements(
        &self,
        selector: &Selector,
        root: Option<&UIElement>,
        timeout: Option<Duration>,
        depth: Option<usize>,
    ) -> Result<Vec<UIElement>, AutomationError>;

    /// Open an application by name
    fn open_application(&self, app_name: &str) -> Result<UIElement, AutomationError>;

    /// Activate an application by name
    fn activate_application(&self, app_name: &str) -> Result<(), AutomationError>;

    /// Open a URL in a specified browser (or default if None)
    fn open_url(&self, url: &str, browser: Option<&str>) -> Result<UIElement, AutomationError>;

    /// Open a file with its default application
    fn open_file(&self, file_path: &str) -> Result<(), AutomationError>;

    /// Execute a terminal command, choosing the appropriate command based on the OS.
    async fn run_command(
        &self,
        windows_command: Option<&str>,
        unix_command: Option<&str>,
    ) -> Result<crate::CommandOutput, AutomationError>;

    /// Capture a screenshot of the primary monitor
    async fn capture_screen(&self) -> Result<crate::ScreenshotResult, AutomationError>;

    /// Capture a screenshot of a specific monitor by name
    async fn capture_monitor_by_name(&self, name: &str) -> Result<crate::ScreenshotResult, AutomationError>;

    /// Perform OCR on the provided image file (requires async runtime)
    async fn ocr_image_path(&self, image_path: &str) -> Result<String, AutomationError>;

    /// Perform OCR on the provided screenshot data (requires async runtime)
    async fn ocr_screenshot(&self, screenshot: &crate::ScreenshotResult) -> Result<String, AutomationError>;

    /// Activate a browser window containing a specific title
    fn activate_browser_window_by_title(&self, title: &str) -> Result<(), AutomationError>;

    /// Find a window by criteria
    async fn find_window_by_criteria(
        &self,
        title_contains: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<UIElement, AutomationError>;

    /// Get the currently focused browser window (async)
    async fn get_current_browser_window(&self) -> Result<UIElement, AutomationError>;

    /// Get the currently focused window
    async fn get_current_window(&self) -> Result<UIElement, AutomationError>;

    /// Get the currently focused application
    async fn get_current_application(&self) -> Result<UIElement, AutomationError>;

    /// Get the UI tree for a window by its title
    fn get_window_tree_by_title(&self, title: &str) -> Result<UINode, AutomationError>;

    /// Get the UI tree for a window by process ID and optional title
    /// If title is provided and matches, use that window
    /// If title is provided but no match found, fall back to any window from the process ID
    /// If title is None, use any window from the process ID
    fn get_window_tree_by_pid_and_title(&self, pid: u32, title: Option<&str>) -> Result<UINode, AutomationError>;
}

*/

use anyhow::Result;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{State, AppHandle};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub fn open_whatsapp() -> () {
    return ();
}
