use anyhow::Result;
use rmcp::{schemars, schemars::JsonSchema};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use terminator::Desktop;
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct EmptyArgs {}

fn default_desktop() -> Arc<Desktop> {
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    let desktop = Desktop::new(false, false).expect("Failed to create default desktop");
    #[cfg(target_os = "macos")]
    let desktop = Desktop::new(true, true).expect("Failed to create default desktop");
    Arc::new(desktop)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DesktopWrapper {
    #[serde(skip, default = "default_desktop")]
    pub desktop: Arc<Desktop>,
}

impl Default for DesktopWrapper {
    fn default() -> Self {
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        let desktop = Desktop::new(false, false).expect("Failed to create default desktop");
        #[cfg(target_os = "macos")]
        let desktop = Desktop::new(true, true).expect("Failed to create default desktop");

        Self {
            desktop: Arc::new(desktop),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetWindowTreeArgs {
    #[schemars(description = "Process ID of the target application")]
    pub pid: u32,
    #[schemars(description = "Optional window title filter")]
    pub title: Option<String>,
    #[schemars(
        description = "Whether to include full UI tree in the response (verbose mode). STRONGLY RECOMMENDED to set to true for detailed element discovery with IDs."
    )]
    pub include_tree: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetWindowsArgs {
    #[schemars(description = "Name of the application to get windows for")]
    pub app_name: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LocatorArgs {
    #[schemars(
        description = "A string selector to locate the element. Can be chained with ` >> `."
    )]
    pub selector: String,
    #[schemars(
        description = "Optional alternative selectors to try in parallel. The first selector that finds an element will be used."
    )]
    pub alternative_selectors: Option<String>,
    #[schemars(description = "Optional timeout in milliseconds for the action")]
    pub timeout_ms: Option<u64>,
    #[schemars(description = "Whether to include full UI tree in the response (verbose mode)")]
    pub include_tree: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TypeIntoElementArgs {
    #[schemars(
        description = "A string selector to locate the element. Can be chained with ` >> `."
    )]
    pub selector: String,
    #[schemars(
        description = "Optional alternative selectors to try in parallel. The first selector that finds an element will be used."
    )]
    pub alternative_selectors: Option<String>,
    #[schemars(description = "The text to type into the element")]
    pub text_to_type: String,
    #[schemars(description = "Optional timeout in milliseconds for the action (default: 3000ms)")]
    pub timeout_ms: Option<u64>,
    #[schemars(description = "Whether to include full UI tree in the response (verbose mode)")]
    pub include_tree: Option<bool>,
    #[schemars(description = "Whether to verify the action succeeded (default: true)")]
    pub verify_action: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PressKeyArgs {
    #[schemars(description = "The key or key combination to press (e.g., 'Enter', 'Ctrl+A')")]
    pub key: String,
    #[schemars(
        description = "A string selector to locate the element. Can be chained with ` >> `."
    )]
    pub selector: String,
    #[schemars(
        description = "Optional alternative selectors to try in parallel. The first selector that finds an element will be used."
    )]
    pub alternative_selectors: Option<String>,
    #[schemars(description = "Optional timeout in milliseconds for the action")]
    pub timeout_ms: Option<u64>,
    #[schemars(description = "Whether to include full UI tree in the response (verbose mode)")]
    pub include_tree: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GlobalKeyArgs {
    #[schemars(
        description = "The key or key combination to press (e.g., '{PageDown}', '{Ctrl}{V}')"
    )]
    pub key: String,
    #[schemars(description = "Whether to include full UI tree in the response (verbose mode)")]
    pub include_tree: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RunCommandArgs {
    #[schemars(description = "The command to run on Windows")]
    pub windows_command: Option<String>,
    #[schemars(description = "The command to run on Linux/macOS")]
    pub unix_command: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetClipboardArgs {
    #[schemars(description = "Optional timeout in milliseconds")]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MouseDragArgs {
    #[schemars(
        description = "A string selector to locate the element. Can be chained with ` >> `."
    )]
    pub selector: String,
    #[schemars(description = "Start X coordinate")]
    pub start_x: f64,
    #[schemars(description = "Start Y coordinate")]
    pub start_y: f64,
    #[schemars(description = "End X coordinate")]
    pub end_x: f64,
    #[schemars(description = "End Y coordinate")]
    pub end_y: f64,
    #[schemars(
        description = "Optional alternative selectors to try in parallel. The first selector that finds an element will be used."
    )]
    pub alternative_selectors: Option<String>,
    #[schemars(description = "Optional timeout in milliseconds")]
    pub timeout_ms: Option<u64>,
    #[schemars(description = "Whether to include full UI tree in the response (verbose mode)")]
    pub include_tree: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ValidateElementArgs {
    #[schemars(
        description = "A string selector to locate the element. Can be chained with ` >> `."
    )]
    pub selector: String,
    #[schemars(
        description = "Optional alternative selectors to try in parallel. The first selector that finds an element will be used."
    )]
    pub alternative_selectors: Option<String>,
    #[schemars(description = "Optional timeout in milliseconds")]
    pub timeout_ms: Option<u64>,
    #[schemars(description = "Whether to include full UI tree in the response (verbose mode)")]
    pub include_tree: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HighlightElementArgs {
    #[schemars(
        description = "A string selector to locate the element. Can be chained with ` >> `."
    )]
    pub selector: String,
    #[schemars(
        description = "Optional alternative selectors to try in parallel. The first selector that finds an element will be used."
    )]
    pub alternative_selectors: Option<String>,
    #[schemars(description = "BGR color code (optional, default red)")]
    pub color: Option<u32>,
    #[schemars(description = "Duration in milliseconds (optional, default 1000ms)")]
    pub duration_ms: Option<u64>,
    #[schemars(description = "Optional timeout in milliseconds")]
    pub timeout_ms: Option<u64>,
    #[schemars(description = "Whether to include full UI tree in the response (verbose mode)")]
    pub include_tree: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WaitForElementArgs {
    #[schemars(
        description = "A string selector to locate the element. Can be chained with ` >> `."
    )]
    pub selector: String,
    #[schemars(
        description = "Optional alternative selectors to try in parallel. The first selector that finds an element will be used."
    )]
    pub alternative_selectors: Option<String>,
    #[schemars(description = "Condition to wait for: 'visible', 'enabled', 'focused', 'exists'")]
    pub condition: String,
    #[schemars(description = "Optional timeout in milliseconds")]
    pub timeout_ms: Option<u64>,
    #[schemars(description = "Whether to include full UI tree in the response (verbose mode)")]
    pub include_tree: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct NavigateBrowserArgs {
    #[schemars(description = "URL to navigate to")]
    pub url: String,
    #[schemars(description = "Optional browser name")]
    pub browser: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OpenApplicationArgs {
    #[schemars(description = "Name of the application to open")]
    pub app_name: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ClipboardArgs {
    #[schemars(description = "Text to set to clipboard")]
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Arguments for scrolling an element")]
pub struct ScrollElementArgs {
    pub selector: String,
    #[schemars(
        description = "Optional alternative selectors to try in parallel. The first selector that finds an element will be used."
    )]
    pub alternative_selectors: Option<String>,
    #[serde(default)]
    #[schemars(description = "Direction to scroll: 'up', 'down', 'left', 'right'")]
    pub direction: String,
    #[serde(default)]
    #[schemars(description = "Amount to scroll (number of lines or pages)")]
    pub amount: f64,
    pub timeout_ms: Option<u64>,
    #[schemars(description = "Whether to include full UI tree in the response (verbose mode)")]
    pub include_tree: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ActivateElementArgs {
    #[schemars(
        description = "A string selector to locate the element. Can be chained with ` >> `."
    )]
    pub selector: String,
    #[schemars(description = "Optional timeout in milliseconds for the action")]
    pub timeout_ms: Option<u64>,
}

