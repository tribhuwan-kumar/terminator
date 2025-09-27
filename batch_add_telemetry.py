#!/usr/bin/env python3

import re

# Read the current server.rs file
with open('terminator-mcp-agent/src/server.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# List of tools that still need telemetry (excluding the ones we've already done)
TOOLS_TO_ADD = [
    ("type_into_element", "TypeIntoElementArgs"),
    ("press_key", "PressKeyArgs"),
    ("press_key_global", "GlobalKeyArgs"),
    ("validate_element", "ValidateElementArgs"),
    ("wait_for_element", "WaitForElementArgs"),
    ("activate_element", "ActivateElementArgs"),
    ("navigate_browser", "NavigateBrowserArgs"),
    ("execute_browser_script", "ExecuteBrowserScriptArgs"),
    ("open_application", "OpenApplicationArgs"),
    ("scroll_element", "ScrollElementArgs"),
    ("delay", "DelayArgs"),
    ("mouse_drag", "MouseDragArgs"),
    ("highlight_element", "HighlightElementArgs"),
    ("close_element", "CloseElementArgs"),
    ("select_option", "SelectOptionArgs"),
    ("list_options", "LocatorArgs"),
    ("set_toggled", "SetToggledArgs"),
    ("set_range_value", "SetRangeValueArgs"),
    ("set_selected", "SetSelectedArgs"),
    ("is_toggled", "LocatorArgs"),
    ("get_range_value", "LocatorArgs"),
    ("is_selected", "LocatorArgs"),
    ("capture_element_screenshot", "ValidateElementArgs"),
    ("invoke_element", "LocatorArgs"),
    ("record_workflow", "RecordWorkflowArgs"),
    ("maximize_window", "MaximizeWindowArgs"),
    ("minimize_window", "MinimizeWindowArgs"),
    ("zoom_in", "ZoomArgs"),
    ("zoom_out", "ZoomArgs"),
    ("set_zoom", "SetZoomArgs"),
    ("set_value", "SetValueArgs"),
    ("export_workflow_sequence", "ExportWorkflowSequenceArgs"),
    ("import_workflow_sequence", "ImportWorkflowSequenceArgs"),
    ("stop_highlighting", "StopHighlightingArgs"),
]

def add_telemetry_to_function(content, func_name, args_type):
    """Add telemetry to a specific function"""

    # Pattern to find the function signature (handle both pub async fn and async fn)
    pattern = rf'((?:pub )?async fn {func_name}\s*\([^)]*\)\s*->\s*Result<CallToolResult,\s*McpError>\s*{{)'

    matches = list(re.finditer(pattern, content, re.MULTILINE | re.DOTALL))

    if not matches:
        print(f"Warning: Function {func_name} not found")
        return content

    for match in reversed(matches):  # Process in reverse to maintain indices
        func_start = match.end()

        # Check if telemetry is already added
        check_area = content[func_start:func_start+200]
        if "StepSpan::new" in check_area:
            print(f"Skipping {func_name} - telemetry already exists")
            continue

        # Find the first newline after the opening brace
        first_newline = content.find('\n', func_start)
        if first_newline == -1:
            continue

        # Add telemetry after the first line
        telemetry_start = f'\n        // Start telemetry span\n        let mut span = StepSpan::new("{func_name}", None);'

        # Insert the telemetry
        content = content[:first_newline] + telemetry_start + content[first_newline:]

        # Now find the corresponding Ok(CallToolResult::success
        # Search forward from the function start
        search_start = func_start + len(telemetry_start)

        # Find all Ok(CallToolResult::success occurrences
        ok_pattern = r'(\n\s*)(Ok\(CallToolResult::success\()'
        ok_matches = list(re.finditer(ok_pattern, content[search_start:search_start+20000]))

        if ok_matches:
            # Get the position of the last Ok
            last_ok = ok_matches[-1]
            ok_pos = search_start + last_ok.start()

            # Insert telemetry before the Ok
            telemetry_end = '\n        span.set_status(true, None);\n        span.end();\n'

            content = content[:ok_pos] + telemetry_end + content[ok_pos:]
            print(f"Added telemetry to {func_name}")
        else:
            print(f"Warning: Could not find Ok(CallToolResult::success in {func_name}")

    return content

# Process each tool
for tool_name, args_type in TOOLS_TO_ADD:
    content = add_telemetry_to_function(content, tool_name, args_type)

# Write the modified content back
with open('terminator-mcp-agent/src/server.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("\nTelemetry addition complete!")
print("Note: run_command_impl needs special handling for error cases")