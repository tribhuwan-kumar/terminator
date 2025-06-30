use serde_json::Value;
use terminator::Desktop;
use serde::{Deserialize, Serialize};
use tauri::{State, AppHandle};
use tracing::{debug, error, info};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub fn scrap_whatsapp() -> Result<Vec<String>, String> {

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    let desktop = Desktop::new(false, false).map_err(|e| {
        format!("Failed to initialize terminator desktop: {}", e)
    })?;

    #[cfg(target_os = "macos")]
    let desktop = Desktop::new(true, true).map_err(|e| {
        format!("Failed to initialize terminator desktop: {}", e)
    })?;

    let whatsapp = desktop.open_application("whatsapp");


    Ok()
}
