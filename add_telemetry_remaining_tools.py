#!/usr/bin/env python3

import re

# Read the current server.rs file
with open('terminator-mcp-agent/src/server.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Tools that need comprehensive telemetry enhancements
TOOLS_TO_ENHANCE = {
    "activate_element": ["selector", "timeout_ms", "retries"],
    "close_element": ["selector", "timeout_ms", "retries"],
    "scroll_element": ["selector", "direction", "amount", "timeout_ms", "retries"],
    "navigate_browser": ["url", "wait_for", "timeout_ms", "use_protocol"],
    "open_application": ["app_name", "app_path", "arguments", "wait_for_window", "timeout_ms"],
    "wait_for_element": ["selector", "state", "timeout_ms", "retries"],
    "select_option": ["selector", "option_selector", "option_value", "option_text", "timeout_ms", "retries"],
    "list_options": ["selector", "timeout_ms", "retries"],
    "set_toggled": ["selector", "toggled", "timeout_ms", "retries"],
    "set_range_value": ["selector", "value", "timeout_ms", "retries"],
    "set_selected": ["selector", "selected", "timeout_ms", "retries"],
    "is_toggled": ["selector", "timeout_ms", "retries"],
    "get_range_value": ["selector", "timeout_ms", "retries"],
    "is_selected": ["selector", "timeout_ms", "retries"],
    "capture_element_screenshot": ["selector", "timeout_ms", "retries"],
    "invoke_element": ["selector", "timeout_ms", "retries"],
    "highlight_element": ["selector", "duration_ms", "color", "timeout_ms", "retries"],
    "stop_highlighting": ["highlight_id"],
    "maximize_window": ["selector", "timeout_ms", "retries"],
    "minimize_window": ["selector", "timeout_ms", "retries"],
    "zoom_in": ["selector", "steps", "timeout_ms", "retries"],
    "zoom_out": ["selector", "steps", "timeout_ms", "retries"],
    "set_zoom": ["selector", "zoom_level", "timeout_ms", "retries"],
    "set_value": ["selector", "value", "timeout_ms", "retries"],
    "execute_browser_script": ["script", "script_file", "browser_pid", "browser_window_name", "browser_url_pattern", "timeout_ms"],
    "mouse_drag": ["from_selector", "to_selector", "timeout_ms", "retries"],
    "delay": ["duration_ms"],
}

def add_comprehensive_telemetry(content, func_name, attributes):
    """Add comprehensive telemetry attributes right after StepSpan::new"""

    # Find the StepSpan::new line for this function
    pattern = rf'(let mut span = StepSpan::new\("{func_name}", None\);)\s*\n'

    matches = list(re.finditer(pattern, content))

    if not matches:
        print(f"Warning: {func_name} - StepSpan not found")
        return content

    for match in reversed(matches):
        # Check if already has attributes
        check_area = content[match.end():match.end()+200]
        if "span.set_attribute" in check_area:
            print(f"Skipping {func_name} - already has attributes")
            continue

        # Build telemetry code
        telemetry_code = ""

        # Add a blank line and comment
        telemetry_code += "\n\n        // Add comprehensive telemetry attributes"

        for attr in attributes:
            if attr == "selector":
                telemetry_code += '\n        span.set_attribute("selector", args.selector.clone());'
            elif attr == "from_selector":
                telemetry_code += '\n        span.set_attribute("from_selector", args.from_selector.clone());'
            elif attr == "to_selector":
                telemetry_code += '\n        span.set_attribute("to_selector", args.to_selector.clone());'
            elif attr == "timeout_ms":
                telemetry_code += '\n        if let Some(timeout) = args.timeout_ms {\n            span.set_attribute("timeout_ms", timeout.to_string());\n        }'
            elif attr == "retries":
                telemetry_code += '\n        if let Some(retries) = args.retries {\n            span.set_attribute("retry.max_attempts", retries.to_string());\n        }'
            elif attr == "duration_ms":
                telemetry_code += '\n        span.set_attribute("duration_ms", args.duration_ms.to_string());'
            elif attr == "url":
                telemetry_code += '\n        span.set_attribute("url", args.url.clone());'
            elif attr == "app_name":
                telemetry_code += '\n        if let Some(ref app_name) = args.app_name {\n            span.set_attribute("app_name", app_name.clone());\n        }'
            elif attr == "app_path":
                telemetry_code += '\n        if let Some(ref app_path) = args.app_path {\n            span.set_attribute("app_path", app_path.clone());\n        }'
            elif attr == "script":
                telemetry_code += '\n        if let Some(ref script) = args.script {\n            span.set_attribute("script.length", script.len().to_string());\n        }'
            elif attr == "script_file":
                telemetry_code += '\n        if let Some(ref script_file) = args.script_file {\n            span.set_attribute("script_file", script_file.clone());\n        }'
            elif attr == "direction":
                telemetry_code += '\n        span.set_attribute("direction", format!("{:?}", args.direction));'
            elif attr == "amount":
                telemetry_code += '\n        if let Some(amount) = args.amount {\n            span.set_attribute("amount", amount.to_string());\n        }'
            elif attr == "state":
                telemetry_code += '\n        span.set_attribute("state", format!("{:?}", args.state));'
            elif attr == "toggled":
                telemetry_code += '\n        span.set_attribute("toggled", args.toggled.to_string());'
            elif attr == "selected":
                telemetry_code += '\n        span.set_attribute("selected", args.selected.to_string());'
            elif attr == "value":
                telemetry_code += '\n        span.set_attribute("value", args.value.clone());'
            elif attr == "zoom_level":
                telemetry_code += '\n        span.set_attribute("zoom_level", args.zoom_level.to_string());'
            elif attr == "steps":
                telemetry_code += '\n        if let Some(steps) = args.steps {\n            span.set_attribute("steps", steps.to_string());\n        }'
            elif attr == "highlight_id":
                telemetry_code += '\n        span.set_attribute("highlight_id", args.highlight_id.clone());'
            elif attr == "color":
                telemetry_code += '\n        if let Some(ref color) = args.color {\n            span.set_attribute("color", format!("#{:02X}{:02X}{:02X}", color.0, color.1, color.2));\n        }'
            elif attr == "wait_for":
                telemetry_code += '\n        if let Some(ref wait_for) = args.wait_for {\n            span.set_attribute("wait_for", wait_for.clone());\n        }'
            elif attr == "use_protocol":
                telemetry_code += '\n        span.set_attribute("use_protocol", args.use_protocol.unwrap_or(false).to_string());'
            elif attr == "wait_for_window":
                telemetry_code += '\n        span.set_attribute("wait_for_window", args.wait_for_window.unwrap_or(false).to_string());'
            elif attr == "arguments":
                telemetry_code += '\n        if let Some(ref arguments) = args.arguments {\n            span.set_attribute("arguments.count", arguments.len().to_string());\n        }'
            elif attr == "option_selector":
                telemetry_code += '\n        if let Some(ref option_selector) = args.option_selector {\n            span.set_attribute("option_selector", option_selector.clone());\n        }'
            elif attr == "option_value":
                telemetry_code += '\n        if let Some(ref option_value) = args.option_value {\n            span.set_attribute("option_value", option_value.clone());\n        }'
            elif attr == "option_text":
                telemetry_code += '\n        if let Some(ref option_text) = args.option_text {\n            span.set_attribute("option_text", option_text.clone());\n        }'
            elif attr == "browser_pid":
                telemetry_code += '\n        if let Some(browser_pid) = args.browser_pid {\n            span.set_attribute("browser_pid", browser_pid.to_string());\n        }'
            elif attr == "browser_window_name":
                telemetry_code += '\n        if let Some(ref browser_window_name) = args.browser_window_name {\n            span.set_attribute("browser_window_name", browser_window_name.clone());\n        }'
            elif attr == "browser_url_pattern":
                telemetry_code += '\n        if let Some(ref browser_url_pattern) = args.browser_url_pattern {\n            span.set_attribute("browser_url_pattern", browser_url_pattern.clone());\n        }'

        # Insert after the StepSpan::new line
        content = content[:match.end()] + telemetry_code + content[match.end():]
        print(f"Enhanced telemetry for {func_name}")

    return content

# Process each tool
for tool_name, attributes in TOOLS_TO_ENHANCE.items():
    content = add_comprehensive_telemetry(content, tool_name, attributes)

# Write the modified content back
with open('terminator-mcp-agent/src/server.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("\nComprehensive telemetry addition complete!")
print(f"Enhanced {len(TOOLS_TO_ENHANCE)} tools with telemetry attributes")