pub fn init_logging() -> Result<()> {
    let log_level = env::var("LOG_LEVEL")
        .map(|level| match level.to_lowercase().as_str() {
            "error" => Level::ERROR,
            "warn" => Level::WARN,
            "info" => Level::INFO,
            "debug" => Level::DEBUG,
            _ => Level::INFO,
        })
        .unwrap_or(Level::INFO);

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(log_level.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    Ok(())
}

pub fn get_timeout(timeout_ms: Option<u64>) -> Option<Duration> {
    // Default to 3 seconds instead of indefinite wait to prevent hanging
    let timeout = timeout_ms.unwrap_or(3000);
    Some(Duration::from_millis(timeout))
}

/// Try multiple selectors with primary selector priority
/// The primary selector is always preferred if it succeeds, even if alternatives also succeed
pub async fn find_element_with_fallbacks(
    desktop: &Desktop,
    primary_selector: &str,
    alternative_selectors: Option<&str>,
    timeout_ms: Option<u64>,
) -> Result<(terminator::UIElement, String), terminator::AutomationError> {
    use tokio::time::Duration;

    let timeout_duration = get_timeout(timeout_ms).unwrap_or(Duration::from_millis(3000));

    // Parse comma-separated alternative selectors
    let alternative_selectors_vec: Option<Vec<String>> = alternative_selectors.map(|alts| {
        alts.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    });

    // Create primary task
    let desktop_clone = desktop.clone();
    let primary_clone = primary_selector.to_string();
    let primary_task = tokio::spawn(async move {
        let locator = desktop_clone.locator(terminator::Selector::from(primary_clone.as_str()));
        match locator.first(Some(timeout_duration)).await {
            Ok(element) => Ok((element, primary_clone)),
            Err(e) => Err((primary_clone, e)),
        }
    });

    // Create alternative tasks
    let mut alternative_tasks = Vec::new();
    if let Some(alternatives) = alternative_selectors_vec.as_ref() {
        for selector_str in alternatives {
            let desktop_clone = desktop.clone();
            let selector_clone = selector_str.clone();
            let task = tokio::spawn(async move {
                let locator =
                    desktop_clone.locator(terminator::Selector::from(selector_clone.as_str()));
                match locator.first(Some(timeout_duration)).await {
                    Ok(element) => Ok((element, selector_clone)),
                    Err(e) => Err((selector_clone, e)),
                }
            });
            alternative_tasks.push(task);
        }
    }

    // Wait for primary task first, then alternatives
    let mut errors = Vec::new();
    let mut completed_tasks = Vec::new();
    completed_tasks.push(primary_task);
    completed_tasks.extend(alternative_tasks);

    // Use select_all but prioritize primary selector if multiple succeed
    while !completed_tasks.is_empty() {
        let (result, index, remaining_tasks) = futures::future::select_all(completed_tasks).await;

        match result {
            Ok(Ok((element, selector))) => {
                // Cancel remaining tasks
                for task in remaining_tasks {
                    task.abort();
                }

                // Always prefer primary selector (index 0) if it succeeds
                if index == 0 {
                    return Ok((element, selector));
                } else {
                    // Alternative succeeded first, but give primary selector a brief grace period
                    // in case it's about to succeed too (within 10ms)
                    let desktop_clone = desktop.clone();
                    let primary_clone = primary_selector.to_string();

                    match tokio::time::timeout(Duration::from_millis(10), async move {
                        let locator = desktop_clone
                            .locator(terminator::Selector::from(primary_clone.as_str()));
                        locator.first(Some(Duration::from_millis(1))).await
                    })
                    .await
                    {
                        Ok(Ok(primary_element)) => {
                            // Primary also succeeded within grace period - prefer it
                            return Ok((primary_element, primary_selector.to_string()));
                        }
                        _ => {
                            // Primary didn't succeed quickly, use the alternative that worked
                            return Ok((element, selector));
                        }
                    }
                }
            }
            Ok(Err((selector, error))) => {
                errors.push(format!("'{}': {}", selector, error));
                completed_tasks = remaining_tasks;
            }
            Err(join_error) => {
                errors.push(format!("Task error: {}", join_error));
                completed_tasks = remaining_tasks;
            }
        }
    }

    // All selectors failed
    let combined_error = if errors.is_empty() {
        "No selectors provided".to_string()
    } else {
        format!(
            "All {} selectors failed: [{}]",
            errors.len(),
            errors.join(", ")
        )
    };

    Err(terminator::AutomationError::ElementNotFound(combined_error))
}
