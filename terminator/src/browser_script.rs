//! Cross-platform browser script execution using terminator SDK
//!
//! Uses terminator SDK selectors for cross-platform browser automation.
//! Finds console tab and prompt using proper selectors, runs JavaScript, extracts results.

use crate::AutomationError;
use std::time::Duration;
use tracing::{debug, info};

/// Execute JavaScript in browser using terminator SDK selectors
pub async fn execute_script(
    browser_element: &crate::UIElement,
    script: &str,
) -> Result<String, AutomationError> {
    info!("🚀 Executing JavaScript using terminator SDK: {}", script);

    // Step 1: Focus the browser window
    debug!("🎯 Focusing browser window");
    browser_element.click()?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Step 2: Open dev tools if not already open (F12)
    debug!("⚙️ Opening dev tools (F12)");
    browser_element.press_key("{F12}")?;
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Step 3: Find Console tab using terminator selector
    debug!("🖥️ Finding Console tab using name:Console");
    match browser_element.locator("name:Console")?.first(None).await {
        Ok(console_tab) => {
            debug!("✅ Found Console tab, clicking it");
            console_tab.click()?;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Err(_) => debug!("⚠️ Console tab not found or already active"),
    }

    // Step 4: Clear console using the Clear console button
    debug!("🧹 Clearing console using Clear console button");
    match browser_element
        .locator("name:Clear console - Ctrl + L")?
        .first(None)
        .await
    {
        Ok(clear_button) => {
            debug!("✅ Found clear console button, clicking it");
            clear_button.click()?;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Err(_) => debug!("⚠️ Clear console button not found, proceeding anyway"),
    }

    // Step 5: Find console prompt using terminator selector
    debug!("🔍 Finding console prompt using name:Console prompt");
    let console_prompt = browser_element
        .locator("name:Console prompt")?
        .first(None)
        .await?;

    debug!("⌨️ Typing JavaScript into console prompt");
    console_prompt.type_text(script, true)?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Step 6: Execute the script (Enter)
    debug!("🚀 Executing script with Enter");
    console_prompt.press_key("{ENTER}")?;
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Step 7: Get result from console messages area
    debug!("📄 Getting result from console messages");
    let result = match browser_element
        .locator("nativeid:console-messages")?
        .first(None)
        .await
    {
        Ok(console_messages) => {
            debug!("✅ Found console messages area");

            // Look for Text elements in the console messages - the result will be in the last one
            match console_messages.locator("role:Text")?.all(None, None).await {
                Ok(text_elements) => {
                    debug!("📋 Found {} text elements in console", text_elements.len());

                    // Find the result by looking at element names (where the result is stored)
                    let mut result_text = String::new();
                    for element in text_elements.iter().rev().take(10) {
                        // Check last 10 elements
                        // Try name first (where console results are stored)
                        if let Some(name) = element.name() {
                            debug!("🔍 Text element name: '{}'", name);
                            let trimmed = name.trim();

                            // Look for our JavaScript result - skip input elements and errors
                            if !trimmed.is_empty()
                                && !trimmed.contains("6sense")
                                && !trimmed.contains("Hit MAX_ITERATIONS")
                                && !trimmed.contains("Form Element:")
                                && !trimmed.contains("Failed to load")
                                && !trimmed.contains("Access to XMLHttpRequest")
                                && trimmed != "document"
                                && trimmed != "."
                                && trimmed != "title"
                                && trimmed != script.trim()
                            {
                                // Not the input line
                                result_text = trimmed.to_string();
                                debug!("✅ Found result in name: '{}'", result_text);
                                break;
                            }
                        }

                        // Fallback to text content
                        if result_text.is_empty() {
                            if let Ok(text) = element.text(100) {
                                debug!("🔍 Text element text: '{}'", text);
                                let trimmed = text.trim();

                                if !trimmed.is_empty()
                                    && trimmed != "document"
                                    && trimmed != "."
                                    && trimmed != "title"
                                    && !trimmed.starts_with("document.title")
                                {
                                    result_text = trimmed.to_string();
                                    debug!("✅ Found result in text: '{}'", result_text);
                                    break;
                                }
                            }
                        }
                    }

                    if result_text.is_empty() {
                        debug!("⚠️ No result text found, trying full text extraction");
                        console_messages.text(100).unwrap_or_default()
                    } else {
                        result_text
                    }
                }
                Err(_) => {
                    debug!("⚠️ Couldn't get text elements, using full text");
                    console_messages.text(100).unwrap_or_default()
                }
            }
        }
        Err(_) => {
            debug!("⚠️ Couldn't find console messages, trying clipboard approach");
            get_console_result_via_clipboard(&console_prompt).await?
        }
    };

    // Step 8: Close dev tools
    debug!("🚪 Closing dev tools");
    browser_element.press_key("{F12}")?;

    debug!("✅ Script execution completed: {}", result);
    Ok(result)
}

/// Fallback method: get console result via clipboard
async fn get_console_result_via_clipboard(
    console_prompt: &crate::UIElement,
) -> Result<String, AutomationError> {
    debug!("📋 Getting console result via clipboard");

    // Clear clipboard first
    let _ = set_clipboard_content("").await;

    // Navigate to the last console output and copy it
    console_prompt.press_key("^{END}")?;
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Go up to the result line
    console_prompt.press_key("{UP}")?;
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Select the line and copy
    console_prompt.press_key("^a")?;
    tokio::time::sleep(Duration::from_millis(200)).await;

    console_prompt.press_key("^c")?;
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Get clipboard content
    get_clipboard_content().await
}

/// Set clipboard content (cross-platform)
async fn set_clipboard_content(content: &str) -> Result<(), AutomationError> {
    use std::process::Command;

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args(["-command", &format!("Set-Clipboard '{content}'")])
            .output()
            .map_err(|e| {
                AutomationError::PlatformError(format!("Failed to set clipboard: {e}"))
            })?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AutomationError::PlatformError(format!(
                "PowerShell set clipboard error: {error}"
            )));
        }
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("pbcopy").arg(content).output().map_err(|e| {
            AutomationError::PlatformError(format!("Failed to set clipboard: {}", e))
        })?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AutomationError::PlatformError(format!(
                "pbcopy error: {}",
                error
            )));
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try xclip first, then xsel as fallback
        let mut success = false;

        if let Ok(output) = Command::new("xclip")
            .args(&["-selection", "clipboard"])
            .arg(content)
            .output()
        {
            if output.status.success() {
                success = true;
            }
        }

        if !success {
            let output = Command::new("xsel")
                .args(&["--clipboard", "--input"])
                .arg(content)
                .output()
                .map_err(|e| {
                    AutomationError::PlatformError(format!(
                        "Failed to set clipboard (xclip/xsel not available): {}",
                        e
                    ))
                })?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(AutomationError::PlatformError(format!(
                    "xsel error: {}",
                    error
                )));
            }
        }
    }

    Ok(())
}

