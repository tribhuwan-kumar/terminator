use crate::utils::{
    get_timeout, ActivateElementArgs, ClickElementArgs, ClipboardArgs, DesktopWrapper, EmptyArgs,
    GetApplicationsArgs, GetClipboardArgs, GetWindowTreeArgs, GetWindowsArgs, GlobalKeyArgs,
    HighlightElementArgs, LocatorArgs, MouseDragArgs, NavigateBrowserArgs, OpenApplicationArgs,
    PressKeyArgs, RunCommandArgs, ScrollElementArgs, SelectOptionArgs, SetRangeValueArgs,
    SetSelectedArgs, SetToggledArgs, TypeIntoElementArgs, ValidateElementArgs, WaitForElementArgs,
};
use chrono::Local;
use rmcp::model::{
    CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::{tool, Error as McpError, ServerHandler};
use serde_json::{json, Value};
use std::env;
use std::sync::Arc;
use terminator::{Browser, Desktop, Selector};

/// Helper function to parse comma-separated alternative selectors into a Vec<String>
fn parse_alternative_selectors(alternatives: Option<&str>) -> Vec<String> {
    alternatives
        .map(|alts| {
            alts.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Helper function to get all selectors tried (primary + alternatives) for error reporting
fn get_selectors_tried(primary: &str, alternatives: Option<&str>) -> Vec<String> {
    let mut all = vec![primary.to_string()];
    all.extend(parse_alternative_selectors(alternatives));
    all
}

#[tool(tool_box)]
impl DesktopWrapper {
    pub async fn new() -> Result<Self, McpError> {
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        let desktop = match Desktop::new(false, false) {
            Ok(d) => d,
            Err(e) => {
                return Err(McpError::internal_error(
                    "Failed to initialize terminator desktop",
                    serde_json::to_value(e.to_string()).ok(),
                ))
            }
        };

        #[cfg(target_os = "macos")]
        let desktop = match Desktop::new(true, true) {
            Ok(d) => d,
            Err(e) => {
                return Err(McpError::internal_error(
                    "Failed to initialize terminator desktop",
                    serde_json::to_value(e.to_string()).ok(),
                ))
            }
        };

        Ok(Self {
            desktop: Arc::new(desktop),
        })
    }

    #[tool(
        description = "Get the complete UI tree for an application by PID and optional window title. This is your primary tool for understanding the application's current state."
    )]
    async fn get_window_tree(
        &self,
        #[tool(param)] args: GetWindowTreeArgs,
    ) -> Result<CallToolResult, McpError> {
        let tree = self
            .desktop
            .get_window_tree(
                args.pid,
                args.title.as_deref(),
                None, // Use default config for now
            )
            .map_err(|e| {
                McpError::resource_not_found(
                    "Failed to get window tree",
                    Some(json!({"reason": e.to_string(), "pid": args.pid, "title": args.title})),
                )
            })?;

        let mut result_json = json!({
            "action": "get_window_tree",
            "status": "success",
            "pid": args.pid,
            "title": args.title,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "recommendation": "Look for element IDs (e.g., '#12345') as primary selectors. Avoid name-based selectors when IDs are available."
        });

        // Always include the tree unless explicitly disabled
        if let Ok(tree_val) = serde_json::to_value(tree) {
            if let Some(obj) = result_json.as_object_mut() {
                obj.insert("ui_tree".to_string(), tree_val);
            }
        }

        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(description = "Get all applications currently running and their state.")]
    async fn get_applications(
        &self,
        #[tool(param)] args: GetApplicationsArgs,
    ) -> Result<CallToolResult, McpError> {
        let apps = self.desktop.applications().map_err(|e| {
            McpError::resource_not_found(
                "Failed to get applications",
                Some(json!({"reason": e.to_string()})),
            )
        })?;

        let include_tree = args.include_tree.unwrap_or(false);

        let app_info_futures: Vec<_> = apps
            .iter()
            .map(|app| {
                let desktop = self.desktop.clone();
                let app_name = app.name().unwrap_or_default();
                let app_id = app.id().unwrap_or_default();
                let app_role = app.role();
                let app_pid = app.process_id().unwrap_or(0);
                let is_focused = app.is_focused().unwrap_or(false);

                tokio::spawn(async move {
                    let tree = if include_tree && app_pid > 0 {
                        desktop.get_window_tree(app_pid, None, None).ok()
                    } else {
                        None
                    };

                    json!({
                        "name": app_name,
                        "id": app_id,
                        "role": app_role,
                        "pid": app_pid,
                        "is_focused": is_focused,
                        "suggested_selector": format!("#{}", app_id),
                        "alternative_selectors": [
                            format!("#{}", app_id),
                            format!("name:{}", app_name)
                        ],
                        "ui_tree": tree.and_then(|t| serde_json::to_value(t).ok())
                    })
                })
            })
            .collect();

        let results = futures::future::join_all(app_info_futures).await;
        let app_info: Vec<Value> = results.into_iter().filter_map(Result::ok).collect();

        Ok(CallToolResult::success(vec![Content::json(json!({
            "applications": app_info,
            "count": app_info.len(),
            "recommendation": "Always prefer using ID selectors (e.g., '#12345') over name selectors for reliability. Use get_window_tree with the PID to get detailed UI structure when needed."
        }))?]))
    }

    #[tool(description = "Get windows for a specific application by name.")]
    async fn get_windows_for_application(
        &self,
        #[tool(param)] args: GetWindowsArgs,
    ) -> Result<CallToolResult, McpError> {
        let windows = self
            .desktop
            .windows_for_application(&args.app_name)
            .await
            .map_err(|e| {
                McpError::resource_not_found(
                    "Failed to get windows for application",
                    Some(json!({"reason": e.to_string()})),
                )
            })?;

        let window_info: Vec<_> = windows
            .iter()
            .map(|window| {
                json!({
                    "title": window.name().unwrap_or_default(),
                    "id": window.id().unwrap_or_default(),
                    "role": window.role(),
                    "bounds": window.bounds().map(|b| json!({
                        "x": b.0, "y": b.1, "width": b.2, "height": b.3
                    })).unwrap_or(json!(null)),
                    "suggested_selector": format!("name:{}", window.name().unwrap_or_default())
                })
            })
            .collect();

        Ok(CallToolResult::success(vec![Content::json(json!({
            "windows": window_info,
            "count": windows.len(),
            "application": args.app_name
        }))?]))
    }

    #[tool(
        description = "Types text into a UI element with smart clipboard optimization and verification. Much faster than press key."
    )]
    async fn type_into_element(
        &self,
        #[tool(param)] args: TypeIntoElementArgs,
    ) -> Result<CallToolResult, McpError> {
        use crate::utils::find_element_with_fallbacks;

        let (element, successful_selector) = find_element_with_fallbacks(
            &self.desktop,
            &args.selector,
            args.alternative_selectors.as_deref(),
            args.timeout_ms,
        )
        .await
        .map_err(|e| {
            let selectors_tried =
                get_selectors_tried(&args.selector, args.alternative_selectors.as_deref());

            McpError::internal_error(
                "Failed to locate element with any selector",
                Some(json!({
                    "reason": e.to_string(),
                    "selectors_tried": selectors_tried,
                    "timeout_used": get_timeout(args.timeout_ms).map(|d| d.as_millis())
                })),
            )
        })?;

        let pid = element.process_id().unwrap_or(0);
        let id = element.id().unwrap_or_default();

        // Get element details before typing for better feedback
        let element_info = json!({
            "name": element.name().unwrap_or_default(),
            "application": element.application_name(),
            "window": element.window_title(),
            "process_id": element.process_id().unwrap_or(0),
            "is_focused": element.is_focused().unwrap_or(false),
            "role": element.role(),
            "id": id,
            "pid": pid,
            "bounds": element.bounds().map(|b| json!({
                "x": b.0, "y": b.1, "width": b.2, "height": b.3
            })).unwrap_or(json!(null)),
            "enabled": element.is_enabled().unwrap_or(false),
            "suggested_selector": format!("#{}", id),
        });

        element.type_text(&args.text_to_type, true).map_err(|e| {
            McpError::resource_not_found(
                "Failed to type text",
                Some(json!({
                    "reason": e.to_string(),
                    "selector": args.selector,
                    "text_to_type": args.text_to_type,
                    "element_info": element_info
                })),
            )
        })?;

        let mut result_json = json!({
            "action": "type_into_element",
            "status": "success",
            "text_typed": args.text_to_type,
            "element": element_info,
            "selector_used": successful_selector,
            "selectors_tried": get_selectors_tried(&args.selector, args.alternative_selectors.as_deref()),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        // Verification if requested
        if args.verify_action.unwrap_or(true) {
            // Create a new locator for verification using the successful selector
            let verification_locator = self
                .desktop
                .locator(Selector::from(successful_selector.as_str()));
            if let Ok(updated_element) = verification_locator
                .wait(Some(std::time::Duration::from_millis(500)))
                .await
            {
                let verification = json!({
                    "element_focused": updated_element.is_focused().unwrap_or(false),
                    "element_enabled": updated_element.is_enabled().unwrap_or(false),
                    "verification_timestamp": chrono::Utc::now().to_rfc3339()
                });
                if let Some(obj) = result_json.as_object_mut() {
                    obj.insert("verification".to_string(), verification);
                }
            }
        }

        // Always attach tree for better context, or if explicitly requested
        self.maybe_attach_tree(true, Some(pid), &mut result_json);

        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(description = "Clicks a UI element.")]
    async fn click_element(
        &self,
        #[tool(param)] args: ClickElementArgs,
    ) -> Result<CallToolResult, McpError> {
        use crate::utils::find_element_with_fallbacks;

        let (element, successful_selector) = find_element_with_fallbacks(
            &self.desktop,
            &args.selector,
            args.alternative_selectors.as_deref(),
            args.timeout_ms,
        )
        .await
        .map_err(|e| {
            let selectors_tried =
                get_selectors_tried(&args.selector, args.alternative_selectors.as_deref());

            McpError::internal_error(
                "Failed to locate element with any selector",
                Some(json!({
                    "reason": e.to_string(),
                    "selectors_tried": selectors_tried,
                    "timeout_used": get_timeout(args.timeout_ms).map(|d| d.as_millis())
                })),
            )
        })?;

        // Get element details before clicking for better feedback
        let element_info = json!({
            "name": element.name().unwrap_or_default(),
            "role": element.role(),
            "id": element.id().unwrap_or_default(),
            "bounds": element.bounds().map(|b| json!({
                "x": b.0, "y": b.1, "width": b.2, "height": b.3
            })).unwrap_or(json!(null)),
            "enabled": element.is_enabled().unwrap_or(false),
        });

        element.click().map_err(|e| {
            McpError::resource_not_found(
                "Failed to click on element",
                Some(json!({
                    "reason": e.to_string(),
                    "selector": args.selector,
                    "element_info": element_info
                })),
            )
        })?;

        // Build base result
        let mut result_json = json!({
            "action": "click",
            "status": "success",
            "element": element_info,
            "selector_used": successful_selector,
            "selectors_tried": get_selectors_tried(&args.selector, args.alternative_selectors.as_deref()),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        // Always attach tree for better context
        self.maybe_attach_tree(true, element.process_id().ok(), &mut result_json);

        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(
        description = "Sends a key press to a UI element. Use curly brace format: '{Ctrl}c', '{Alt}{F4}', '{Enter}', '{PageDown}', etc."
    )]
    async fn press_key(
        &self,
        #[tool(param)] args: PressKeyArgs,
    ) -> Result<CallToolResult, McpError> {
        let locator = self.desktop.locator(Selector::from(args.selector.as_str()));
        let element = locator
            .wait(get_timeout(args.timeout_ms))
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "Failed to locate element",
                    Some(json!({"reason": e.to_string(), "selector": args.selector})),
                )
            })?;
        // Get element details before pressing key for better feedback
        let element_info = json!({
            "name": element.name().unwrap_or_default(),
            "role": element.role(),
            "id": element.id().unwrap_or_default(),
            "application": element.application_name(),
            "window_title": element.window_title(),
            "process_id": element.process_id().unwrap_or(0),
            "is_focused": element.is_focused().unwrap_or(false),
            "bounds": element.bounds().map(|b| json!({
                "x": b.0, "y": b.1, "width": b.2, "height": b.3
            })).unwrap_or(json!(null)),
            "enabled": element.is_enabled().unwrap_or(false),
        });

        element.press_key(&args.key).map_err(|e| {
            McpError::resource_not_found(
                "Failed to press key",
                Some(json!({
                    "reason": e.to_string(),
                    "selector": args.selector,
                    "key_pressed": args.key,
                    "element_info": element_info
                })),
            )
        })?;

        let mut result_json = json!({
            "action": "press_key",
            "status": "success",
            "key_pressed": args.key,
            "element": element_info,
            "selector": args.selector,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        self.maybe_attach_tree(true, element.process_id().ok(), &mut result_json);

        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(
        description = "Sends a key press to the currently focused element (no selector required). Use curly brace format: '{Ctrl}c', '{Alt}{F4}', '{Enter}', '{PageDown}', etc."
    )]
    async fn press_key_global(
        &self,
        #[tool(param)] args: GlobalKeyArgs,
    ) -> Result<CallToolResult, McpError> {
        // Identify focused element
        let focused = self.desktop.focused_element().map_err(|e| {
            McpError::internal_error(
                "Failed to get focused element",
                Some(json!({"reason": e.to_string()})),
            )
        })?;

        // Gather metadata for debugging / result payload
        let element_info = json!({
            "name": focused.name().unwrap_or_default(),
            "role": focused.role(),
            "id": focused.id().unwrap_or_default(),
            "application": focused.application_name(),
            "window_title": focused.window_title(),
            "process_id": focused.process_id().unwrap_or(0),
            "is_focused": focused.is_focused().unwrap_or(false),
            "bounds": focused.bounds().map(|b| json!({
                "x": b.0, "y": b.1, "width": b.2, "height": b.3
            })).unwrap_or(json!(null)),
            "enabled": focused.is_enabled().unwrap_or(false),
        });

        // Perform the key press
        focused.press_key(&args.key).map_err(|e| {
            McpError::resource_not_found(
                "Failed to press key on focused element",
                Some(json!({
                    "reason": e.to_string(),
                    "key_pressed": args.key,
                    "element_info": element_info
                })),
            )
        })?;

        let mut result_json = json!({
            "action": "press_key_global",
            "status": "success",
            "key_pressed": args.key,
            "element": element_info,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        self.maybe_attach_tree(true, focused.process_id().ok(), &mut result_json);

        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(description = "Executes a shell command.")]
    async fn run_command(
        &self,
        #[tool(param)] args: RunCommandArgs,
    ) -> Result<CallToolResult, McpError> {
        let output = self
            .desktop
            .run_command(
                args.windows_command.as_deref(),
                args.unix_command.as_deref(),
            )
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "Failed to run command",
                    Some(json!({"reason": e.to_string()})),
                )
            })?;

        Ok(CallToolResult::success(vec![Content::json(json!({
            "exit_status": output.exit_status,
            "stdout": output.stdout,
            "stderr": output.stderr,
        }))?]))
    }

    #[tool(
        description = "Activates the window containing the specified element, bringing it to the foreground."
    )]
    async fn activate_element(
        &self,
        #[tool(param)] args: ActivateElementArgs,
    ) -> Result<CallToolResult, McpError> {
        let locator = self.desktop.locator(Selector::from(args.selector.as_str()));
        let element = locator
            .wait(get_timeout(args.timeout_ms))
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "Failed to locate element",
                    Some(json!({"reason": e.to_string(), "selector": args.selector})),
                )
            })?;

        let pid = element.process_id().unwrap_or(0);
        let id = element.id().unwrap_or_default();

        let element_info = json!({
            "name": element.name().unwrap_or_default(),
            "role": element.role(),
            "id": id,
            "pid": pid,
            "application": element.application_name(),
            "window_title": element.window_title(),
            "is_focused": element.is_focused().unwrap_or(false),
            "bounds": element.bounds().map(|b| json!({
                "x": b.0, "y": b.1, "width": b.2, "height": b.3
            })).unwrap_or(json!(null)),
            "enabled": element.is_enabled().unwrap_or(false),
            "suggested_selector": format!("#{}", id),
        });

        element.activate_window().map_err(|e| {
            McpError::resource_not_found(
                "Failed to activate window with that element",
                Some(json!({"reason": e.to_string(), "selector": args.selector, "element_info": element_info})),
            )
        })?;

        let mut result_json = json!({
            "action": "activate_element",
            "status": "success",
            "element": element_info,
            "selector": args.selector,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "recommendation": "Window activated successfully. The UI tree is attached to help you find specific elements to interact with next."
        });

        // Always attach UI tree for activated elements to help with next actions
        self.maybe_attach_tree(
            true, // Always attach tree for activate_element since it's important for next actions
            Some(pid),
            &mut result_json,
        );

        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(
        description = "Captures a screenshot of the primary monitor and returns the recognized text content (OCR)."
    )]
    async fn capture_screen(
        &self,
        #[tool(param)] _args: EmptyArgs,
    ) -> Result<CallToolResult, McpError> {
        let monitor = self.desktop.get_primary_monitor().await.map_err(|e| {
            McpError::internal_error(
                "Failed to get primary monitor",
                Some(json!({"reason": e.to_string()})),
            )
        })?;

        let screenshot = self.desktop.capture_monitor(&monitor).await.map_err(|e| {
            McpError::internal_error(
                "Failed to capture screen",
                Some(json!({"reason": e.to_string()})),
            )
        })?;

        let ocr_text = self
            .desktop
            .ocr_screenshot(&screenshot)
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "Failed to perform OCR",
                    Some(json!({"reason": e.to_string()})),
                )
            })?;

        Ok(CallToolResult::success(vec![Content::json(&ocr_text)?]))
    }

    #[tool(description = "Sets text to the system clipboard using shell commands.")]
    async fn set_clipboard(
        &self,
        #[tool(param)] args: ClipboardArgs,
    ) -> Result<CallToolResult, McpError> {
        // todo use native clipbaord feature we implemented
        let result = if cfg!(target_os = "windows") {
            // Windows: echo "text" | clip
            let command = format!("echo \"{}\" | clip", args.text.replace("\"", "\\\""));
            self.desktop.run_command(Some(&command), None).await
        } else if cfg!(target_os = "macos") {
            // macOS: echo "text" | pbcopy
            let command = format!("echo \"{}\" | pbcopy", args.text.replace("\"", "\\\""));
            self.desktop.run_command(None, Some(&command)).await
        } else {
            // Linux: echo "text" | xclip -selection clipboard
            let command = format!(
                "echo \"{}\" | xclip -selection clipboard",
                args.text.replace("\"", "\\\"")
            );
            self.desktop.run_command(None, Some(&command)).await
        };

        result.map_err(|e| {
            McpError::internal_error(
                "Failed to set clipboard",
                Some(json!({"reason": e.to_string(), "text": args.text})),
            )
        })?;

        Ok(CallToolResult::success(vec![Content::json(json!({
            "action": "set_clipboard",
            "status": "success",
            "text": args.text,
            "method": "shell_command",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))?]))
    }

    #[tool(description = "Gets text from the system clipboard using shell commands.")]
    async fn get_clipboard(
        &self,
        #[tool(param)] _args: GetClipboardArgs,
    ) -> Result<CallToolResult, McpError> {
        let command_result = if cfg!(target_os = "windows") {
            // Windows: powershell Get-Clipboard
            self.desktop
                .run_command(Some("powershell -command \"Get-Clipboard\""), None)
                .await
        } else if cfg!(target_os = "macos") {
            // macOS: pbpaste
            self.desktop.run_command(None, Some("pbpaste")).await
        } else {
            // Linux: xclip -selection clipboard -o
            self.desktop
                .run_command(None, Some("xclip -selection clipboard -o"))
                .await
        };

        match command_result {
            Ok(output) => Ok(CallToolResult::success(vec![Content::json(json!({
                "action": "get_clipboard",
                "status": "success",
                "text": output.stdout.trim(),
                "method": "shell_command",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))?])),
            Err(e) => Err(McpError::internal_error(
                "Failed to get clipboard text",
                Some(json!({"reason": e.to_string()})),
            )),
        }
    }

    #[tool(description = "Performs a mouse drag operation from start to end coordinates.")]
    async fn mouse_drag(
        &self,
        #[tool(param)] args: MouseDragArgs,
    ) -> Result<CallToolResult, McpError> {
        let locator = self.desktop.locator(Selector::from(args.selector.as_str()));
        let element = locator
            .wait(get_timeout(args.timeout_ms))
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "Failed to locate element",
                    Some(json!({"reason": e.to_string(), "selector": args.selector})),
                )
            })?;

        // Get element details before dragging for better feedback
        let element_info = json!({
            "name": element.name().unwrap_or_default(),
            "role": element.role(),
            "id": element.id().unwrap_or_default(),
            "application": element.application_name(),
            "window_title": element.window_title(),
            "process_id": element.process_id().unwrap_or(0),
            "is_focused": element.is_focused().unwrap_or(false),
            "bounds": element.bounds().map(|b| json!({
                "x": b.0, "y": b.1, "width": b.2, "height": b.3
            })).unwrap_or(json!(null)),
            "enabled": element.is_enabled().unwrap_or(false),
        });

        element
            .mouse_drag(args.start_x, args.start_y, args.end_x, args.end_y)
            .map_err(|e| {
                McpError::resource_not_found(
                    "Failed to perform mouse drag",
                    Some(json!({
                        "reason": e.to_string(),
                        "selector": args.selector,
                        "start": (args.start_x, args.start_y),
                        "end": (args.end_x, args.end_y),
                        "element_info": element_info
                    })),
                )
            })?;

        let mut result_json = json!({
            "action": "mouse_drag",
            "status": "success",
            "element": element_info,
            "selector": args.selector,
            "start": (args.start_x, args.start_y),
            "end": (args.end_x, args.end_y),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        self.maybe_attach_tree(true, element.process_id().ok(), &mut result_json);

        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(
        description = "Validates that an element exists and provides detailed information about it."
    )]
    async fn validate_element(
        &self,
        #[tool(param)] args: ValidateElementArgs,
    ) -> Result<CallToolResult, McpError> {
        use crate::utils::find_element_with_fallbacks;

        match find_element_with_fallbacks(
            &self.desktop,
            &args.selector,
            args.alternative_selectors.as_deref(),
            args.timeout_ms,
        )
        .await
        {
            Ok((element, successful_selector)) => {
                let element_info = json!({
                    "exists": true,
                    "name": element.name().unwrap_or_default(),
                    "role": element.role(),
                    "id": element.id().unwrap_or_default(),
                    "application": element.application_name(),
                    "window_title": element.window_title(),
                    "process_id": element.process_id().unwrap_or(0),
                    "is_focused": element.is_focused().unwrap_or(false),
                    "bounds": element.bounds().map(|b| json!({
                        "x": b.0, "y": b.1, "width": b.2, "height": b.3
                    })).unwrap_or(json!(null)),
                    "enabled": element.is_enabled().unwrap_or(false),
                    "visible": element.is_visible().unwrap_or(false),
                    "focused": element.is_focused().unwrap_or(false),
                    "keyboard_focusable": element.is_keyboard_focusable().unwrap_or(false),
                    "text": element.text(1).unwrap_or_default(),
                    "value": element.attributes().value.unwrap_or_default(),
                });

                let mut result_json = json!({
                    "action": "validate_element",
                    "status": "success",
                    "element": element_info,
                    "selector_used": successful_selector,
                    "selectors_tried": get_selectors_tried(&args.selector, args.alternative_selectors.as_deref()),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                self.maybe_attach_tree(true, element.process_id().ok(), &mut result_json);

                Ok(CallToolResult::success(vec![Content::json(result_json)?]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::json(json!({
                "action": "validate_element",
                "status": "failed",
                "exists": false,
                "reason": e.to_string(),
                "selectors_tried": get_selectors_tried(&args.selector, args.alternative_selectors.as_deref()),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))?])),
        }
    }

    #[tool(description = "Highlights an element with a colored border for visual confirmation.")]
    async fn highlight_element(
        &self,
        #[tool(param)] args: HighlightElementArgs,
    ) -> Result<CallToolResult, McpError> {
        let locator = self.desktop.locator(Selector::from(args.selector.as_str()));
        let element = locator
            .wait(get_timeout(args.timeout_ms))
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "Failed to locate element for highlighting",
                    Some(json!({"reason": e.to_string(), "selector": args.selector})),
                )
            })?;

        let duration = args.duration_ms.map(std::time::Duration::from_millis);
        element.highlight(args.color, duration).map_err(|e| {
            McpError::internal_error(
                "Failed to highlight element",
                Some(json!({"reason": e.to_string(), "selector": args.selector})),
            )
        })?;

        let element_info = json!({
            "name": element.name().unwrap_or_default(),
            "role": element.role(),
            "id": element.id().unwrap_or_default(),
            "application": element.application_name(),
            "window_title": element.window_title(),
            "process_id": element.process_id().unwrap_or(0),
            "is_focused": element.is_focused().unwrap_or(false),
            "bounds": element.bounds().map(|b| json!({
                "x": b.0, "y": b.1, "width": b.2, "height": b.3
            })).unwrap_or(json!(null)),
        });

        let mut result_json = json!({
            "action": "highlight_element",
            "status": "success",
            "element": element_info,
            "selector": args.selector,
            "color": args.color.unwrap_or(0x0000FF),
            "duration_ms": args.duration_ms.unwrap_or(1000),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        self.maybe_attach_tree(true, element.process_id().ok(), &mut result_json);

        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(
        description = "Waits for an element to meet a specific condition (visible, enabled, focused, exists)."
    )]
    async fn wait_for_element(
        &self,
        #[tool(param)] args: WaitForElementArgs,
    ) -> Result<CallToolResult, McpError> {
        let locator = self.desktop.locator(Selector::from(args.selector.as_str()));
        let timeout = get_timeout(args.timeout_ms);

        let condition_lower = args.condition.to_lowercase();
        let result = match condition_lower.as_str() {
            "exists" => match locator.wait(timeout).await {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            },
            "visible" => match locator.wait(timeout).await {
                Ok(element) => element.is_visible().map_err(|e| {
                    McpError::internal_error(
                        "Failed to check visibility",
                        Some(json!({"reason": e.to_string()})),
                    )
                }),
                Err(e) => Err(McpError::internal_error(
                    "Element not found",
                    Some(json!({"reason": e.to_string()})),
                )),
            },
            "enabled" => match locator.wait(timeout).await {
                Ok(element) => element.is_enabled().map_err(|e| {
                    McpError::internal_error(
                        "Failed to check enabled state",
                        Some(json!({"reason": e.to_string()})),
                    )
                }),
                Err(e) => Err(McpError::internal_error(
                    "Element not found",
                    Some(json!({"reason": e.to_string()})),
                )),
            },
            "focused" => match locator.wait(timeout).await {
                Ok(element) => element.is_focused().map_err(|e| {
                    McpError::internal_error(
                        "Failed to check focus state",
                        Some(json!({"reason": e.to_string()})),
                    )
                }),
                Err(e) => Err(McpError::internal_error(
                    "Element not found",
                    Some(json!({"reason": e.to_string()})),
                )),
            },
            _ => Err(McpError::invalid_params(
                "Invalid condition. Valid conditions: exists, visible, enabled, focused",
                Some(json!({"provided_condition": args.condition})),
            )),
        };

        match result {
            Ok(condition_met) => Ok(CallToolResult::success(vec![Content::json(json!({
                "action": "wait_for_element",
                "status": "success",
                "condition": args.condition,
                "condition_met": condition_met,
                "selector": args.selector,
                "timeout_ms": args.timeout_ms.unwrap_or(5000),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))?])),
            Err(e) => Err(e),
        }
    }

    #[tool(
        description = "Opens a URL in the specified browser (uses SDK's built-in browser automation)."
    )]
    async fn navigate_browser(
        &self,
        #[tool(param)] args: NavigateBrowserArgs,
    ) -> Result<CallToolResult, McpError> {
        let browser = args.browser.clone().map(Browser::Custom);
        let ui_element = self.desktop.open_url(&args.url, browser).map_err(|e| {
            McpError::internal_error(
                "Failed to open URL",
                Some(json!({"reason": e.to_string(), "url": args.url, "browser": args.browser})),
            )
        })?;

        let element_info = json!({
            "name": ui_element.name().unwrap_or_default(),
            "role": ui_element.role(),
            "id": ui_element.id().unwrap_or_default(),
            "application": ui_element.application_name(),
            "window_title": ui_element.window_title(),
            "process_id": ui_element.process_id().unwrap_or(0),
            "is_focused": ui_element.is_focused().unwrap_or(false),
            "bounds": ui_element.bounds().map(|b| json!({
                "x": b.0, "y": b.1, "width": b.2, "height": b.3
            })).unwrap_or(json!(null)),
        });

        let tree = self
            .desktop
            .get_window_tree(ui_element.process_id().unwrap_or(0), None, None)
            .unwrap_or_default();

        Ok(CallToolResult::success(vec![Content::json(json!({
            "action": "navigate_browser",
            "status": "success",
            "url": args.url,
            "browser": args.browser,
            "element": element_info,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "ui_tree": tree
        }))?]))
    }

    #[tool(description = "Opens an application by name (uses SDK's built-in app launcher).")]
    async fn open_application(
        &self,
        #[tool(param)] args: OpenApplicationArgs,
    ) -> Result<CallToolResult, McpError> {
        let ui_element = self.desktop.open_application(&args.app_name).map_err(|e| {
            McpError::internal_error(
                "Failed to open application",
                Some(json!({"reason": e.to_string(), "app_name": args.app_name})),
            )
        })?;

        let process_id = ui_element.process_id().unwrap_or(0);

        let element_info = json!({
            "name": ui_element.name().unwrap_or_default(),
            "role": ui_element.role(),
            "id": ui_element.id().unwrap_or_default(),
            "application": ui_element.application_name(),
            "window_title": ui_element.window_title(),
            "process_id": process_id,
            "is_focused": ui_element.is_focused().unwrap_or(false),
            "bounds": ui_element.bounds().map(|b| json!({
                "x": b.0, "y": b.1, "width": b.2, "height": b.3
            })).unwrap_or(json!(null)),
        });

        let mut result_json = json!({
            "action": "open_application",
            "status": "success",
            "app_name": args.app_name,
            "application": element_info,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "recommendation": "Application opened successfully. Use get_window_tree with the PID to get the full UI structure for reliable element targeting."
        });

        // Always attach the full UI tree for newly opened applications
        if process_id > 0 {
            if let Ok(tree) = self.desktop.get_window_tree(process_id, None, None) {
                if let Ok(tree_val) = serde_json::to_value(tree) {
                    if let Some(obj) = result_json.as_object_mut() {
                        obj.insert("ui_tree".to_string(), tree_val);
                    }
                }
            }
        }

        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(
        description = "Closes a UI element (window, application, dialog, etc.) if it's closable."
    )]
    async fn close_element(
        &self,
        #[tool(param)] args: LocatorArgs,
    ) -> Result<CallToolResult, McpError> {
        let locator = self.desktop.locator(Selector::from(args.selector.as_str()));
        let element = locator
            .wait(get_timeout(args.timeout_ms))
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "Failed to locate element for closing",
                    Some(json!({"reason": e.to_string(), "selector": args.selector})),
                )
            })?;

        // Get element details before closing for better feedback
        let element_info = json!({
            "name": element.name().unwrap_or_default(),
            "role": element.role(),
            "id": element.id().unwrap_or_default(),
            "process_id": element.process_id().unwrap_or(0),
            "is_focused": element.is_focused().unwrap_or(false),
            "bounds": element.bounds().map(|b| json!({
                "x": b.0, "y": b.1, "width": b.2, "height": b.3
            })).unwrap_or(json!(null)),
            "application": element.application_name(),
            "window_title": element.window_title(),
        });

        element.close().map_err(|e| {
            McpError::resource_not_found(
                "Failed to close element",
                Some(json!({
                    "reason": e.to_string(),
                    "selector": args.selector,
                    "element_info": element_info
                })),
            )
        })?;

        Ok(CallToolResult::success(vec![Content::json(json!({
            "action": "close_element",
            "status": "success",
            "element": element_info,
            "selector": args.selector,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))?]))
    }

    #[tool(description = "Scrolls a UI element in the specified direction by the given amount.")]
    async fn scroll_element(
        &self,
        #[tool(param)] args: ScrollElementArgs,
    ) -> Result<CallToolResult, McpError> {
        let locator = self.desktop.locator(Selector::from(args.selector.as_str()));
        let element = locator
            .wait(get_timeout(args.timeout_ms))
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "Failed to locate element for scrolling",
                    Some(json!({"reason": e.to_string(), "selector": args.selector})),
                )
            })?;

        // Get element details before scrolling for better feedback
        let element_info = json!({
            "name": element.name().unwrap_or_default(),
            "role": element.role(),
            "id": element.id().unwrap_or_default(),
            "application": element.application_name(),
            "window_title": element.window_title(),
            "process_id": element.process_id().unwrap_or(0),
            "is_focused": element.is_focused().unwrap_or(false),
            "bounds": element.bounds().map(|b| json!({
                "x": b.0, "y": b.1, "width": b.2, "height": b.3
            })).unwrap_or(json!(null)),
        });

        element.scroll(&args.direction, args.amount).map_err(|e| {
            McpError::resource_not_found(
                "Failed to scroll element",
                Some(json!({
                    "reason": e.to_string(),
                    "selector": args.selector,
                    "direction": args.direction,
                    "amount": args.amount,
                    "element_info": element_info
                })),
            )
        })?;

        let mut result_json = json!({
            "action": "scroll_element",
            "status": "success",
            "element": element_info,
            "selector": args.selector,
            "direction": args.direction,
            "amount": args.amount,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        self.maybe_attach_tree(true, element.process_id().ok(), &mut result_json);

        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(description = "Selects an option in a dropdown or combobox by its visible text.")]
    async fn select_option(
        &self,
        #[tool(param)] args: SelectOptionArgs,
    ) -> Result<CallToolResult, McpError> {
        use crate::utils::find_element_with_fallbacks;

        let (element, successful_selector) = find_element_with_fallbacks(
            &self.desktop,
            &args.selector,
            args.alternative_selectors.as_deref(),
            args.timeout_ms,
        )
        .await
        .map_err(|e| {
            McpError::internal_error(
                "Failed to locate element",
                Some(json!({
                    "reason": e.to_string(),
                    "selectors_tried": get_selectors_tried(&args.selector, args.alternative_selectors.as_deref()),
                })),
            )
        })?;

        let element_info =
            json!({ "id": element.id(), "name": element.name(), "role": element.role() });

        element.select_option(&args.option_name).map_err(|e| {
            McpError::resource_not_found(
                "Failed to select option",
                Some(
                    json!({ "reason": e.to_string(), "selector": args.selector, "option": args.option_name }),
                ),
            )
        })?;

        let mut result_json = json!({
            "action": "select_option",
            "status": "success",
            "element": element_info,
            "selector_used": successful_selector,
            "option_selected": args.option_name,
        });
        self.maybe_attach_tree(true, element.process_id().ok(), &mut result_json);
        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(
        description = "Lists all available option strings from a dropdown, list box, or similar control."
    )]
    async fn list_options(
        &self,
        #[tool(param)] args: LocatorArgs,
    ) -> Result<CallToolResult, McpError> {
        use crate::utils::find_element_with_fallbacks;

        let (element, successful_selector) = find_element_with_fallbacks(
            &self.desktop,
            &args.selector,
            args.alternative_selectors.as_deref(),
            args.timeout_ms,
        )
        .await
        .map_err(|e| {
            McpError::internal_error(
                "Failed to locate element",
                Some(json!({
                    "reason": e.to_string(),
                    "selectors_tried": get_selectors_tried(&args.selector, args.alternative_selectors.as_deref()),
                })),
            )
        })?;

        let element_info =
            json!({ "id": element.id(), "name": element.name(), "role": element.role() });
        let options = element.list_options().map_err(|e| {
            McpError::internal_error(
                "Failed to list options",
                Some(json!({ "reason": e.to_string(), "selector": args.selector })),
            )
        })?;

        let mut result_json = json!({
            "action": "list_options",
            "status": "success",
            "element": element_info,
            "selector_used": successful_selector,
            "options": options,
            "count": options.len(),
        });
        self.maybe_attach_tree(true, element.process_id().ok(), &mut result_json);
        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(description = "Sets the state of a toggleable control (e.g., checkbox, switch).")]
    async fn set_toggled(
        &self,
        #[tool(param)] args: SetToggledArgs,
    ) -> Result<CallToolResult, McpError> {
        use crate::utils::find_element_with_fallbacks;

        let (element, successful_selector) = find_element_with_fallbacks(
            &self.desktop,
            &args.selector,
            args.alternative_selectors.as_deref(),
            args.timeout_ms,
        )
        .await
        .map_err(|e| {
            McpError::internal_error(
                "Failed to locate element",
                Some(json!({
                    "reason": e.to_string(),
                    "selectors_tried": get_selectors_tried(&args.selector, args.alternative_selectors.as_deref()),
                })),
            )
        })?;

        let element_info =
            json!({ "id": element.id(), "name": element.name(), "role": element.role() });

        element.set_toggled(args.state).map_err(|e| {
            McpError::internal_error(
                "Failed to set toggle state",
                Some(
                    json!({ "reason": e.to_string(), "selector": args.selector, "state": args.state }),
                ),
            )
        })?;

        let mut result_json = json!({
            "action": "set_toggled",
            "status": "success",
            "element": element_info,
            "selector_used": successful_selector,
            "state_set_to": args.state,
        });
        self.maybe_attach_tree(true, element.process_id().ok(), &mut result_json);
        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(description = "Sets the value of a range-based control like a slider.")]
    async fn set_range_value(
        &self,
        #[tool(param)] args: SetRangeValueArgs,
    ) -> Result<CallToolResult, McpError> {
        use crate::utils::find_element_with_fallbacks;

        let (element, successful_selector) = find_element_with_fallbacks(
            &self.desktop,
            &args.selector,
            args.alternative_selectors.as_deref(),
            args.timeout_ms,
        )
        .await
        .map_err(|e| {
            McpError::internal_error(
                "Failed to locate element",
                Some(json!({
                    "reason": e.to_string(),
                    "selectors_tried": get_selectors_tried(&args.selector, args.alternative_selectors.as_deref()),
                })),
            )
        })?;

        let element_info =
            json!({ "id": element.id(), "name": element.name(), "role": element.role() });

        element.set_range_value(args.value).map_err(|e| {
            McpError::internal_error(
                "Failed to set range value",
                Some(
                    json!({ "reason": e.to_string(), "selector": args.selector, "value": args.value }),
                ),
            )
        })?;

        let mut result_json = json!({
            "action": "set_range_value",
            "status": "success",
            "element": element_info,
            "selector_used": successful_selector,
            "value_set_to": args.value,
        });
        self.maybe_attach_tree(true, element.process_id().ok(), &mut result_json);
        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(
        description = "Sets the selection state of a selectable item (e.g., in a list or calendar)."
    )]
    async fn set_selected(
        &self,
        #[tool(param)] args: SetSelectedArgs,
    ) -> Result<CallToolResult, McpError> {
        use crate::utils::find_element_with_fallbacks;

        let (element, successful_selector) = find_element_with_fallbacks(
            &self.desktop,
            &args.selector,
            args.alternative_selectors.as_deref(),
            args.timeout_ms,
        )
        .await
        .map_err(|e| {
            McpError::internal_error(
                "Failed to locate element",
                Some(json!({
                    "reason": e.to_string(),
                    "selectors_tried": get_selectors_tried(&args.selector, args.alternative_selectors.as_deref()),
                })),
            )
        })?;

        let element_info =
            json!({ "id": element.id(), "name": element.name(), "role": element.role() });

        element.set_selected(args.state).map_err(|e| {
            McpError::internal_error(
                "Failed to set selected state",
                Some(
                    json!({ "reason": e.to_string(), "selector": args.selector, "state": args.state }),
                ),
            )
        })?;

        let mut result_json = json!({
            "action": "set_selected",
            "status": "success",
            "element": element_info,
            "selector_used": successful_selector,
            "state_set_to": args.state,
        });
        self.maybe_attach_tree(true, element.process_id().ok(), &mut result_json);
        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(
        description = "Checks if a control (like a checkbox or toggle switch) is currently toggled on."
    )]
    async fn is_toggled(
        &self,
        #[tool(param)] args: LocatorArgs,
    ) -> Result<CallToolResult, McpError> {
        use crate::utils::find_element_with_fallbacks;

        let (element, successful_selector) = find_element_with_fallbacks(
            &self.desktop,
            &args.selector,
            args.alternative_selectors.as_deref(),
            args.timeout_ms,
        )
        .await
        .map_err(|e| {
            McpError::internal_error(
                "Failed to locate element",
                Some(json!({
                    "reason": e.to_string(),
                    "selectors_tried": get_selectors_tried(&args.selector, args.alternative_selectors.as_deref()),
                })),
            )
        })?;

        let element_info =
            json!({ "id": element.id(), "name": element.name(), "role": element.role() });
        let is_toggled = element.is_toggled().map_err(|e| {
            McpError::internal_error(
                "Failed to get toggle state",
                Some(json!({ "reason": e.to_string(), "selector": args.selector })),
            )
        })?;

        let mut result_json = json!({
            "action": "is_toggled",
            "status": "success",
            "element": element_info,
            "selector_used": successful_selector,
            "is_toggled": is_toggled,
        });
        self.maybe_attach_tree(true, element.process_id().ok(), &mut result_json);
        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(
        description = "Gets the current value from a range-based control like a slider or progress bar."
    )]
    async fn get_range_value(
        &self,
        #[tool(param)] args: LocatorArgs,
    ) -> Result<CallToolResult, McpError> {
        use crate::utils::find_element_with_fallbacks;

        let (element, successful_selector) = find_element_with_fallbacks(
            &self.desktop,
            &args.selector,
            args.alternative_selectors.as_deref(),
            args.timeout_ms,
        )
        .await
        .map_err(|e| {
            McpError::internal_error(
                "Failed to locate element",
                Some(json!({
                    "reason": e.to_string(),
                    "selectors_tried": get_selectors_tried(&args.selector, args.alternative_selectors.as_deref()),
                })),
            )
        })?;

        let element_info =
            json!({ "id": element.id(), "name": element.name(), "role": element.role() });
        let value = element.get_range_value().map_err(|e| {
            McpError::internal_error(
                "Failed to get range value",
                Some(json!({ "reason": e.to_string(), "selector": args.selector })),
            )
        })?;

        let mut result_json = json!({
            "action": "get_range_value",
            "status": "success",
            "element": element_info,
            "selector_used": successful_selector,
            "value": value,
        });
        self.maybe_attach_tree(true, element.process_id().ok(), &mut result_json);
        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    #[tool(
        description = "Checks if a selectable item (e.g., in a calendar, list, or tab) is currently selected."
    )]
    async fn is_selected(
        &self,
        #[tool(param)] args: LocatorArgs,
    ) -> Result<CallToolResult, McpError> {
        use crate::utils::find_element_with_fallbacks;

        let (element, successful_selector) = find_element_with_fallbacks(
            &self.desktop,
            &args.selector,
            args.alternative_selectors.as_deref(),
            args.timeout_ms,
        )
        .await
        .map_err(|e| {
            McpError::internal_error(
                "Failed to locate element",
                Some(json!({
                    "reason": e.to_string(),
                    "selectors_tried": get_selectors_tried(&args.selector, args.alternative_selectors.as_deref()),
                })),
            )
        })?;

        let element_info =
            json!({ "id": element.id(), "name": element.name(), "role": element.role() });
        let is_selected = element.is_selected().map_err(|e| {
            McpError::internal_error(
                "Failed to get selected state",
                Some(json!({ "reason": e.to_string(), "selector": args.selector })),
            )
        })?;

        let mut result_json = json!({
            "action": "is_selected",
            "status": "success",
            "element": element_info,
            "selector_used": successful_selector,
            "is_selected": is_selected,
        });
        self.maybe_attach_tree(true, element.process_id().ok(), &mut result_json);
        Ok(CallToolResult::success(vec![Content::json(result_json)?]))
    }

    // Helper to optionally attach UI tree to response
    fn maybe_attach_tree(&self, include_tree: bool, pid_opt: Option<u32>, result_json: &mut Value) {
        if !include_tree {
            return;
        }
        if let Some(pid) = pid_opt {
            if let Ok(tree) = self.desktop.get_window_tree(pid, None, None) {
                if let Ok(tree_val) = serde_json::to_value(tree) {
                    if let Some(obj) = result_json.as_object_mut() {
                        obj.insert("ui_tree".to_string(), tree_val);
                    }
                }
            }
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for DesktopWrapper {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(get_server_instructions().to_string()),
        }
    }
}

