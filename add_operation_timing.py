#!/usr/bin/env python3

import re

# Read the current server.rs file
with open('terminator-mcp-agent/src/server.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Tools that likely use find_and_execute pattern
TOOLS_WITH_ELEMENT_FINDING = [
    "activate_element",
    "close_element",
    "scroll_element",
    "wait_for_element",
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
    "maximize_window",
    "minimize_window",
    "zoom_in",
    "zoom_out",
    "set_zoom",
    "set_value",
    "mouse_drag",
]

def add_operation_timing(content, func_name):
    """Add operation timing tracking to functions that find elements"""

    # Find the function and look for find_and_execute pattern
    func_pattern = rf'async fn {func_name}\b.*?Result<CallToolResult,\s*McpError>'
    func_match = re.search(func_pattern, content, re.DOTALL)

    if not func_match:
        print(f"Warning: {func_name} not found")
        return content

    # Look for find_and_execute_with_retry_with_fallback within this function
    func_start = func_match.start()

    # Find the end of the function by counting braces
    brace_count = 0
    in_function = False
    func_end = func_start

    for i in range(func_start, len(content)):
        if content[i] == '{':
            brace_count += 1
            in_function = True
        elif content[i] == '}':
            brace_count -= 1
            if in_function and brace_count == 0:
                func_end = i
                break

    func_content = content[func_start:func_end]

    # Check if it has find_and_execute_with_retry_with_fallback
    if "find_and_execute_with_retry_with_fallback" not in func_content:
        print(f"Skipping {func_name} - no find_and_execute pattern")
        return content

    # Check if already has operation timing
    if "operation_start" in func_content:
        print(f"Skipping {func_name} - already has operation timing")
        return content

    # Find where to add operation_start
    # Look for pattern before find_and_execute_with_retry_with_fallback
    pattern = r'(\n\s+)((?:let.*=\s*)?(?:match\s+)?find_and_execute_with_retry_with_fallback\()'

    match = re.search(pattern, func_content)
    if not match:
        print(f"Warning: {func_name} - couldn't find find_and_execute pattern")
        return content

    # Add operation timing before the find_and_execute call
    indent = match.group(1)
    timing_code = f'{indent}let operation_start = std::time::Instant::now();{indent}'

    # Insert the timing code
    insert_pos = func_start + match.start(2)
    content = content[:insert_pos] + f'let operation_start = std::time::Instant::now();\n        ' + content[insert_pos:]

    # Now find the Ok branch to add timing tracking
    # Look for pattern like Ok(((result, element), selector)) =>
    ok_pattern = r'Ok\(\(\(.*?\),\s*(?:selector|successful_selector)\)\)\s*=>\s*{'

    # Search within the updated function content
    func_end_updated = func_end + len(f'let operation_start = std::time::Instant::now();\n        ')
    func_content_updated = content[func_start:func_end_updated]

    ok_match = re.search(ok_pattern, func_content_updated)
    if ok_match:
        # Add timing tracking after the Ok match
        ok_pos = func_start + ok_match.end()
        timing_track = '''
                let operation_time_ms = operation_start.elapsed().as_millis() as i64;
                span.set_attribute("operation.duration_ms", operation_time_ms.to_string());
                span.set_attribute("element.found", "true".to_string());'''

        content = content[:ok_pos] + timing_track + content[ok_pos:]
        print(f"Added operation timing to {func_name}")
    else:
        # Try simpler Ok pattern
        ok_pattern2 = r'Ok\(\(\(.*?\)\)\)\s*=>\s*{'
        ok_match2 = re.search(ok_pattern2, func_content_updated)
        if ok_match2:
            ok_pos = func_start + ok_match2.end()
            timing_track = '''
                let operation_time_ms = operation_start.elapsed().as_millis() as i64;
                span.set_attribute("operation.duration_ms", operation_time_ms.to_string());
                span.set_attribute("element.found", "true".to_string());'''

            content = content[:ok_pos] + timing_track + content[ok_pos:]
            print(f"Added operation timing to {func_name}")
        else:
            print(f"Warning: {func_name} - couldn't find Ok branch")

    return content

# Process each tool
for tool_name in TOOLS_WITH_ELEMENT_FINDING:
    content = add_operation_timing(content, tool_name)

# Write the modified content back
with open('terminator-mcp-agent/src/server.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("\nOperation timing addition complete!")