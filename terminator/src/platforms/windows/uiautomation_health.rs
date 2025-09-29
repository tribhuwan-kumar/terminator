//! UIAutomation health check module for Windows
//!
//! This module provides health check functionality to verify that the UIAutomation API
//! is available and functioning correctly. This is critical for detecting when VMs lose
//! RDP connections or virtual display functionality.

use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{debug, error, warn};
use uiautomation::UIAutomation;
use windows::core::HRESULT;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIAutomationHealthStatus {
    /// Whether UIAutomation API is available
    pub available: bool,
    /// Whether we can access the desktop root element
    pub can_access_desktop: bool,
    /// Whether we can enumerate child elements
    pub can_enumerate_children: bool,
    /// Time taken to perform the health check in milliseconds
    pub check_duration_ms: u64,
    /// Error message if any check failed
    pub error_message: Option<String>,
    /// Additional diagnostic information
    pub diagnostics: Option<UIAutomationDiagnostics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIAutomationDiagnostics {
    /// Number of desktop children found (if enumeration succeeded)
    pub desktop_child_count: Option<usize>,
    /// Whether COM initialization succeeded
    pub com_initialized: bool,
    /// Whether we detected a headless/RDP environment
    pub is_headless: bool,
    /// Current display configuration
    pub display_info: Option<String>,
}

impl Default for UIAutomationHealthStatus {
    fn default() -> Self {
        Self {
            available: false,
            can_access_desktop: false,
            can_enumerate_children: false,
            check_duration_ms: 0,
            error_message: Some("Health check not performed".to_string()),
            diagnostics: None,
        }
    }
}

/// Performs a comprehensive health check of the UIAutomation API
pub async fn check_uiautomation_health() -> UIAutomationHealthStatus {
    let start = Instant::now();

    // Run the check in a blocking task with a timeout since UIAutomation uses COM
    let check_future = tokio::task::spawn_blocking(|| perform_sync_health_check());

    // Apply a 5-second timeout to the health check
    let result = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        check_future
    ).await {
        Ok(Ok(status)) => status,
        Ok(Err(e)) => {
            error!("Failed to spawn UIAutomation health check task: {}", e);
            UIAutomationHealthStatus {
                available: false,
                can_access_desktop: false,
                can_enumerate_children: false,
                check_duration_ms: start.elapsed().as_millis() as u64,
                error_message: Some(format!("Failed to spawn health check task: {}", e)),
                diagnostics: None,
            }
        }
        Err(_) => {
            error!("UIAutomation health check timed out after 5 seconds");
            UIAutomationHealthStatus {
                available: false,
                can_access_desktop: false,
                can_enumerate_children: false,
                check_duration_ms: start.elapsed().as_millis() as u64,
                error_message: Some("Health check timed out after 5 seconds - UIAutomation API may be unresponsive".to_string()),
                diagnostics: None,
            }
        }
    };

    result
}

