#!/usr/bin/env python3

import re

# Read the current server.rs file
with open('terminator-mcp-agent/src/server.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# List of all tools that need telemetry
TOOLS_NEEDING_TELEMETRY = [
    "validate_element",
    "wait_for_element",
    "navigate_browser",
    "open_application",
    "scroll_element",
    "close_element",
    "select_option",
    "list_options",
    "set_toggled",
    "set_range_value",
    "set_selected",
    "is_toggled",
    "get_range_value",
    "is_selected",
    "capture_element_screenshot",
    "invoke_element",
    "highlight_element",
    "stop_highlighting",
    "maximize_window",
    "minimize_window",
    "zoom_in",
    "zoom_out",
    "set_zoom",
    "set_value",
    "execute_browser_script",
    "mouse_drag",
    "delay",
    "activate_element",
    "record_workflow",
    "export_workflow_sequence",
    "import_workflow_sequence",
]

def add_comprehensive_telemetry(content, func_name):
    """Add comprehensive telemetry to a tool function"""

    # Pattern to find the function signature (handles pub async fn and async fn)
    pattern = rf'((?:pub\s+)?async\s+fn\s+{func_name}\s*\([^)]*\)\s*->\s*Result<CallToolResult,\s*McpError>\s*{{\s*\n)'

    matches = list(re.finditer(pattern, content, re.MULTILINE | re.DOTALL))

    if not matches:
        print(f"Warning: Function {func_name} not found")
        return content, False

    added = False
    for match in reversed(matches):  # Process in reverse to maintain indices
        # Check if already has telemetry
        check_area = content[match.end():match.end()+200]
        if "StepSpan::new" in check_area:
            print(f"Skipping {func_name} - already has telemetry")
            continue

        # Add telemetry with attributes
        telemetry_code = f'        let mut span = StepSpan::new("{func_name}", None);\n'

        # Add common attributes based on function name patterns
        if "selector" in func_name or func_name in ["validate_element", "wait_for_element", "scroll_element",
                                                     "close_element", "select_option", "list_options",
                                                     "set_toggled", "set_range_value", "set_selected",
                                                     "is_toggled", "get_range_value", "is_selected",
                                                     "capture_element_screenshot", "invoke_element",
                                                     "highlight_element", "maximize_window", "minimize_window",
                                                     "zoom_in", "zoom_out", "set_zoom", "set_value", "activate_element"]:
            telemetry_code += '\n        // Add telemetry attributes\n'
            telemetry_code += '        span.set_attribute("selector", args.selector.clone());\n'
            telemetry_code += '        if let Some(timeout) = args.timeout_ms {\n'
            telemetry_code += '            span.set_attribute("timeout_ms", timeout.to_string());\n'
            telemetry_code += '        }\n'
            telemetry_code += '        if let Some(retries) = args.retries {\n'
            telemetry_code += '            span.set_attribute("retry.max_attempts", retries.to_string());\n'
            telemetry_code += '        }\n'

        elif func_name == "mouse_drag":
            telemetry_code += '\n        // Add telemetry attributes\n'
            telemetry_code += '        span.set_attribute("from_selector", args.from_selector.clone());\n'
            telemetry_code += '        span.set_attribute("to_selector", args.to_selector.clone());\n'
            telemetry_code += '        if let Some(timeout) = args.timeout_ms {\n'
            telemetry_code += '            span.set_attribute("timeout_ms", timeout.to_string());\n'
            telemetry_code += '        }\n'
            telemetry_code += '        if let Some(retries) = args.retries {\n'
            telemetry_code += '            span.set_attribute("retry.max_attempts", retries.to_string());\n'
            telemetry_code += '        }\n'

        elif func_name == "navigate_browser":
            telemetry_code += '\n        // Add telemetry attributes\n'
            telemetry_code += '        span.set_attribute("url", args.url.clone());\n'
            telemetry_code += '        if let Some(ref wait_for) = args.wait_for {\n'
            telemetry_code += '            span.set_attribute("wait_for", wait_for.clone());\n'
            telemetry_code += '        }\n'
            telemetry_code += '        if let Some(timeout) = args.timeout_ms {\n'
            telemetry_code += '            span.set_attribute("timeout_ms", timeout.to_string());\n'
            telemetry_code += '        }\n'

        elif func_name == "open_application":
            telemetry_code += '\n        // Add telemetry attributes\n'
            telemetry_code += '        if let Some(ref app_name) = args.app_name {\n'
            telemetry_code += '            span.set_attribute("app_name", app_name.clone());\n'
            telemetry_code += '        }\n'
            telemetry_code += '        if let Some(ref app_path) = args.app_path {\n'
            telemetry_code += '            span.set_attribute("app_path", app_path.clone());\n'
            telemetry_code += '        }\n'
            telemetry_code += '        if let Some(timeout) = args.timeout_ms {\n'
            telemetry_code += '            span.set_attribute("timeout_ms", timeout.to_string());\n'
            telemetry_code += '        }\n'

        elif func_name == "delay":
            telemetry_code += '\n        // Add telemetry attributes\n'
            telemetry_code += '        span.set_attribute("duration_ms", args.duration_ms.to_string());\n'

        elif func_name == "execute_browser_script":
            telemetry_code += '\n        // Add telemetry attributes\n'
            telemetry_code += '        if let Some(ref script) = args.script {\n'
            telemetry_code += '            span.set_attribute("script.length", script.len().to_string());\n'
            telemetry_code += '        }\n'
            telemetry_code += '        if let Some(ref script_file) = args.script_file {\n'
            telemetry_code += '            span.set_attribute("script_file", script_file.clone());\n'
            telemetry_code += '        }\n'
            telemetry_code += '        if let Some(timeout) = args.timeout_ms {\n'
            telemetry_code += '            span.set_attribute("timeout_ms", timeout.to_string());\n'
            telemetry_code += '        }\n'

        elif func_name == "stop_highlighting":
            telemetry_code += '\n        // Add telemetry attributes\n'
            telemetry_code += '        span.set_attribute("highlight_id", args.highlight_id.clone());\n'

        # Insert the telemetry
        content = content[:match.end()] + telemetry_code + content[match.end():]

        # Now find where to add span.set_status and span.end
        # Look for Ok(CallToolResult::success pattern
        search_start = match.end() + len(telemetry_code)

        # Find the function's end by tracking braces
        brace_count = 1
        pos = search_start
        function_end = -1

        while pos < len(content) and brace_count > 0:
            if content[pos] == '{':
                brace_count += 1
            elif content[pos] == '}':
                brace_count -= 1
                if brace_count == 0:
                    function_end = pos
                    break
            pos += 1

        if function_end == -1:
            print(f"Warning: Couldn't find end of function {func_name}")
            continue

        # Find all Ok(CallToolResult::success within this function
        function_content = content[search_start:function_end]
        ok_pattern = r'(\s+)(Ok\(CallToolResult::success\([^)]*\)\))'

        # Find the last occurrence
        ok_matches = list(re.finditer(ok_pattern, function_content))

        if ok_matches:
            last_ok = ok_matches[-1]
            ok_pos = search_start + last_ok.start()
            indent = last_ok.group(1)

            # Check if telemetry end already exists
            check_before = content[ok_pos-100:ok_pos]
            if "span.end()" not in check_before:
                # Add telemetry before the Ok
                telemetry_end = f'\n{indent}span.set_status(true, None);\n{indent}span.end();\n'
                content = content[:ok_pos] + telemetry_end + content[ok_pos:]
                print(f"Added comprehensive telemetry to {func_name}")
                added = True
            else:
                print(f"Skipping {func_name} - already has span.end()")
        else:
            print(f"Warning: Couldn't find Ok(CallToolResult::success in {func_name}")

    return content, added

# Process each tool
added_count = 0
for tool_name in TOOLS_NEEDING_TELEMETRY:
    content, added = add_comprehensive_telemetry(content, tool_name)
    if added:
        added_count += 1

# Write the modified content back
with open('terminator-mcp-agent/src/server.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print(f"\nComprehensive telemetry addition complete!")
print(f"Added telemetry to {added_count} functions")
print("Next: Compile and test the changes")