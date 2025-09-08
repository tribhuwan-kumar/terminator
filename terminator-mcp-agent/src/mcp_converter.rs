use crate::workflow_events::{
    ClickEvent, EnhancedUIElement, McpToolStep, TextInputCompletedEvent, WorkflowEvent,
};
use anyhow::Result;
use serde_json::json;
use tracing::{debug, warn};

/// Configuration for MCP conversion behavior
#[derive(Debug, Clone)]
pub struct ConversionConfig {
    /// Whether to include MCP conversion during recording
    pub enable_mcp_conversion: bool,
    /// Whether to detect UI patterns during recording
    pub enable_pattern_detection: bool,
    /// Maximum number of fallback strategies to generate
    pub max_fallback_strategies: usize,
    /// Whether to validate generated sequences during recording
    pub validate_during_recording: bool,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            enable_mcp_conversion: true,
            enable_pattern_detection: true,
            max_fallback_strategies: 3,
            validate_during_recording: false, // Expensive, off by default
        }
    }
}

/// Result of converting a workflow event to MCP sequences
#[derive(Debug, Clone)]
pub struct ConversionResult {
    /// Primary MCP tool sequence
    pub primary_sequence: Vec<McpToolStep>,
    /// Semantic action description
    pub semantic_action: String,
    /// Alternative sequences as fallbacks
    pub fallback_sequences: Vec<Vec<McpToolStep>>,
    /// Analysis notes for debugging
    pub conversion_notes: Vec<String>,
}

/// Converts workflow events into MCP-compatible tool sequences
#[derive(Clone)]
pub struct McpConverter {
    config: ConversionConfig,
    /// Track the last known window context for fallback
    last_window_context: std::sync::Arc<std::sync::Mutex<Option<(String, String, String)>>>,
}

impl Default for McpConverter {
    fn default() -> Self {
        Self::with_config(ConversionConfig::default())
    }
}

impl McpConverter {
    /// Create a new MCP converter with default configuration
    pub fn new() -> Self {
        Self::with_config(ConversionConfig::default())
    }