fn perform_sync_health_check() -> UIAutomationHealthStatus {
    let start = Instant::now();
    let mut diagnostics = UIAutomationDiagnostics {
        desktop_child_count: None,
        com_initialized: false,
        is_headless: crate::platforms::windows::virtual_display::is_headless_environment(),
        display_info: None,
    };

    // Step 1: Initialize COM
    let com_result = unsafe {
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        if hr.is_err() && hr != HRESULT(0x80010106u32 as i32) {
            // 0x80010106 = RPC_E_CHANGED_MODE (already initialized in different mode)
            Err(format!("Failed to initialize COM: {:?}", hr))
        } else {
            diagnostics.com_initialized = true;
            Ok(())
        }
    };

    if let Err(e) = com_result {
        return UIAutomationHealthStatus {
            available: false,
            can_access_desktop: false,
            can_enumerate_children: false,
            check_duration_ms: start.elapsed().as_millis() as u64,
            error_message: Some(format!("COM initialization failed: {}", e)),
            diagnostics: Some(diagnostics),
        };
    }

    // Step 2: Create UIAutomation instance
    let automation = match UIAutomation::new_direct() {
        Ok(a) => a,
        Err(e) => {
            error!("Failed to create UIAutomation instance: {}", e);
            return UIAutomationHealthStatus {
                available: false,
                can_access_desktop: false,
                can_enumerate_children: false,
                check_duration_ms: start.elapsed().as_millis() as u64,
                error_message: Some(format!("UIAutomation creation failed: {}", e)),
                diagnostics: Some(diagnostics),
            };
        }
    };

    debug!("UIAutomation instance created successfully");

    // Step 3: Try to get desktop root element
    let root_element = match automation.get_root_element() {
        Ok(root) => root,
        Err(e) => {
            error!("Failed to get desktop root element: {}", e);

            // Check if we're in a headless/virtual display environment
            if diagnostics.is_headless {
                diagnostics.display_info = Some("Virtual display may be disconnected".to_string());
            }

            return UIAutomationHealthStatus {
                available: true, // API is available but desktop is not accessible
                can_access_desktop: false,
                can_enumerate_children: false,
                check_duration_ms: start.elapsed().as_millis() as u64,
                error_message: Some(format!("Cannot access desktop: {}. This typically indicates RDP disconnection or virtual display issues.", e)),
                diagnostics: Some(diagnostics),
            };
        }
    };

    debug!("Desktop root element obtained");

    // Step 4: Try to enumerate desktop children
    let can_enumerate = match automation.create_true_condition() {
        Ok(condition) => {
            match root_element.find_all(uiautomation::types::TreeScope::Children, &condition) {
                Ok(children) => {
                    let child_count = children.len();
                    diagnostics.desktop_child_count = Some(child_count);
                    debug!("Found {} desktop children", child_count);

                    if child_count == 0 {
                        warn!("Desktop has no children - possible display issue");
                        diagnostics.display_info = Some("Desktop has no child windows".to_string());
                    }

                    true
                }
                Err(e) => {
                    error!("Failed to enumerate desktop children: {}", e);
                    false
                }
            }
        }
        Err(e) => {
            error!("Failed to create condition for enumeration: {}", e);
            false
        }
    };

    // Step 5: Additional diagnostics for virtual display
    if diagnostics.is_headless {
        // Try to get display information
        if let Ok(name) = root_element.get_name() {
            diagnostics.display_info = Some(format!("Desktop name: {}", name));
        }

        // Check if we can get bounding rectangle (indicates display is properly configured)
        match root_element.get_bounding_rectangle() {
            Ok(rect) => {
                let info = format!(
                    "Display bounds: {}x{} at ({},{})",
                    rect.get_width(),
                    rect.get_height(),
                    rect.get_left(),
                    rect.get_top()
                );
                diagnostics.display_info = Some(info);

                if rect.get_width() == 0 || rect.get_height() == 0 {
                    warn!("Display has zero dimensions - virtual display may be misconfigured");
                }
            }
            Err(e) => {
                warn!("Cannot get display bounds: {} - display may be disconnected", e);
                diagnostics.display_info = Some("Cannot determine display bounds".to_string());
            }
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    // Determine overall health status
    let (healthy, error_message) = if !can_enumerate {
        (false, Some("UIAutomation API is available but cannot enumerate UI elements".to_string()))
    } else if diagnostics.desktop_child_count == Some(0) {
        (false, Some("Desktop has no child windows - display may be disconnected".to_string()))
    } else {
        (true, None)
    };

    UIAutomationHealthStatus {
        available: true,
        can_access_desktop: true,
        can_enumerate_children: can_enumerate,
        check_duration_ms: duration_ms,
        error_message: if healthy { None } else { error_message },
        diagnostics: Some(diagnostics),
    }
}

/// Quick health check that just verifies basic UIAutomation availability
pub async fn quick_health_check() -> bool {
    let result = tokio::task::spawn_blocking(|| {
        // Try to initialize COM and create UIAutomation
        unsafe {
            let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
            if hr.is_err() && hr != HRESULT(0x80010106u32 as i32) {
                return false;
            }
        }

        // Try to create UIAutomation and get root
        match UIAutomation::new_direct() {
            Ok(automation) => automation.get_root_element().is_ok(),
            Err(_) => false,
        }
    })
    .await
    .unwrap_or(false);

    result
}