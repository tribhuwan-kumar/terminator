#!/usr/bin/env python3

import re
import sys

# List of all MCP tools that need telemetry
TOOLS = [
    "get_focused_window_tree",
    "get_applications",
    "click_element",
    "type_into_element",
    "press_key",
    "press_key_global",
    "validate_element",
    "wait_for_element",
    "activate_element",
    "navigate_browser",
    "execute_browser_script",
    "open_application",
    "scroll_element",
    "delay",
    "run_command_impl",  # Note: run_command delegates to run_command_impl
    "mouse_drag",
    "highlight_element",
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
    "record_workflow",
    "maximize_window",
    "minimize_window",
    "zoom_in",
    "zoom_out",
    "set_zoom",
    "set_value",
    "export_workflow_sequence",
    "import_workflow_sequence",
    "stop_highlighting"
]

def add_telemetry_to_function(content, func_name):
    """Add telemetry tracking to a specific function"""

    # Pattern to find the function
    pattern = rf'(pub async fn {func_name}\([^)]*\)[^{{]*{{)'

    # Find all matches
    matches = list(re.finditer(pattern, content, re.MULTILINE | re.DOTALL))

    if not matches:
        print(f"Warning: Function {func_name} not found")
        return content

    for match in reversed(matches):  # Process in reverse to maintain indices
        func_start = match.end()

        # Find the matching closing brace for this function
        brace_count = 1
        i = func_start
        while i < len(content) and brace_count > 0:
            if content[i] == '{':
                brace_count += 1
            elif content[i] == '}':
                brace_count -= 1
            i += 1
        func_end = i - 1

        # Check if telemetry is already added
        func_body = content[func_start:func_end]
        if "StepSpan::new" in func_body[:500]:  # Check first 500 chars
            print(f"Skipping {func_name} - telemetry already exists")
            continue

        # Add telemetry at the start
        telemetry_start = f'\n        // Start telemetry span\n        let mut span = StepSpan::new("{func_name}", None);\n'

        # Add telemetry at the end (before the final return)
        # Find the last Ok( or Err( return statement
        return_matches = list(re.finditer(r'(\n\s*)(Ok\(|Err\()', func_body[::-1]))
        if return_matches:
            # Get the position of the last return
            last_return_pos = len(func_body) - return_matches[0].start()

            # Insert success telemetry before the return
            telemetry_end = '\n        span.set_status(true, None);\n        span.end();\n'

            # Reconstruct the function body
            new_func_body = (
                telemetry_start +
                func_body[:last_return_pos] +
                telemetry_end +
                '\n        ' +
                func_body[last_return_pos:]
            )

            # Replace in the original content
            content = content[:func_start] + new_func_body + content[func_end:]
            print(f"Added telemetry to {func_name}")
        else:
            print(f"Warning: Could not find return statement in {func_name}")

    return content

def main():
    # Read the server.rs file
    with open('terminator-mcp-agent/src/server.rs', 'r', encoding='utf-8') as f:
        content = f.read()

    # Process each tool function
    for tool in TOOLS:
        content = add_telemetry_to_function(content, tool)

    # Write the modified content
    with open('terminator-mcp-agent/src/server.rs', 'w', encoding='utf-8') as f:
        f.write(content)

    print("\nTelemetry addition complete!")

if __name__ == "__main__":
    main()