fn get_server_instructions() -> String {
    let current_date_time = Local::now().to_string();
    let current_os = env::consts::OS;
    let current_working_dir = env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    format!(
        "
You are an AI assistant designed to control a computer desktop. Your primary goal is to understand the user's request and translate it into a sequence of tool calls to automate GUI interactions.

**Golden Rules for Robust Automation**

1.  **CHECK FOCUS FIRST:** Before any `click`, `type`, or `press_key` action, you **MUST** verify the target application `is_focused` using `get_applications`. If it's not, you **MUST** call `activate_element` before proceeding. This is the #1 way to prevent sending commands to the wrong window.

2.  **WAIT AFTER NAVIGATION:** After actions like `click_element` on a link or `navigate_browser`, the UI needs time to load. You **MUST** explicitly wait. The best method is to use `wait_for_element` targeting a known element on the new page. Do not call `get_window_tree` immediately.

3.  **VERIFY EVERY ACTION:** After every significant action, call `get_window_tree` to get fresh UI state and confirm your action had the intended effect. Do not trust a 'success' status alone.

4.  **USE IDs OVER NAMES:** When an element has an `id` in the UI tree, you **MUST** use it as the primary selector (e.g., `selector: \"#12345\"`). It is the most reliable method. Use name-based selectors as fallbacks.

**Core Workflow: Discover, then Act with ID Priority**

Your most reliable strategy is to inspect the application's UI structure *before* trying to interact with it. Never guess selectors.

1.  **Discover Running Applications:** Use `get_applications` to see what's running. This gives you the `name`, `id`, and `pid` (Process ID) for each application. Note the `suggested_selector` field which prioritizes ID selectors.

2.  **Get the UI Tree:** This is the most important step. Once you have the `pid` of your target application, call `get_window_tree` with `include_tree: true`. This returns a complete, JSON-like structure of all UI elements in that application.

3.  **Find Your Target Element in the Tree:** Parse the tree to locate the element you need. MANDATORY priority order:
    *   `id`: HIGHEST PRIORITY - This is the most reliable way to find an element. It's a unique identifier.
    *   `name`: The visible text or label of the element (e.g., \"Save\", \"File\").
    *   `role`: The type of the element (e.g., \"Button\", \"Window\", \"Edit\").

4.  **Construct Smart Selector Strategies:** You have powerful tools to create robust targeting strategies:
    *   **Primary Strategy - ID When Available:** When an element has an `id`, use it with hash prefix: `\"#12345\"`. This is the most reliable.
    *   **Multi-Selector Fallbacks:** Use the `alternative_selectors` parameter to provide 1-3 backup strategies. The system tries all selectors in parallel and uses the first successful match:
        ```json
        {{
          \"selector\": \"#12345\",
          \"alternative_selectors\": [\"name:Submit Button\", \"role:Document >> role:Button\"]
        }}
        ```
    *   **Chain Selectors for Context:** Use `>>` to chain selectors for specificity. Examples:
        - `\"role:Document >> name:Email\"` (find Email field within document content)
        - `\"role:Window >> role:Document >> role:Button\"` (find button within document, not browser chrome)
        - `\"name:Form >> role:Edit\"` (find edit field within a specific form)
    *   **Coordinate-Based Selectors for Visual Targeting:** When dealing with graphical elements that lack stable IDs or names (like canvas elements, drawing applications, or specific parts of an image), you can use coordinate-based selectors:
        - `pos:x,y`: To target a specific point on the screen for actions like `click_element`.
    *   **Intelligent Selector Design:** Consider the application context:
        - **Web browsers:** Chain with `role:Document >>` to target page content, not browser UI
        - **Forms:** Use parent containers to avoid ambiguity: `\"name:Contact Form >> name:Email\"`
        - **Complex apps:** Navigate from window → document → specific element
    *   **Smart Fallback Strategy:** Order selectors from most to least specific:
        1. Exact ID: `\"#12345\"`
        2. Contextual name: `\"role:Document >> name:Submit\"`
        3. Contextual role: `\"role:Document >> role:Button\"`
        4. Generic (last resort): `\"role:Button\"`

5.  **Interact with the Element:** Once you have a reliable `selector`, use an action tool:
    *   `click_element`: To click buttons, links, etc. For elements without stable selectors (e.g., items on a drawing canvas), favor using a position selector like `pos:x,y`.
    *   `type_into_element`: To type text into input fields.
    *   `press_key`: Sends a key press to a UI element. **Key Syntax: Use curly braces for special keys!**
    *   `activate_element`: To bring a window to the foreground.
    *   `mouse_drag`: To perform drag and drop operations.
    *   `set_clipboard`: To set text to the system clipboard.
    *   `get_clipboard`: To get text from the system clipboard.
    *   `scroll_element`: To scroll within elements like web pages, documents, or lists.

6.  **Handle Scrolling for Full Context:** When working with pages or long content, ALWAYS scroll to see all content. Use `scroll_element` to scroll pages up/down to get the full context before making decisions or extracting information.

**Important: Key Syntax for press_key Tool**
When using the press_key or press_key_global tools, use the following format:
- Key combinations: Use curly braces: {{Ctrl}}c, {{Alt}}{{F4}}, {{Ctrl}}{{Shift}}n
- Single special keys: Use curly braces: {{Enter}}, {{Tab}}, {{Escape}}, {{Delete}}, {{PageDown}}, {{PageUp}}
- Function keys: Use curly braces: {{F1}}, {{F5}}, {{F12}}
- Arrow keys: Use curly braces: {{Up}}, {{Down}}, {{Left}}, {{Right}}
- Regular text: Just type the text directly, it will be sent as individual keystrokes
- Examples: {{Ctrl}}c (copy), {{Ctrl}}v (paste), {{Alt}}{{Tab}} (switch windows), {{Win}}d (show desktop)

**Complex Web Form Automation Strategies:**

For challenging web automation tasks, apply these advanced techniques:

*   **Post-Action Verification is MANDATORY:** After every critical action (typing, clicking, selecting), verify success:
    - Check `verification` data in action responses
    - Use `validate_element` to confirm current element state
    - Re-examine UI tree to verify changes took effect

*   **Progressive Form Filling Strategy:**
    - Fill one field at a time, verify each step
    - Use `capture_screen` periodically to visually confirm progress
    - If a field fails to fill, try different role targeting (Edit vs TextArea)

*   **Handle Dynamic Elements:** For complex components (dropdowns, multi-select):
    - Try typing the value directly first (often triggers autocomplete)
    - Use key navigation: type partial match + {{Down}} + {{Enter}}
    - Fall back to clicking container then typing

*   **Required Field Detection:** Before submitting forms:
    - Verify all visible required fields have values
    - Check form validation state in UI tree
    - Handle client-side validation errors gracefully

*   **Multi-Step Process Recovery:** For complex workflows:
    - Save progress checkpoints using `get_window_tree`
    - Implement rollback strategies for failed steps
    - Break complex tasks into smaller, verifiable chunks

**Element Role Recognition Patterns:**

Different element roles require different interaction strategies:

*   `Edit` vs `TextArea`:** Both accept text input, but TextArea typically for longer content
*   `Button` vs `Link`:** Both clickable, but buttons often trigger actions, links navigate
*   `ComboBox` vs `ListBox`:** ComboBox allows typing + selection, ListBox is selection-only
*   `CheckBox` vs `RadioButton`:** CheckBox allows multiple selections, RadioButton is exclusive
*   `Document` role:** Indicates page content area - chain selectors from here for web content

**Error Recovery and Debugging:**

When automation fails, follow this diagnostic sequence:

1.  **Selector Failure:** Element not found
    - Refresh UI tree - element might have moved/changed
    - Try more generic selectors (role-based instead of name-based)
    - Check if element is inside a different parent container
    - Verify application window is active and focused

2.  **Action Failure:** Element found but action fails
    - Confirm element is enabled and visible using `validate_element`
    - Try activating the element's window first
    - Check if element requires focus before interaction
    - For form fields: clear existing content before typing new content

3.  **Verification Failure:** Action seems to work but doesn't take effect
    - Wait for UI to update (use `wait_for_element` with conditions)
    - Check for JavaScript validation or async form processing
    - Look for error messages or validation hints in UI tree
    - Retry with slightly different timing or approach

**Example Scenario:**
1.  User: \"Type hello into Notepad.\"
2.  AI: Calls `get_applications` -> Finds Notepad, gets `pid`.
3.  AI: Calls `get_window_tree` with Notepad's `pid`.
4.  AI: Looks through the tree and finds the text area element with `id: \"edit_pane\"`.
5.  AI: Calls `type_into_element` with `selector: \"#edit_pane\"` and `text_to_type: \"hello\"`.

**Playbook for Robust Automation**

To make your automation scripts more reliable and easier to debug, follow these plays:

- **Favor Shell Commands for File Operations:** When working with files and folders (e.g., in File Explorer), use `run_command` with PowerShell (on Windows) or bash commands. This is significantly more reliable than simulating UI clicks for creating folders, moving files, or deleting them.
    - *Example:* To create a directory, use `run_command` with `\"mkdir 'my-folder'\"` instead of trying to find and click the \"New folder\" button.

- **Disambiguate Between Multiple Application Windows:** It's common for an application like `explorer.exe` or a web browser to have multiple processes or windows. If `get_applications` shows multiple entries, use `get_window_tree` on each `pid` to inspect them. Check the `title` or other unique elements in the tree to identify the correct window before proceeding.

- **Validate Before You Act:** Before you call `click_element` or `type_into_element`, consider using `validate_element` first. This helps confirm that your `selector` is correct and the element is present and enabled, preventing unnecessary failures.

- **Ensure Window is Active:** If clicks or key presses are not registering, the target window may not be in the foreground. Use `activate_element` on the window or a known element within it to bring it into focus before sending interactions.

**Available Tools:**

*   `get_applications`: Lists all currently running applications and their PIDs.
*   `get_window_tree`: Retrieves the entire UI element tree for an application, given its PID. **(Your primary discovery tool)**
*   `get_windows_for_application`: Get windows for a specific application by name.
*   `click_element`: Clicks a UI element specified by its `selector`.
*   `type_into_element`: Types text into a UI element.
*   `press_key`: Sends a key press to a UI element. **Key Syntax: Use curly braces for special keys!**
*   `activate_element`: Brings the window containing the element to the foreground.
*   `close_element`: Closes a UI element (window, application, dialog, etc.) if it's closable.
*   `scroll_element`: Scrolls a UI element in specified direction (up, down, left, right) by given amount.
*   `run_command`: Executes a shell command. Use this for file operations, etc., instead of UI automation.
*   `capture_screen`: Captures the screen and performs OCR.
*   `set_clipboard`: Sets text to the system clipboard using native commands.
*   `get_clipboard`: Gets text from the system clipboard using native commands.
*   `mouse_drag`: Performs a mouse drag operation from start to end coordinates.
*   `validate_element`: Validates that an element exists and provides detailed information.
*   `highlight_element`: Highlights an element with a colored border for visual confirmation.
*   `wait_for_element`: Waits for an element to meet a specific condition (visible, enabled, focused, exists).
*   `navigate_browser`: Opens a URL in the specified browser.
*   `open_application`: Opens an application by name.
*   **New Tools for High-Level Interactions:**
    *   `select_option`: Selects an option in a dropdown by its visible text.
    *   `list_options`: Lists all available options from a dropdown or list box.
    *   `set_toggled`: Sets the state of a checkbox or switch.
    *   `is_toggled`: Checks if a checkbox or switch is on.
    *   `set_range_value`: Sets the value of a slider.
    *   `get_range_value`: Gets the current value of a slider or progress bar.
    *   `set_selected`: Sets the selection state of an item in a list or calendar.
    *   `is_selected`: Checks if an item is selected.

Contextual information:
- The current date and time is {}.
- Current operating system: {}.
- Current working directory: {}.

**Smart Decision-Making Guidelines:**

You are empowered to make intelligent decisions based on context. Use your understanding to:

*   **Analyze Element Context:** When you see multiple elements with similar names/roles, use the UI tree structure to understand their relationships and choose the most appropriate target.
*   **Adapt Selector Strategy:** If an element lacks an ID, intelligently craft chain selectors based on the application type and UI hierarchy.
*   **Detect Potential Issues:** If selectors seem ambiguous (e.g., multiple \"Edit\" roles), proactively create more specific alternatives using parent containers.
*   **Optimize for Reliability:** Consider the application context when choosing between different selector approaches - web content vs native apps have different best practices.

**Technical Rules:** 
1. **ALWAYS** set timeouts (timeout_ms: 3000 or less) to prevent hanging - never leave actions without timeouts.
2. **ALWAYS** call `get_window_tree` with `include_tree: true` to understand the UI landscape before acting.
3. **ALWAYS** verify important actions by checking the response for verification data or calling `get_window_tree` again.
4. When an element has an `id`, prefer using that ID with hash prefix as the primary selector (e.g., `#12345`).
5. Use chain selectors (`>>`) to add context and avoid targeting wrong elements (e.g., browser chrome vs page content).
6. For text input, let `type_into_element` auto-choose clipboard vs direct typing (it's smart about large text).
7. Use `press_key_global` for simple keyboard shortcuts like {{Ctrl}}c, {{Ctrl}}v, {{Enter}}, {{Tab}}.
8. Always use `highlight_element` to show the user what you are targeting.
9. Leverage `alternative_selectors` to provide robust fallback strategies - you decide what makes sense for the context.

**Verification Workflow:**
1. Get UI tree → 2. Act with timeout → 3. Check action response for verification data → 4. If unclear, get UI tree again to confirm state

**CRITICAL DEBUGGING PRINCIPLE:** When any action fails or produces unexpected results, IMMEDIATELY call `get_window_tree` again to understand the current state. The UI may have changed, and fresh context is essential for recovery.
",
        current_date_time, current_os, current_working_dir
    )
}
