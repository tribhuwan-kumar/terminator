#!/usr/bin/env python3

import re

# Read the server.rs file
with open('terminator-mcp-agent/src/server.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Functions to add telemetry to
functions_to_update = [
    "activate_element",
    "delay",
    "mouse_drag",
    "validate_element",
    "highlight_element",
    "wait_for_element",
    "navigate_browser",
    "open_application",
    "close_element",
    "scroll_element",
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
    "stop_highlighting",
    "export_workflow_sequence",
    "import_workflow_sequence",
    "maximize_window",
    "minimize_window",
    "zoom_in",
    "zoom_out",
    "set_zoom",
    "set_value",
    "execute_browser_script",
    "run_command_impl"
]

def add_telemetry_to_lines(lines, func_name):
    """Add telemetry to a function in the lines"""
    i = 0
    while i < len(lines):
        # Find the function signature
        if re.match(rf'\s*(?:pub\s+)?async\s+fn\s+{func_name}\s*\(', lines[i]):
            print(f"Found {func_name} at line {i+1}")

            # Find the opening brace
            brace_line = i
            while brace_line < len(lines) and '{' not in lines[brace_line]:
                brace_line += 1

            # Check if telemetry already exists
            check_ahead = min(brace_line + 5, len(lines))
            has_telemetry = any('StepSpan::new' in lines[j] for j in range(brace_line, check_ahead))

            if has_telemetry:
                print(f"  {func_name} already has telemetry, skipping")
                i += 1
                continue

            # Add telemetry after the opening brace
            insert_line = brace_line + 1

            # Insert telemetry start
            telemetry_lines = [
                f'        // Start telemetry span\n',
                f'        let mut span = StepSpan::new("{func_name}", None);\n',
                '\n'
            ]

            lines[insert_line:insert_line] = telemetry_lines

            # Now find the Ok(CallToolResult::success and add telemetry before it
            j = insert_line + len(telemetry_lines)
            brace_count = 1

            # Find the end of the function
            while j < len(lines) and brace_count > 0:
                brace_count += lines[j].count('{') - lines[j].count('}')

                # Check for Ok(CallToolResult::success
                if 'Ok(CallToolResult::success' in lines[j]:
                    # Find the start of this line (indentation)
                    indent_match = re.match(r'^(\s*)', lines[j])
                    indent = indent_match.group(1) if indent_match else '        '

                    # Insert telemetry before this line
                    telemetry_end = [
                        f'\n{indent}span.set_status(true, None);\n',
                        f'{indent}span.end();\n\n'
                    ]
                    lines[j:j] = telemetry_end
                    j += len(telemetry_end)

                j += 1

            print(f"  Added telemetry to {func_name}")
            i = j
        else:
            i += 1

    return lines

# Process each function
for func_name in functions_to_update:
    lines = add_telemetry_to_lines(lines, func_name)

# Write the modified content
with open('terminator-mcp-agent/src/server.rs', 'w', encoding='utf-8') as f:
    f.writelines(lines)

print("\nTelemetry addition complete!")