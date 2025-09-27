#!/usr/bin/env python3

import re

# Read the current server.rs file
with open('terminator-mcp-agent/src/server.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# List of all tools that need basic telemetry
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

def add_basic_telemetry(content, func_name):
    """Add basic telemetry to a tool function"""

    # Pattern to find the function signature
    pattern = rf'((?:pub )?async fn {func_name}\s*\([^)]*\)\s*->\s*Result<CallToolResult,\s*McpError>\s*{{\s*\n)'

    matches = list(re.finditer(pattern, content, re.MULTILINE))

    if not matches:
        print(f"Warning: Function {func_name} not found")
        return content

    for match in reversed(matches):  # Process in reverse to maintain indices
        # Check if already has telemetry
        check_area = content[match.end():match.end()+200]
        if "StepSpan::new" in check_area:
            print(f"Skipping {func_name} - already has telemetry")
            continue

        # Add basic telemetry after the opening brace
        telemetry_code = f'        let mut span = StepSpan::new("{func_name}", None);\n'

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

            # Add telemetry before the Ok
            telemetry_end = f'\n{indent}span.set_status(true, None);\n{indent}span.end();\n'
            content = content[:ok_pos] + telemetry_end + content[ok_pos:]
            print(f"Added basic telemetry to {func_name}")
        else:
            print(f"Warning: Couldn't find Ok(CallToolResult::success in {func_name}")

    return content

# Process each tool
for tool_name in TOOLS_NEEDING_TELEMETRY:
    content = add_basic_telemetry(content, tool_name)

# Write the modified content back
with open('terminator-mcp-agent/src/server.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("\nBasic telemetry addition complete!")
print("Run add_comprehensive_telemetry.py next to enhance with attributes")