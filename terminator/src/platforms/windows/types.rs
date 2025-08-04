//! Type definitions and RAII wrappers for Windows platform

use crate::AutomationError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use uiautomation::UIAutomation;
use windows::Win32::Foundation::{CloseHandle, HANDLE};

/// RAII wrapper for Windows HANDLE that ensures proper cleanup
pub(crate) struct HandleGuard(pub(crate) HANDLE);

impl Drop for HandleGuard {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_invalid() {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

/// Thread-safe wrapper for UIAutomation COM object
pub struct ThreadSafeWinUIAutomation();

// Safety: UIAutomation is thread-safe after proper COM initialization
unsafe impl Send for ThreadSafeWinUIAutomation {}
unsafe impl Sync for ThreadSafeWinUIAutomation {}

/// Position options for text overlays in highlighting
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
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

/// Font styling options for text overlays
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FontStyle {
    pub size: u32,
    pub bold: bool,
    pub color: u32, // BGR format
}

impl Default for FontStyle {
    fn default() -> Self {
        Self {
            size: 12,
            bold: false,
            color: 0x000000, // Black
        }
    }
}

/// Handle for managing active highlights with cleanup
pub struct HighlightHandle {
    pub(crate) should_close: Arc<AtomicBool>,
    pub(crate) handle: Option<thread::JoinHandle<()>>,
}

impl HighlightHandle {
    /// Manually close the highlight
    pub fn close(mut self) {
        self.should_close.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for HighlightHandle {
    fn drop(&mut self) {
        self.should_close.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Thread-safe wrapper for UIElement
#[derive(Clone)]
pub(crate) struct ThreadSafeWinUIElement(pub(crate) Arc<uiautomation::UIElement>);

// Safety: UIElement is thread-safe when wrapped properly
unsafe impl Send for ThreadSafeWinUIElement {}
unsafe impl Sync for ThreadSafeWinUIElement {}

/// Options for application activation
#[allow(dead_code)]
pub enum ActivateOptions {
    None = 0x00000000,
    DesignMode = 0x00000001,
    NoErrorUI = 0x00000002,
    NoSplashScreen = 0x00000004,
}

// Tree building types are now in tree_builder.rs

// Error conversions

impl From<uiautomation::Error> for AutomationError {
    fn from(error: uiautomation::Error) -> Self {
        AutomationError::PlatformError(format!("UIAutomation error: {error}"))
    }
}