    /// Create a new MCP converter with custom configuration
    pub fn with_config(config: ConversionConfig) -> Self {
        Self {
            config,
            last_window_context: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Get the last known window context for fallback
    fn get_last_window_context(&self) -> Option<(String, String, String)> {
        self.last_window_context.lock().ok()?.clone()
    }

    /// Update the last known window context
    fn update_last_window_context(&self, context: Option<(String, String, String)>) {
        if let Ok(mut last) = self.last_window_context.lock() {
            *last = context;
        }
    }

    /// Convert a workflow event to MCP sequences
    pub async fn convert_event(
        &self,
        event: &WorkflowEvent,
        ui_context: Option<&EnhancedUIElement>,
    ) -> Result<ConversionResult> {
        if !self.config.enable_mcp_conversion {
            return Ok(ConversionResult {
                primary_sequence: vec![],
                semantic_action: "disabled".to_string(),
                fallback_sequences: vec![],
                conversion_notes: vec!["MCP conversion disabled".to_string()],
            });
        }

        debug!("Converting workflow event to MCP sequence: {:?}", event);

        let result = match event {
            WorkflowEvent::TextInputCompleted(text_event) => {
                self.convert_text_input(text_event, ui_context).await
            }
            WorkflowEvent::Click(click_event) => self.convert_click(click_event, ui_context).await,
            WorkflowEvent::ApplicationSwitch(app_event) => {
                self.convert_application_switch(app_event).await
            }
            WorkflowEvent::BrowserTabNavigation(nav_event) => {
                self.convert_browser_navigation(nav_event).await
            }
            WorkflowEvent::Mouse(mouse_event)
                if mouse_event.event_type == crate::workflow_events::MouseEventType::Wheel =>
            {
                self.convert_scroll(mouse_event).await
            }
            WorkflowEvent::Hotkey(hotkey_event) => self.convert_hotkey(hotkey_event).await,
            WorkflowEvent::Clipboard(clipboard_event) => {
                self.convert_clipboard(clipboard_event).await
            }
            // Add other event types as needed
            _ => {
                warn!("MCP conversion not implemented for event type: {:?}", event);
                Ok(ConversionResult {
                    primary_sequence: vec![],
                    semantic_action: "unsupported".to_string(),
                    fallback_sequences: vec![],
                    conversion_notes: vec![
                        "Event type not supported for MCP conversion".to_string()
                    ],
                })
            }
        }?;

        // Apply validation to all generated selectors
        let mut result = result;
        for step in &mut result.primary_sequence {
            if let Some(selector) = step.arguments.get_mut("selector") {
                if let Some(selector_str) = selector.as_str() {
                    let validated = self.validate_selector(selector_str);
                    if validated != selector_str {
                        debug!("Validated selector: '{}' -> '{}'", selector_str, validated);
                    }
                    *selector = json!(validated);
                }
            }
        }

        // Also validate fallback sequences
        for fallback_seq in &mut result.fallback_sequences {
            for step in fallback_seq {
                if let Some(selector) = step.arguments.get_mut("selector") {
                    if let Some(selector_str) = selector.as_str() {
                        let validated = self.validate_selector(selector_str);
                        *selector = json!(validated);
                    }
                }
            }
        }

        // Update last known window context from clicks and application switches
        match event {
            WorkflowEvent::Click(click_event) => {
                if let Some(metadata) = &click_event.metadata.ui_element {
                    if let Ok(serialized) = serde_json::to_value(metadata) {
                        let app = serialized
                            .get("application")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        if !app.is_empty() {
                            let window_role = "Window".to_string();
                            self.update_last_window_context(Some((
                                app.to_string(),
                                app.to_string(),
                                window_role,
                            )));
                        }
                    }
                }
            }
            WorkflowEvent::ApplicationSwitch(app_event) => {
                // Store window context from app switch
                self.update_last_window_context(Some((
                    app_event.to_application.clone(),
                    app_event.to_application.clone(),
                    "Window".to_string(),
                )));
            }
            _ => {}
        }

        Ok(result)
    }

    /// Convert text input event to MCP sequence
    async fn convert_text_input(
        &self,
        event: &TextInputCompletedEvent,
        ui_context: Option<&EnhancedUIElement>,
    ) -> Result<ConversionResult> {
        let mut notes = Vec::new();

        // Analyze input method to determine conversion strategy
        let conversion_strategy = match event.input_method {
            crate::workflow_events::TextInputMethod::Suggestion => {
                notes.push("Detected suggestion-based input".to_string());
                self.convert_suggestion_input(event, ui_context).await?
            }
            crate::workflow_events::TextInputMethod::Typed => {
                notes.push("Detected typed input".to_string());
                self.convert_typed_input(event, ui_context).await?
            }
            crate::workflow_events::TextInputMethod::Pasted => {
                notes.push("Detected pasted input".to_string());
                self.convert_pasted_input(event, ui_context).await?
            }
            _ => {
                notes.push("Using fallback conversion for mixed/unknown input method".to_string());
                self.convert_fallback_input(event, ui_context).await?
            }
        };

        Ok(ConversionResult {
            primary_sequence: conversion_strategy.sequence,
            semantic_action: conversion_strategy.semantic_action,
            fallback_sequences: conversion_strategy.fallbacks,
            conversion_notes: notes,
        })
    }

    /// Convert click event to MCP sequence
    async fn convert_click(
        &self,
        event: &ClickEvent,
        ui_context: Option<&EnhancedUIElement>,
    ) -> Result<ConversionResult> {
        let mut sequence = Vec::new();
        let mut notes = Vec::new();

        // Extract application/window context for scoped selector generation
        let window_context = if let Some(metadata) = &event.metadata.ui_element {
            // Try to get the serialized application field directly if it's a SerializableUIElement
            // Otherwise fall back to the UIElement methods
            let (app_name, window_title, window_role) =
                if let Ok(serialized) = serde_json::to_value(metadata) {
                    // We have a serialized form - extract fields directly
                    let app = serialized
                        .get("application")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let title = serialized
                        .get("window_title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let role = serialized
                        .get("role")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Window");
                    (app.to_string(), title.to_string(), role.to_string())
                } else {
                    // Fall back to UIElement methods (for live elements)
                    let app_name = metadata.application_name();
                    let window_title = metadata.window_title();
                    let window_role = metadata
                        .window()
                        .ok()
                        .flatten()
                        .map(|w| w.role())
                        .unwrap_or_else(|| "Window".to_string());
                    (app_name, window_title, window_role)
                };

            // Use the application name directly as window context if available
            let effective_window_title = if !app_name.is_empty() {
                app_name.clone()
            } else if !window_title.is_empty() {
                window_title.clone()
            } else {
                String::new()
            };

            let effective_app_name = if app_name.contains("Chrome") {
                "Google Chrome"
            } else if app_name.contains("Firefox") {
                "Firefox"
            } else if app_name.contains("Edge") {
                "Microsoft Edge"
            } else if !app_name.is_empty() {
                &app_name
            } else {
                "Application"
            };

            tracing::info!("🔍 MCP Converter - app: '{}', effective_title: '{}', window_role: '{}', clicked_element_role: '{}'", 
                          effective_app_name, effective_window_title, window_role, metadata.role());

            // Always provide window context if we have ANY application info
            if !effective_window_title.is_empty() {
                Some((
                    effective_app_name.to_string(),
                    effective_window_title,
                    window_role,
                ))
            } else {
                // Use last known window context as fallback
                notes.push(
                    "No window context found in element, using last known context".to_string(),
                );
                self.get_last_window_context()
            }
        } else {
            // No metadata at all - use last known window context
            notes.push("No UI element metadata, using last known context".to_string());
            self.get_last_window_context()
        };

        // NEW: Check if this is a click-away action to dismiss UI elements
        if self.is_click_away_action(event) {
            tracing::info!("🔄 Detected click-away action - converting to Escape key press");

            // Generate escape key step instead of click
            let escape_step = self.generate_escape_key_step();
            sequence.push(escape_step);

            notes.push(format!("Converted click-away action to Escape key press - detected container click: role='{}', children={}", 
                event.element_role, event.child_text_content.len()));

            tracing::info!("✅ Generated Escape key press for click-away dismissal");

            return Ok(ConversionResult {
                semantic_action: "dismiss_ui".to_string(),
                primary_sequence: sequence,
                fallback_sequences: vec![],
                conversion_notes: notes,
            });
        }

        // Generate scoped selector for the element using >> operator
        let selector = if let Some(context) = ui_context {
            notes.push("Using enhanced UI context for selector generation".to_string());
            context
                .suggested_selectors
                .first()
                .cloned()
                .unwrap_or_else(|| self.generate_scoped_selector(event, &window_context))
        } else {
            notes.push("Using scoped selector generation from event data".to_string());
            self.generate_scoped_selector(event, &window_context)
        };

        // Add note about scoped selector usage
        if window_context.is_some() {
            notes
                .push("Generated scoped selector using >> operator for window context".to_string());
        } else {
            notes.push("WARNING: No window context available for scoping selector".to_string());
        }

        // Store window context if available
        if let Some(ref ctx) = window_context {
            self.update_last_window_context(Some(ctx.clone()));
        }

        // Create the click step with 3000ms timeout as requested
        // Build arguments with optional position
        let mut arguments = json!({
            "selector": selector,
            "timeout_ms": 3000
        });

        // Add click position if available
        if let Some((x_ratio, y_ratio)) = event.relative_position {
            let x_percent = (x_ratio * 100.0).round() as u32;
            let y_percent = (y_ratio * 100.0).round() as u32;

            arguments["click_position"] = json!({
                "x_percentage": x_percent,
                "y_percentage": y_percent
            });

            notes.push(format!(
                "Click position captured: {x_percent}% x {y_percent}% within element"
            ));
        }

        sequence.push(McpToolStep {
            tool_name: "click_element".to_string(),
            arguments,
            description: Some(format!(
                "Click '{}' element{}",
                if !event.element_text.is_empty() {
                    &event.element_text
                } else if !event.child_text_content.is_empty() {
                    &event.child_text_content[0]
                } else {
                    &event.element_role
                },
                if let Some((x_ratio, y_ratio)) = event.relative_position {
                    format!(
                        " at {}%,{}%",
                        (x_ratio * 100.0).round() as u32,
                        (y_ratio * 100.0).round() as u32
                    )
                } else {
                    String::new()
                }
            )),
            timeout_ms: Some(3000),
            continue_on_error: Some(false),
            delay_ms: Some(200),
        });

        // No fallback sequences as requested
        Ok(ConversionResult {
            primary_sequence: sequence,
            semantic_action: "element_click".to_string(),
            fallback_sequences: vec![], // No fallbacks as requested
            conversion_notes: notes,
        })
    }

    /// Convert application switch event to MCP sequence
    async fn convert_application_switch(
        &self,
        event: &crate::workflow_events::ApplicationSwitchEvent,
    ) -> Result<ConversionResult> {
        let mut sequence = Vec::new();
        let mut notes = Vec::new();

        // Store this as the new window context for subsequent clicks
        self.update_last_window_context(Some((
            event.to_application.clone(),
            event.to_application.clone(),
            "Window".to_string(),
        )));
        notes.push(format!(
            "Updated window context to: {}",
            event.to_application
        ));

        // Generate stable fallback selector for common applications
        let fallback_selector = self.generate_stable_fallback_selector(&event.to_application);

        // Generate selector with proper role: prefix for application switching
        let selector = format!("role:Window|name:contains:{}", event.to_application);

        let mut arguments = json!({
            "selector": selector,
            "timeout_ms": 800,
            "retries": 0
        });

        // Add fallback selector if we generated one
        if let Some(fallback) = fallback_selector {
            arguments["fallback_selectors"] = json!(fallback);
            notes.push(format!("Added stable fallback selector: {fallback}"));
        }

        sequence.push(McpToolStep {
            tool_name: "activate_element".to_string(),
            arguments,
            description: Some(format!("Switch to application: {}", event.to_application)),
            timeout_ms: Some(800),
            continue_on_error: Some(false),
            delay_ms: Some(150), // Reduced from 1000ms since server already waits 500ms for verification
        });

        notes.push(format!(
            "Application switch method: {:?}",
            event.switch_method
        ));
        notes.push("Optimized for speed: timeout=800ms, retries=0".to_string());

        Ok(ConversionResult {
            primary_sequence: sequence,
            semantic_action: "application_switch".to_string(),
            fallback_sequences: vec![],
            conversion_notes: notes,
        })
    }

    /// Convert browser navigation event to MCP sequence
    async fn convert_browser_navigation(
        &self,
        event: &crate::workflow_events::BrowserTabNavigationEvent,
    ) -> Result<ConversionResult> {
        let mut sequence = Vec::new();
        let mut notes = Vec::new();

        if let Some(url) = &event.to_url {
            sequence.push(McpToolStep {
                tool_name: "navigate_browser".to_string(),
                arguments: json!({
                    "url": url
                }),
                description: Some(format!("Navigate to URL: {url}")),
                timeout_ms: Some(10000),
                continue_on_error: Some(false),
                delay_ms: Some(1000),
            });
            notes.push(format!("Browser navigation to: {url}"));
        }

        Ok(ConversionResult {
            primary_sequence: sequence,
            semantic_action: "browser_navigation".to_string(),
            fallback_sequences: vec![],
            conversion_notes: notes,
        })
    }

    /// Convert scroll event to MCP sequence
    async fn convert_scroll(
        &self,
        event: &crate::workflow_events::MouseEvent,
    ) -> Result<ConversionResult> {
        let mut sequence = Vec::new();
        let mut notes = Vec::new();

        if let Some((_, delta_y)) = event.scroll_delta {
            // Only handle vertical scroll, ignore horizontal for simplicity
            let direction = if delta_y > 0 { "down" } else { "up" };
            let amount = (delta_y.abs() as f64 / 120.0).max(1.0); // 120 = standard wheel notch

            // Generate selector based on captured UI element if available
            let selector = if let Some(ui_element) = &event.metadata.ui_element {
                // Try to generate a proper selector from the UI element
                let element_name = ui_element.name().unwrap_or_default();
                let element_role = ui_element.role();

                if !element_role.is_empty() {
                    if !element_name.is_empty() && element_name.len() > 2 {
                        // Use role and name for more specific targeting
                        format!("role:{element_role}|name:contains:{element_name}")
                    } else {
                        // Use just role if no meaningful name
                        format!("role:{element_role}")
                    }
                } else {
                    // Fallback to Window if no role available
                    "role:Window".to_string()
                }
            } else {
                // No UI element captured, use default
                "role:Window".to_string()
            };

            sequence.push(McpToolStep {
                tool_name: "scroll_element".to_string(),
                arguments: json!({
                    "selector": selector.clone(),
                    "direction": direction,
                    "amount": amount,
                    "timeout_ms": 2000
                }),
                description: Some(format!("Scroll {direction} by {amount:.1} units")),
                timeout_ms: Some(2000),
                continue_on_error: Some(true), // Scrolling can be non-critical
                delay_ms: Some(100),
            });

            notes.push(format!(
                "Converted scroll event: {direction} by {amount:.1} on {selector}"
            ));
        }

        Ok(ConversionResult {
            primary_sequence: sequence,
            semantic_action: "scroll".to_string(),
            fallback_sequences: vec![],
            conversion_notes: notes,
        })
    }

    /// Detect if this click is a "click away" action to dismiss UI elements
    fn is_click_away_action(&self, event: &ClickEvent) -> bool {
        // Non-clickable container roles that are typically used for layout, not interaction
        const NON_CLICKABLE_ROLES: &[&str] = &[
            "custom",        // Generic containers
            "document",      // Page content areas
            "group",         // Layout containers
            "main",          // Main content areas
            "section",       // Content sections
            "article",       // Article containers
            "div",           // Generic divs
            "region",        // ARIA regions
            "banner",        // Header areas
            "contentinfo",   // Footer areas
            "complementary", // Sidebar areas
            "generic",       // Generic elements
            "pane",          // Content panes
            "client",        // Client areas
        ];

        // Generic container names that suggest layout rather than interaction
        const GENERIC_NAMES: &[&str] = &[
            "home",
            "main",
            "content",
            "container",
            "page",
            "body",
            "wrapper",
            "layout",
            "section",
            "area",
            "panel",
            "view",
            "canvas",
            "workspace",
        ];

        // Check if role suggests a non-interactive container
        let is_non_clickable_role =
            NON_CLICKABLE_ROLES.contains(&event.element_role.to_lowercase().as_str());

        // Check if element has many children (strong indicator of layout container)
        let has_many_children = event.child_text_content.len() >= 8;

        // Check if element name suggests a generic container
        let has_generic_name = GENERIC_NAMES
            .iter()
            .any(|&name| event.element_text.to_lowercase().contains(name));

        // Future enhancement: Check bounds information to detect large containers
        // let is_large_container = if let Some(metadata) = &event.metadata.ui_element {
        //     // This would require accessing bounds from UIElement
        //     false
        // } else {
        //     false
        // };

        // Classify as click-away if it's a non-clickable role AND has container characteristics
        is_non_clickable_role && (has_many_children || has_generic_name)
    }

    /// Generate escape key press for dismissing UI elements
    fn generate_escape_key_step(&self) -> McpToolStep {
        McpToolStep {
            tool_name: "press_key".to_string(),
            arguments: json!({
                "key": "Escape",
                "timeout_ms": 1000
            }),
            description: Some("Press Escape to dismiss dropdown/modal/overlay".to_string()),
            timeout_ms: Some(1000),
            continue_on_error: Some(false),
            delay_ms: Some(100),
        }
    }

    /// Generate primary selector for element clicks - prefers child text when more specific
    #[allow(dead_code)] // TODO: Will be used for enhanced selector generation
    fn generate_primary_selector(&self, event: &ClickEvent) -> String {
        // Check for desktop context first
        if let Some(metadata) = &event.metadata.ui_element {
            let app_name = metadata.application_name();
            let window_title = metadata.window_title();

            if self.is_desktop_context(&app_name, &window_title) {
                // Desktop-specific selector generation - use standard format
                if !event.child_text_content.is_empty() {
                    return format!(
                        "role:{}|name:{}",
                        event.element_role, event.child_text_content[0]
                    );
                } else if !event.element_text.is_empty() {
                    return format!("role:{}|name:{}", event.element_role, event.element_text);
                } else {
                    return format!("role:{}", event.element_role);
                }
            }
        }

        // Regular application selector generation - trust the deepest element finder's result
        // Since our deepest element finder already performed coordinate checking,
        // we should use the actual clicked element's text, not child text from elements
        // that may not be under the click coordinates.

        if !event.element_text.is_empty() {
            // Use the actual clicked element's text with proper format
            format!("role:{}|name:{}", event.element_role, event.element_text)
        } else {
            // If the clicked element has no text, try child text as a fallback
            // (but only if we have child text and it's not from a large container)
            if !event.child_text_content.is_empty() && event.child_text_content.len() < 5 {
                let child_text = &event.child_text_content[0];
                // Only use child text if it's concise and specific
                if child_text.len() < 50 && !child_text.to_lowercase().contains("click") {
                    format!("role:{}|name:{}", event.element_role, child_text)
                } else {
                    // Child text is too verbose, use role-only selector
                    format!("role:{}", event.element_role)
                }
            } else {
                // No usable text, use role-only selector
                format!("role:{}", event.element_role)
            }
        }
    }

    /// Generate scoped selector using >> operator for better targeting
    fn generate_scoped_selector(
        &self,
        event: &ClickEvent,
        window_context: &Option<(String, String, String)>,
    ) -> String {
        // If we have window context, use scoped selector with >> operator
        if let Some((app_name, window_title, window_role)) = window_context {
            let window_selector =
                self.generate_window_selector(app_name, window_title, window_role);
            let element_selector = self.generate_element_selector(event);

            // Generate scoped selector: window >> element
            format!("{window_selector} >> {element_selector}")
        } else {
            // Fallback to basic selector if no window context
            self.generate_element_selector(event)
        }
    }

    /// Generate window selector for scoped search using actual detected role
    fn generate_window_selector(
        &self,
        app_name: &str,
        window_title: &str,
        window_role: &str,
    ) -> String {
        // Desktop-specific window selector
        if self.is_desktop_context(app_name, window_title) {
            return format!("role:{window_role}|name:Desktop");
        }

        // CHROME-SPECIFIC FIX: Override detected role for Chrome applications
        // Chrome applications should use "Pane" selectors even if window() returns "Window"
        let role = if app_name.to_lowercase().contains("chrome") {
            tracing::info!(
                "🎯 Chrome detected - using role:Pane instead of role:{}",
                window_role
            );
            "Pane"
        } else if window_role.is_empty() {
            "Window"
        } else {
            window_role
        };

        // Extract meaningful title part from window title
        if let Some(title_part) = self.extract_meaningful_title(window_title) {
            format!("role:{role}|name:contains:{title_part}")
        } else {
            // App name-based window selector for regular applications
            match app_name.to_lowercase().as_str() {
                name if name.contains("chrome") => format!("role:{role}|name:contains:Chrome"),
                name if name.contains("firefox") => format!("role:{role}|name:contains:Firefox"),
                name if name.contains("edge") => format!("role:{role}|name:contains:Edge"),
                _ => format!("role:{role}|name:contains:{app_name}"),
            }
        }
    }

    /// Generate element selector part for scoped search
    fn generate_element_selector(&self, event: &ClickEvent) -> String {
        // If element has text, use it directly
        if !event.element_text.is_empty() {
            return format!("role:{}|name:{}", event.element_role, event.element_text);
        }

        // If element has no text but has children with text
        if event.element_text.is_empty() && !event.child_text_content.is_empty() {
            // Check if this is a container role (group, pane, etc.)
            let is_container = matches!(
                event.element_role.to_lowercase().as_str(),
                "group" | "pane" | "custom" | "region" | "section" | "document" | "client"
            );

            if is_container {
                // For containers, create a parent>>child selector instead of using child text as parent name
                let child_text = &event.child_text_content[0];
                if child_text.len() < 50 {
                    // Try to create a more specific selector that will actually work
                    // Option 1: If we know it's likely a Text element child
                    if event.child_text_content.len() == 1 {
                        tracing::info!(
                            "📝 Container '{}' has no name, using parent>>child selector for child text: '{}'",
                            event.element_role, child_text
                        );
                        return format!(
                            "role:{} >> role:Text|name:{}",
                            event.element_role, child_text
                        );
                    } else {
                        // Multiple children, use text selector
                        return format!("role:{} >> text:{}", event.element_role, child_text);
                    }
                }
            } else {
                // For non-containers, we might still be able to use child text
                // but only if it makes sense for the element type
                let child_text = &event.child_text_content[0];
                if child_text.len() < 50 && !child_text.to_lowercase().contains("click") {
                    tracing::info!(
                        "📝 Non-container '{}' using child text as name: '{}'",
                        event.element_role,
                        child_text
                    );
                    return format!("role:{}|name:{}", event.element_role, child_text);
                }
            }
        }

        // Check if we have an element ID we can use as last resort
        if let Some(metadata) = &event.metadata.ui_element {
            // Try to get ID from the UIElement
            // Note: This requires the UIElement to have an id() method
            if let Ok(serialized) = serde_json::to_value(metadata) {
                if let Some(id) = serialized.get("id").and_then(|v| v.as_str()) {
                    if !id.is_empty() {
                        tracing::info!("📝 Using element ID as selector fallback: #{}", id);
                        return format!("#{id}");
                    }
                }
            }
        }

        // Add position hint for wide elements (likely table rows or containers)
        let base_selector = format!("role:{}", event.element_role);

        if let Some((x_ratio, _y_ratio)) = event.relative_position {
            // Check if element is wide (likely a table row or container)
            if let Some(metadata) = &event.metadata.ui_element {
                if let Ok(bounds) = metadata.bounds() {
                    if bounds.2 > 800.0 {
                        // Width > 800px suggests a wide element
                        // Add position hint to selector
                        let x_percent = (x_ratio * 100.0) as u32;
                        tracing::info!(
                            "📍 Adding position hint to selector: {}% across element (width: {})",
                            x_percent,
                            bounds.2
                        );
                        return format!("{base_selector}|x:{x_percent}%");
                    }
                }
            }
        }

        // Fallback to role-only selector
        tracing::warn!(
            "⚠️ Generating role-only selector for element with no identifiable content: role:{}",
            event.element_role
        );
        base_selector
    }

    /// Generate activation step for window/application targeting
    #[allow(dead_code)] // TODO: Will be used for application switching
    fn generate_activation_step(
        &self,
        app_name: &str,
        window_title: &str,
        window_role: &str,
    ) -> McpToolStep {
        let selector = self.generate_activation_selector(app_name, window_title, window_role);

        McpToolStep {
            tool_name: "activate_element".to_string(),
            arguments: json!({
                "selector": selector,
                "timeout_ms": 2000
            }),
            description: Some(format!("Activate {app_name} window")),
            timeout_ms: Some(2000),
            continue_on_error: Some(false),
            delay_ms: Some(100),
        }
    }

    /// Generate stable fallback selector for common applications
    pub fn generate_stable_fallback_selector(&self, app_name: &str) -> Option<String> {
        let app_lower = app_name.to_lowercase();

        // Map common applications to stable window selectors
        if app_lower.contains("chrome") || app_lower.contains("google chrome") {
            Some("role:Window|name:contains:Google Chrome".to_string())
        } else if app_lower.contains("firefox") {
            Some("role:Window|name:contains:Firefox".to_string())
        } else if app_lower.contains("edge") || app_lower.contains("microsoft edge") {
            Some("role:Window|name:contains:Microsoft Edge".to_string())
        } else if app_lower.contains("notepad") {
            Some("role:Window|name:contains:Notepad".to_string())
        } else if app_lower.contains("calculator") {
            Some("role:Window|name:contains:Calculator".to_string())
        } else if app_lower.contains("cursor") {
            Some("role:Window|name:contains:Cursor".to_string())
        } else if app_lower.contains("visual studio code") || app_lower.contains("vscode") {
            Some("role:Window|name:contains:Visual Studio Code".to_string())
        } else if app_lower.contains("explorer") || app_lower.contains("file explorer") {
            Some("role:Window|name:contains:File Explorer".to_string())
        } else if app_lower.contains("cmd") || app_lower.contains("command prompt") {
            Some("role:Window|name:contains:Command Prompt".to_string())
        } else if app_lower.contains("powershell") {
            Some("role:Window|name:contains:PowerShell".to_string())
        } else {
            // For unknown apps, generate a generic window selector using the app name
            // Strip common suffixes and use contains for flexibility
            let clean_name = app_name
                .replace(" - ", " ")
                .replace(".exe", "")
                .trim()
                .to_string();

            if clean_name.len() > 3 {
                Some(format!("role:Window|name:contains:{clean_name}"))
            } else {
                None
            }
        }
    }

    /// Generate activation selector based on app name and window title with actual role
    #[allow(dead_code)] // TODO: Will be used for application switching
    fn generate_activation_selector(
        &self,
        app_name: &str,
        window_title: &str,
        window_role: &str,
    ) -> String {
        // Desktop-specific activation
        if self.is_desktop_context(app_name, window_title) {
            return format!("role:{window_role}|name:Desktop");
        }

        // Use the ACTUAL detected role instead of hardcoding "Window"
        let role = if window_role.is_empty() {
            "Window"
        } else {
            window_role
        };

        // Extract meaningful title part from window title
        if let Some(title_part) = self.extract_meaningful_title(window_title) {
            format!("role:{role}|name:contains:{title_part}")
        } else {
            // App name-based activation for regular applications
            match app_name.to_lowercase().as_str() {
                name if name.contains("chrome") => format!("role:{role}|name:contains:Chrome"),
                name if name.contains("firefox") => format!("role:{role}|name:contains:Firefox"),
                name if name.contains("edge") => format!("role:{role}|name:contains:Edge"),
                _ => format!("role:{role}|name:{app_name}"),
            }
        }
    }

    /// Extract meaningful title part from full window title
    fn extract_meaningful_title(&self, full_title: &str) -> Option<String> {
        // Split on common patterns: " - ", " – ", " | "
        let separators = [" - ", " – ", " | "];

        for separator in &separators {
            if let Some(title_part) = full_title.split(separator).next() {
                let trimmed = title_part.trim();
                // Only use if it's meaningful (more than 3 chars and not generic)
                if trimmed.len() > 3
                    && !trimmed.to_lowercase().contains("new tab")
                    && !trimmed.to_lowercase().contains("untitled")
                {
                    return Some(trimmed.to_string());
                }
            }
        }
        None
    }

    /// Detect if the click is happening in desktop context
    fn is_desktop_context(&self, app_name: &str, window_title: &str) -> bool {
        let app_lower = app_name.to_lowercase();
        let title_lower = window_title.to_lowercase();

        (app_lower.contains("explorer") && title_lower.contains("desktop")) ||
        app_lower.contains("dwm") ||           // Desktop Window Manager
        app_lower.contains("shell") ||         // Windows Shell  
        title_lower == "desktop" ||            // Direct desktop window
        app_lower.contains("progman") // Program Manager (desktop)
    }

    /// Generate fallback sequences for element clicks
    #[allow(dead_code)] // TODO: Will be used for robust click fallback strategies
    async fn generate_click_fallbacks(
        &self,
        event: &ClickEvent,
        _ui_context: Option<&EnhancedUIElement>,
    ) -> Result<Vec<Vec<McpToolStep>>> {
        let mut fallbacks = Vec::new();

        // Fallback 1: Name-only selector (parent element text)
        if !event.element_text.is_empty() {
            fallbacks.push(vec![McpToolStep {
                tool_name: "click_element".to_string(),
                arguments: json!({
                    "selector": format!("name:{}", event.element_text)
                }),
                description: Some(format!("Click element by name: {}", event.element_text)),
                timeout_ms: Some(5000),
                continue_on_error: Some(false),
                delay_ms: Some(200),
            }]);
        }

        // Fallback 2: Position-based click for wide elements (NEW!)
        if let Some((x_ratio, y_ratio)) = event.relative_position {
            // Check if this is a wide element
            if let Some(metadata) = &event.metadata.ui_element {
                if let Ok(bounds) = metadata.bounds() {
                    if bounds.2 > 800.0 {
                        // Wide element
                        let x_percent = (x_ratio * 100.0) as u32;
                        let y_percent = (y_ratio * 100.0) as u32;

                        // Create a position-aware click as fallback
                        fallbacks.push(vec![McpToolStep {
                            tool_name: "click_element".to_string(),
                            arguments: json!({
                                "selector": format!("role:{}|name:{}", 
                                    event.element_role,
                                    event.child_text_content.first().unwrap_or(&event.element_text)),
                                "click_position": {
                                    "x_percentage": x_percent,
                                    "y_percentage": y_percent
                                }
                            }),
                            description: Some(format!(
                                "Click at {x_percent}%,{y_percent}% within element"
                            )),
                            timeout_ms: Some(5000),
                            continue_on_error: Some(false),
                            delay_ms: Some(200),
                        }]);
                    }
                }
            }
        }

        // Fallback 3: Child text-based selectors (EXISTING)
        for (i, child_text) in event.child_text_content.iter().enumerate() {
            if !child_text.is_empty() {
                // Child text with role selector
                fallbacks.push(vec![McpToolStep {
                    tool_name: "click_element".to_string(),
                    arguments: json!({
                        "selector": format!("role:{}|name:{}", event.element_role, child_text)
                    }),
                    description: Some(format!(
                        "Click {} containing '{}'",
                        event.element_role, child_text
                    )),
                    timeout_ms: Some(5000),
                    continue_on_error: Some(false),
                    delay_ms: Some(200),
                }]);

                // Child text with contains selector for broader matching
                fallbacks.push(vec![McpToolStep {
                    tool_name: "click_element".to_string(),
                    arguments: json!({
                        "selector": format!("contains:{}", child_text)
                    }),
                    description: Some(format!("Click element containing text: {child_text}")),
                    timeout_ms: Some(5000),
                    continue_on_error: Some(false),
                    delay_ms: Some(200),
                }]);

                // Limit to first 2 child texts to avoid too many fallbacks
                if i >= 1 {
                    break;
                }
            }
        }

        // Removed overly broad role-only fallback for safety
        // (Previously: format!("role:{}", event.element_role) - too generic)

        Ok(fallbacks)
    }
}

/// Internal strategy result for text input conversion
struct TextInputStrategy {
    sequence: Vec<McpToolStep>,
    semantic_action: String,
    fallbacks: Vec<Vec<McpToolStep>>,
}

impl McpConverter {
    /// Convert suggestion-based text input (dropdown/autocomplete)
    async fn convert_suggestion_input(
        &self,
        event: &TextInputCompletedEvent,
        ui_context: Option<&EnhancedUIElement>,
    ) -> Result<TextInputStrategy> {
        let mut sequence = Vec::new();
        let mut notes = Vec::new();

        // Analyze UI pattern if context is available
        if let Some(context) = ui_context {
            if context.interaction_context.ui_pattern == "dropdown" {
                notes.push("Detected dropdown pattern".to_string());
                return self.generate_dropdown_sequence(event, context).await;
            } else if context.interaction_context.ui_pattern == "autocomplete" {
                notes.push("Detected autocomplete pattern".to_string());
                return self.generate_autocomplete_sequence(event, context).await;
            }
        }

        // Fallback: simple menu selection
        sequence.push(McpToolStep {
            tool_name: "click_element".to_string(),
            arguments: json!({
                "selector": format!("role:MenuItem|name:{}", event.text_value)
            }),
            description: Some(format!("Select '{}' from menu", event.text_value)),
            timeout_ms: Some(5000),
            continue_on_error: Some(false),
            delay_ms: Some(300),
        });

        Ok(TextInputStrategy {
            sequence,
            semantic_action: "menu_selection".to_string(),
            fallbacks: vec![],
        })
    }

    /// Convert typed text input
    async fn convert_typed_input(
        &self,
        event: &TextInputCompletedEvent,
        ui_context: Option<&EnhancedUIElement>,
    ) -> Result<TextInputStrategy> {
        let mut sequence = Vec::new();

        // Generate selector for the input field
        let selector = if let Some(context) = ui_context {
            context
                .suggested_selectors
                .first()
                .cloned()
                .unwrap_or_else(|| self.generate_text_field_selector(event))
        } else {
            self.generate_text_field_selector(event)
        };

        // Type the text directly - the field should already be focused from previous user actions
        sequence.push(McpToolStep {
            tool_name: "type_into_element".to_string(),
            arguments: json!({
                "selector": selector,
                "text_to_type": event.text_value,
                "clear_before_typing": true
            }),
            description: Some(format!("Type '{}' into field", event.text_value)),
            timeout_ms: Some(5000),
            continue_on_error: Some(false),
            delay_ms: Some(200),
        });

        Ok(TextInputStrategy {
            sequence,
            semantic_action: "text_input".to_string(),
            fallbacks: vec![],
        })
    }

    /// Convert pasted text input
    async fn convert_pasted_input(
        &self,
        event: &TextInputCompletedEvent,
        ui_context: Option<&EnhancedUIElement>,
    ) -> Result<TextInputStrategy> {
        // Similar to typed input but potentially faster/different method
        self.convert_typed_input(event, ui_context).await
    }

    /// Convert fallback text input
    async fn convert_fallback_input(
        &self,
        event: &TextInputCompletedEvent,
        ui_context: Option<&EnhancedUIElement>,
    ) -> Result<TextInputStrategy> {
        // Use typed input as fallback
        self.convert_typed_input(event, ui_context).await
    }

    /// Generate dropdown interaction sequence
    async fn generate_dropdown_sequence(
        &self,
        event: &TextInputCompletedEvent,
        context: &EnhancedUIElement,
    ) -> Result<TextInputStrategy> {
        let mut sequence = Vec::new();

        // Step 1: Click dropdown trigger
        if let Some(trigger_selector) = self.find_dropdown_trigger(context) {
            sequence.push(McpToolStep {
                tool_name: "click_element".to_string(),
                arguments: json!({
                    "selector": trigger_selector
                }),
                description: Some("Open dropdown menu".to_string()),
                timeout_ms: Some(5000),
                continue_on_error: Some(false),
                delay_ms: Some(500),
            });
        }

        // Step 2: Select from dropdown
        let item_selector = format!("role:MenuItem|name:{}", event.text_value);
        sequence.push(McpToolStep {
            tool_name: "click_element".to_string(),
            arguments: json!({
                "selector": item_selector
            }),
            description: Some(format!("Select '{}' from dropdown", event.text_value)),
            timeout_ms: Some(5000),
            continue_on_error: Some(false),
            delay_ms: Some(200),
        });

        Ok(TextInputStrategy {
            sequence,
            semantic_action: "select_from_dropdown".to_string(),
            fallbacks: vec![],
        })
    }

    /// Generate autocomplete interaction sequence
    async fn generate_autocomplete_sequence(
        &self,
        event: &TextInputCompletedEvent,
        context: &EnhancedUIElement,
    ) -> Result<TextInputStrategy> {
        let mut sequence = Vec::new();

        // For autocomplete, we might need to type partial text then select
        let selector = context
            .suggested_selectors
            .first()
            .cloned()
            .unwrap_or_else(|| self.generate_text_field_selector(event));

        // Type to trigger autocomplete - field should already be focused
        let partial_text = if event.text_value.len() > 3 {
            &event.text_value[..3] // Type first 3 characters
        } else {
            &event.text_value
        };

        sequence.push(McpToolStep {
            tool_name: "type_into_element".to_string(),
            arguments: json!({
                "selector": selector,
                "text_to_type": partial_text,
                "clear_before_typing": true
            }),
            description: Some(format!("Type '{partial_text}' to trigger autocomplete")),
            timeout_ms: Some(3000),
            continue_on_error: Some(false),
            delay_ms: Some(500),
        });

        // Step 3: Select from autocomplete suggestions
        sequence.push(McpToolStep {
            tool_name: "click_element".to_string(),
            arguments: json!({
                "selector": format!("role:ListItem|name:{}", event.text_value)
            }),
            description: Some(format!("Select '{}' from autocomplete", event.text_value)),
            timeout_ms: Some(5000),
            continue_on_error: Some(false),
            delay_ms: Some(200),
        });

        Ok(TextInputStrategy {
            sequence,
            semantic_action: "autocomplete_selection".to_string(),
            fallbacks: vec![],
        })
    }

    /// Find dropdown trigger element
    fn find_dropdown_trigger(&self, context: &EnhancedUIElement) -> Option<String> {
        // Look for related elements that might be dropdown triggers
        for related in &context.interaction_context.related_elements {
            if related.role == "TabItem"
                && related
                    .name
                    .as_ref()
                    .is_some_and(|name| name.contains("expand"))
            {
                return related.suggested_selectors.first().cloned();
            }
            if related.role == "Button"
                && related
                    .name
                    .as_ref()
                    .is_some_and(|name| name.contains("dropdown") || name.contains("▼"))
            {
                return related.suggested_selectors.first().cloned();
            }
        }
        None
    }

    /// Generate selector for text input field
    fn generate_text_field_selector(&self, event: &TextInputCompletedEvent) -> String {
        if let Some(field_name) = &event.field_name {
            if !field_name.is_empty() {
                return format!("role:{}|name:{}", event.field_type, field_name);
            }
        }
        format!("role:{}", event.field_type)
    }

    /// Convert hotkey event to MCP sequence
    async fn convert_hotkey(
        &self,
        event: &crate::workflow_events::HotkeyEvent,
    ) -> Result<ConversionResult> {
        let mut sequence = Vec::new();
        let mut notes = Vec::new();

        // Log the original hotkey combination
        tracing::info!(
            "Converting hotkey: {} -> {:?}",
            event.combination,
            event.action
        );
        notes.push(format!(
            "Hotkey event: {} ({})",
            event.combination,
            event
                .action
                .as_ref()
                .unwrap_or(&"Unknown action".to_string())
        ));

        // Convert the hotkey combination to MCP format
        let mcp_key = self.convert_hotkey_format(&event.combination, event.action.as_deref());

        // Create press_key step
        sequence.push(McpToolStep {
            tool_name: "press_key".to_string(),
            arguments: json!({
                "key": mcp_key
            }),
            description: Some(format!(
                "Press hotkey: {}",
                event.action.as_ref().unwrap_or(&event.combination)
            )),
            timeout_ms: Some(1000),
            continue_on_error: Some(false),
            delay_ms: Some(100),
        });

        notes.push(format!(
            "Converted '{}' to MCP format: '{}'",
            event.combination, mcp_key
        ));

        Ok(ConversionResult {
            primary_sequence: sequence,
            semantic_action: format!(
                "hotkey_{}",
                event
                    .action
                    .as_ref()
                    .unwrap_or(&"custom".to_string())
                    .to_lowercase()
                    .replace(' ', "_")
            ),
            fallback_sequences: vec![],
            conversion_notes: notes,
        })
    }

    /// Convert hotkey format from recorder to MCP format
    fn convert_hotkey_format(&self, combination: &str, action: Option<&str>) -> String {
        // Handle common action-based mappings first
        match action {
            Some("Copy") => return "{Ctrl}c".to_string(),
            Some("Paste") => return "{Ctrl}v".to_string(),
            Some("Cut") => return "{Ctrl}x".to_string(),
            Some("Undo") => return "{Ctrl}z".to_string(),
            Some("Redo") => return "{Ctrl}y".to_string(),
            Some("Save") => return "{Ctrl}s".to_string(),
            Some("Select All") => return "{Ctrl}a".to_string(),
            Some("Alt+Tab") => return "{Alt}{Tab}".to_string(),
            _ => {}
        }

        // Parse the combination string (e.g., "[162, 67]" or "Ctrl+C" format)
        if combination.starts_with('[') {
            // Handle raw key code format "[162, 67]"
            // 162 = Ctrl, 67 = C
            // This is a fallback for when we get raw keycodes
            if combination.contains("162") && combination.contains("67") {
                return "{Ctrl}c".to_string();
            } else if combination.contains("162") && combination.contains("86") {
                return "{Ctrl}v".to_string();
            } else if combination.contains("162") && combination.contains("88") {
                return "{Ctrl}x".to_string();
            } else if combination.contains("162") && combination.contains("90") {
                return "{Ctrl}z".to_string();
            } else if combination.contains("162") && combination.contains("89") {
                return "{Ctrl}y".to_string();
            } else if combination.contains("162") && combination.contains("83") {
                return "{Ctrl}s".to_string();
            } else if combination.contains("162") && combination.contains("65") {
                return "{Ctrl}a".to_string();
            } else if combination.contains("18") && combination.contains("9") {
                return "{Alt}{Tab}".to_string();
            }
            // If we can't parse it, return a comment
            return format!("{{Unknown: {combination}}}");
        }

        // Handle string format "Ctrl+C", "Alt+Tab", etc.
        let mut result = String::new();
        let parts: Vec<&str> = combination.split('+').collect();

        for (i, part) in parts.iter().enumerate() {
            let lower = part.to_lowercase();
            let key = match lower.as_str() {
                "ctrl" | "control" => "{Ctrl}".to_string(),
                "alt" => "{Alt}".to_string(),
                "shift" => "{Shift}".to_string(),
                "win" | "windows" | "meta" | "cmd" => "{Win}".to_string(),
                "tab" => "{Tab}".to_string(),
                "enter" | "return" => "{Enter}".to_string(),
                "esc" | "escape" => "{Escape}".to_string(),
                "space" => "{Space}".to_string(),
                "backspace" => "{Backspace}".to_string(),
                "delete" | "del" => "{Delete}".to_string(),
                "home" => "{Home}".to_string(),
                "end" => "{End}".to_string(),
                "pageup" | "pgup" => "{PageUp}".to_string(),
                "pagedown" | "pgdn" => "{PageDown}".to_string(),
                "up" => "{Up}".to_string(),
                "down" => "{Down}".to_string(),
                "left" => "{Left}".to_string(),
                "right" => "{Right}".to_string(),
                "f1" => "{F1}".to_string(),
                "f2" => "{F2}".to_string(),
                "f3" => "{F3}".to_string(),
                "f4" => "{F4}".to_string(),
                "f5" => "{F5}".to_string(),
                "f6" => "{F6}".to_string(),
                "f7" => "{F7}".to_string(),
                "f8" => "{F8}".to_string(),
                "f9" => "{F9}".to_string(),
                "f10" => "{F10}".to_string(),
                "f11" => "{F11}".to_string(),
                "f12" => "{F12}".to_string(),
                _ => {
                    // For regular keys, only wrap in braces if it's a modifier or special key
                    if i < parts.len() - 1 {
                        // This is likely a modifier we didn't recognize
                        format!("{{{part}}}")
                    } else {
                        // This is the actual key being pressed (single character)
                        lower
                    }
                }
            };
            result.push_str(&key);
        }

        result
    }

    /// Validate and fix selector format to ensure proper "role:" prefix
    fn validate_selector(&self, selector: &str) -> String {
        // Fix invalid "application|" prefix
        if selector.starts_with("application|") {
            let title = selector.strip_prefix("application|").unwrap_or("");
            let is_browser =
                title.contains("Chrome") || title.contains("Edge") || title.contains("Firefox");
            if is_browser {
                return format!(
                    "role:TabItem|name:contains:{}",
                    title.split(" - ").next().unwrap_or(title)
                );
            } else {
                return format!("role:Window|name:contains:{title}");
            }
        }

        // Fix missing "role:" prefix for standard role|name selectors
        if selector.contains('|')
            && !selector.starts_with("role:")
            && !selector.starts_with("text:")
            && !selector.starts_with("name:")
            && !selector.starts_with("#")
        {
            let parts: Vec<&str> = selector.split('|').collect();
            if parts.len() == 2 {
                // Check if first part looks like a role (starts with uppercase or common roles)
                let potential_role = parts[0];
                if potential_role
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_uppercase())
                    || [
                        "button", "edit", "menuitem", "listitem", "window", "pane", "tabitem",
                    ]
                    .iter()
                    .any(|&r| potential_role.to_lowercase() == r)
                {
                    return format!("role:{}|name:{}", parts[0], parts[1]);
                }
            }
        }

        // Fix "desktop:" prefix - convert to standard format
        if selector.starts_with("desktop:") {
            let rest = selector.strip_prefix("desktop:").unwrap_or("");
            if rest.contains('|') {
                let parts: Vec<&str> = rest.split('|').collect();
                if parts.len() == 2 {
                    return format!("role:{}|name:{}", parts[0], parts[1]);
                }
            } else if rest.starts_with("role:") {
                return rest.to_string();
            } else {
                return format!("role:{rest}");
            }
        }

        selector.to_string()
    }

    /// Convert clipboard event to MCP sequence
    ///
    /// Since MCP doesn't have direct clipboard manipulation tools, we:
    /// 1. Store clipboard content as metadata
    /// 2. For paste operations, potentially use type_into_element
    /// 3. Track clipboard state for context
    async fn convert_clipboard(
        &self,
        event: &crate::workflow_events::ClipboardEvent,
    ) -> Result<ConversionResult> {
        let mut notes = Vec::new();
        let sequence = Vec::new();

        // Log the clipboard event details
        tracing::info!(
            "Converting clipboard event: {:?} action with {} bytes of content",
            event.action,
            event.content_size.unwrap_or(0)
        );

        let content_preview = event.content.as_ref().map(|c| {
            if c.len() > 100 {
                format!("{}...", &c[..100])
            } else {
                c.clone()
            }
        });

        match event.action {
            crate::workflow_events::ClipboardAction::Copy => {
                // Copy is typically handled by the preceding Ctrl+C hotkey
                // We just track what was copied for context
                notes.push(format!(
                    "Clipboard copy detected: {} bytes",
                    event.content_size.unwrap_or(0)
                ));

                if let Some(preview) = &content_preview {
                    notes.push(format!("Copied content: '{preview}'"));
                    // Store this for potential future paste operations
                    // In a real implementation, we'd maintain clipboard state
                }
            }
            crate::workflow_events::ClipboardAction::Paste => {
                // Paste can be implemented as typing the clipboard content
                notes.push("Clipboard paste detected".to_string());

                if let Some(content) = &event.content {
                    if !event.truncated {
                        // Only create a type step if we have the full content
                        // and it's reasonable to type
                        if content.len() <= 5000 {
                            // Reasonable limit for typing
                            // Note: In a real scenario, we'd need to know the target element
                            // For now, we'll create a placeholder that shows the intent
                            notes.push(format!(
                                "Paste operation could be replayed as typing: {} chars",
                                content.len()
                            ));

                            // Store metadata about the paste for future reference
                            if let Some(preview) = &content_preview {
                                notes.push(format!("Pasted text: '{preview}'"));
                            }
                        } else {
                            notes.push(
                                "Paste content too large for direct typing replay".to_string(),
                            );
                        }
                    } else {
                        notes.push("Paste content was truncated in recording".to_string());
                    }
                } else {
                    notes.push("Paste detected but content not captured".to_string());
                }
            }
            crate::workflow_events::ClipboardAction::Cut => {
                // Cut is like copy but also deletes the original
                notes.push(format!(
                    "Clipboard cut detected: {} bytes",
                    event.content_size.unwrap_or(0)
                ));

                if let Some(preview) = &content_preview {
                    notes.push(format!("Cut content: '{preview}'"));
                }
            }
            crate::workflow_events::ClipboardAction::Clear => {
                notes.push("Clipboard cleared".to_string());
            }
        }

        // Since we don't have direct clipboard tools in MCP,
        // we return an empty sequence but with rich metadata
        // The hotkey events (Ctrl+C, Ctrl+V) will handle the actual operations

        Ok(ConversionResult {
            primary_sequence: sequence,
            semantic_action: format!("clipboard_{}", format!("{:?}", event.action).to_lowercase()),
            fallback_sequences: vec![],
            conversion_notes: notes,
        })
    }
}
