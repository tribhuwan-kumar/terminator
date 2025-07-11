use crate::expression_eval;
use crate::utils::ToolCall;
use regex::Regex;
use rmcp::Error as McpError;
use serde_json::{json, Value};
use std::time::Duration;
use terminator::{Desktop, Selector, UIElement}; // NEW: import expression evaluator

/// Helper function to parse comma-separated alternative selectors into a Vec<String>
pub fn parse_alternative_selectors(alternatives: Option<&str>) -> Vec<String> {
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
pub fn get_selectors_tried(primary: &str, alternatives: Option<&str>) -> Vec<String> {
    let mut all = vec![primary.to_string()];
    all.extend(parse_alternative_selectors(alternatives));
    all
}

/// Returns all selectors tried, including primary, alternatives, and fallback selectors.
pub fn get_selectors_tried_all(
    primary: &str,
    alternatives: Option<&str>,
    fallback: Option<&str>,
) -> Vec<String> {
    let mut all = get_selectors_tried(primary, alternatives);
    all.extend(parse_alternative_selectors(fallback));
    all
}

/// Builds a standardized JSON object with detailed information about a UIElement.
/// This includes a suggested selector that prioritizes role|name over just the ID.
pub fn build_element_info(element: &UIElement) -> Value {
    let id = element.id().unwrap_or_default();
    let role = element.role();
    let name = element.name().unwrap_or_default();

    let suggested_selector = if !name.is_empty() && role != "Unknown" {
        format!("{}|{}", &role, &name)
    } else {
        format!("#{id}")
    };

    json!({
        "name": name,
        "role": role,
        "id": id,
        "suggested_selector": suggested_selector,
        "application": element.application_name(),
        "window_title": element.window_title(),
        "process_id": element.process_id().unwrap_or(0),
        "is_focused": element.is_focused().unwrap_or(false),
        "text": element.text(0).unwrap_or_default(),
        "bounds": element.bounds().map(|b| json!({
            "x": b.0, "y": b.1, "width": b.2, "height": b.3
        })).unwrap_or(json!(null)),
        "enabled": element.is_enabled().unwrap_or(false),
        "is_selected": element.is_selected().unwrap_or(false),
        "is_toggled": element.is_toggled().unwrap_or(false),
        "keyboard_focusable": element.is_keyboard_focusable().unwrap_or(false),
    })
}

/// Builds a standardized, actionable error when an element cannot be found.
pub fn build_element_not_found_error(
    primary_selector: &str,
    alternatives: Option<&str>,
    fallback: Option<&str>,
    original_error: anyhow::Error,
) -> McpError {
    let selectors_tried = get_selectors_tried_all(primary_selector, alternatives, fallback);
    let error_payload = json!({
        "error_type": "ElementNotFound",
        "message": format!("The specified element could not be found after trying all selectors. Original error: {}", original_error),
        "selectors_tried": selectors_tried,
        "suggestions": [
            "Call `get_window_tree` again to get a fresh view of the UI; it might have changed.",
            "Verify the element's 'name' and 'role' in the new UI tree. The 'name' attribute might be empty or different from the visible text.",
            "If the element has no 'name', use its numeric ID selector (e.g., '#12345'). This is required for many clickable 'Group' elements.",
            "Use `validate_element` with your selectors to debug existence issues before calling an action tool."
        ]
    });

    McpError::invalid_params("Element not found", Some(error_payload))
}

/// Substitutes `{{variable}}` placeholders in a JSON value.
pub fn substitute_variables(args: &mut Value, variables: &Value) {
    match args {
        Value::Object(map) => {
            for (_, value) in map {
                substitute_variables(value, variables);
            }
        }
        Value::Array(arr) => {
            for value in arr {
                substitute_variables(value, variables);
            }
        }
        Value::String(s) => {
            // This regex finds all occurrences of {{ ... }} capturing anything until the next }}
            let re = Regex::new(r"\{\{([^}]+)\}\}").unwrap(); // UPDATED PATTERN TO ALLOW EXPRESSIONS

            // Handle full string replacement first, e.g., args is "{{my_var}}" or an expression like {{contains(list, 'A')}}
            if let Some(caps) = re.captures(s) {
                if caps.get(0).unwrap().as_str() == s {
                    let expr = caps.get(1).unwrap().as_str().trim();

                    // Try simple variable replacement first (identifier characters only)
                    let is_simple_var = expr
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.');
                    if is_simple_var {
                        let pointer = format!("/{}", expr.replace('.', "/"));
                        if let Some(replacement_val) = variables.pointer(&pointer) {
                            *args = replacement_val.clone();
                            return;
                        }
                    }

                    // Fallback: attempt to evaluate expression (returns bool)
                    let eval_result = expression_eval::evaluate(expr, variables);
                    *args = Value::Bool(eval_result);
                    return; // Done after full replacement
                }
            }

            // Handle partial replacement within a larger string
            let new_s = re
                .replace_all(s, |caps: &regex::Captures| {
                    let expr = caps.get(1).unwrap().as_str().trim();
                    let pointer = format!("/{}", expr.replace('.', "/"));
                    if let Some(val) = variables.pointer(&pointer) {
                        if val.is_string() {
                            val.as_str().unwrap().to_string()
                        } else {
                            val.to_string()
                        }
                    } else {
                        // Attempt expression evaluation and convert bool to string
                        let bool_val = expression_eval::evaluate(expr, variables);
                        bool_val.to_string()
                    }
                })
                .to_string();

            *s = new_s;
        }
        _ => {} // Other types are left as is
    }
}

/// Waits for a detectable UI change after an action, like an element disappearing or focus shifting.
/// This is more efficient than a fixed sleep, as it returns as soon as a change is detected.
pub async fn wait_for_ui_change(
    desktop: &Desktop,
    original_element_id: &str,
    timeout: Duration,
) -> String {
    let start = tokio::time::Instant::now();

    // If the element has no unique ID, we cannot reliably track it.
    // In this case, we fall back to a brief, fixed delay.
    if original_element_id.is_empty() {
        tokio::time::sleep(Duration::from_millis(150)).await;
        return "untracked_element_clicked_fixed_delay".to_string();
    }

    let original_selector = Selector::from(format!("#{original_element_id}").as_str());

    while start.elapsed() < timeout {
        // Check 1: Did focus change? This is often the quickest indicator.
        if let Ok(focused_element) = desktop.focused_element() {
            if focused_element.id_or_empty() != original_element_id {
                return format!("focus_changed_to: #{}", focused_element.id_or_empty());
            }
        }

        // Check 2: Did the original element disappear? (e.g., a dialog closed)
        if desktop
            .locator(original_selector.clone())
            .first(Some(Duration::from_millis(20)))
            .await
            .is_err()
        {
            return "element_disappeared".to_string();
        }

        // Yield to the scheduler and wait before the next poll.
        tokio::time::sleep(Duration::from_millis(30)).await;
    }

    "no_significant_change_detected".to_string()
}

// Helper methods for export_workflow_sequence
pub fn generate_step_description(tool_name: &str, args: &Value) -> String {
    match tool_name {
        "click_element" => format!(
            "Click on element: {}",
            args.get("selector")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
        ),
        "type_into_element" => format!(
            "Type '{}' into {}",
            args.get("text_to_type")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            args.get("selector")
                .and_then(|v| v.as_str())
                .unwrap_or("field")
        ),
        "navigate_browser" => format!(
            "Navigate to {}",
            args.get("url").and_then(|v| v.as_str()).unwrap_or("URL")
        ),
        "select_option" => format!(
            "Select '{}' from dropdown",
            args.get("option_name")
                .and_then(|v| v.as_str())
                .unwrap_or("option")
        ),
        _ => format!("Execute {tool_name}"),
    }
}

pub fn get_wait_condition(tool_name: &str) -> Option<String> {
    match tool_name {
        "click_element" => Some("Element state changes or UI updates".to_string()),
        "type_into_element" => Some("Text appears in field".to_string()),
        "navigate_browser" => Some("Page loads completely".to_string()),
        "open_application" => Some("Application window appears".to_string()),
        _ => None,
    }
}

pub fn extract_required_tools(tool_calls: &[ToolCall]) -> Vec<String> {
    tool_calls
        .iter()
        .map(|tc| tc.tool_name.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}

pub fn infer_expected_outcomes(tool_calls: &[ToolCall]) -> Vec<String> {
    let mut outcomes = Vec::new();

    for call in tool_calls {
        match call.tool_name.as_str() {
            "navigate_browser" => outcomes.push("Target webpage loaded successfully".to_string()),
            "type_into_element" => outcomes.push("Form fields populated with data".to_string()),
            "click_element"
                if call
                    .arguments
                    .get("selector")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .contains("Submit") =>
            {
                outcomes.push("Form submitted successfully".to_string())
            }
            "select_option" => outcomes.push("Option selected in dropdown".to_string()),
            _ => {}
        }
    }

    outcomes
}

// Helper to optionally attach UI tree to response
pub fn maybe_attach_tree(
    desktop: &Desktop,
    include_tree: bool,
    pid_opt: Option<u32>,
    result_json: &mut Value,
) {
    if !include_tree {
        return;
    }
    if let Some(pid) = pid_opt {
        if let Ok(tree) = desktop.get_window_tree(pid, None, None) {
            if let Ok(tree_val) = serde_json::to_value(tree) {
                if let Some(obj) = result_json.as_object_mut() {
                    obj.insert("ui_tree".to_string(), tree_val);
                }
            }
        }
    }
}

pub fn should_add_focus_check(tool_calls: &[ToolCall], current_index: usize) -> bool {
    // Add focus check if:
    // 1. It's the first UI interaction
    // 2. Previous action was navigation or opened a new window
    // 3. There was a significant gap (e.g., after get_window_tree or wait)

    if current_index == 0 {
        return true;
    }

    let prev_tool = &tool_calls[current_index - 1].tool_name;
    matches!(
        prev_tool.as_str(),
        "navigate_browser"
            | "open_application"
            | "close_element"
            | "get_window_tree"
            | "get_applications"
            | "activate_element"
    )
}

pub fn is_state_changing_action(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "click_element"
            | "type_into_element"
            | "select_option"
            | "set_toggled"
            | "set_selected"
            | "set_range_value"
            | "invoke_element"
            | "press_key"
            | "mouse_drag"
            | "scroll_element"
    )
}

pub fn should_capture_tree(tool_name: &str, index: usize, total_steps: usize) -> bool {
    // Capture tree at key points:
    // 1. After major navigation
    // 2. Before complex sequences
    // 3. At regular intervals (every 5 steps)
    // 4. Before the final action

    matches!(tool_name, "navigate_browser" | "open_application")
        || index % 5 == 0
        || index == total_steps - 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_substitute_simple_variable() {
        let mut args = json!({"state": "{{desired_state}}"});
        let vars = json!({"desired_state": true});
        substitute_variables(&mut args, &vars);
        assert_eq!(args["state"], true);
    }

    #[test]
    fn test_substitute_expression_true() {
        let mut args = json!({"state": "{{contains(product_types, 'FEX')}}"});
        let vars = json!({"product_types": ["FEX", "Term"]});
        substitute_variables(&mut args, &vars);
        assert_eq!(args["state"], true);
    }

    #[test]
    fn test_substitute_expression_false() {
        let mut args = json!({"state": "{{startsWith(name, 'Jane')}}"});
        let vars = json!({"name": "John Doe"});
        substitute_variables(&mut args, &vars);
        assert_eq!(args["state"], false);
    }
}