/// Get content from clipboard (cross-platform)
async fn get_clipboard_content() -> Result<String, AutomationError> {
    use std::process::Command;

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args(["-command", "Get-Clipboard"])
            .output()
            .map_err(|e| {
                AutomationError::PlatformError(format!("Failed to run PowerShell: {e}"))
            })?;

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout).trim().to_string();
            debug!("📋 Clipboard content: {}", content);
            Ok(content)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(AutomationError::PlatformError(format!(
                "PowerShell error: {error}"
            )))
        }
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("pbpaste")
            .output()
            .map_err(|e| AutomationError::PlatformError(format!("Failed to run pbpaste: {}", e)))?;

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout).trim().to_string();
            debug!("📋 Clipboard content: {}", content);
            Ok(content)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(AutomationError::PlatformError(format!(
                "pbpaste error: {}",
                error
            )))
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try xclip first, then xsel as fallback
        if let Ok(output) = Command::new("xclip")
            .args(&["-selection", "clipboard", "-o"])
            .output()
        {
            if output.status.success() {
                let content = String::from_utf8_lossy(&output.stdout).trim().to_string();
                debug!("📋 Clipboard content (xclip): {}", content);
                return Ok(content);
            }
        }

        let output = Command::new("xsel")
            .args(&["--clipboard", "--output"])
            .output()
            .map_err(|e| {
                AutomationError::PlatformError(format!(
                    "Failed to get clipboard (xclip/xsel not available): {}",
                    e
                ))
            })?;

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout).trim().to_string();
            debug!("📋 Clipboard content (xsel): {}", content);
            Ok(content)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(AutomationError::PlatformError(format!(
                "xsel error: {}",
                error
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clipboard_operations() {
        // Test clipboard functionality
        let test_content = "test content";

        match set_clipboard_content(test_content).await {
            Ok(()) => println!("✅ Set clipboard successfully"),
            Err(e) => println!("❌ Set clipboard error: {}", e),
        }

        match get_clipboard_content().await {
            Ok(content) => println!("✅ Clipboard content: {}", content),
            Err(e) => println!("❌ Get clipboard error: {}", e),
        }
    }
}
