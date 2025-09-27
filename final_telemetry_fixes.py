#!/usr/bin/env python3

import re

# Read the current server.rs file
with open('terminator-mcp-agent/src/server.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Fix issues line by line
fixed_lines = []
i = 0
while i < len(lines):
    line = lines[i]

    # Fix delay_ms for highlight_element (it should be duration_ms field of HighlightElementArgs)
    if 'span.set_attribute("delay_ms", args.delay_ms.to_string());' in line:
        # Check context - if it's in delay function, keep it; if in highlight_element, change
        # Look back for function name
        func_context = ""
        for j in range(max(0, i-20), i):
            if 'async fn' in lines[j]:
                func_context = lines[j]
                break

        if 'delay' in func_context and 'highlight' not in func_context:
            fixed_lines.append(line)  # Keep for delay function
        else:
            # Skip this line for highlight_element
            i += 1
            continue

    # Remove state attribute for wait_for_element
    elif 'span.set_attribute("state"' in line:
        i += 1
        continue  # Skip this line

    # Fix OpenApplicationArgs - only has app_name
    elif 'if let Some(ref app_path) = args.app_path' in line:
        i += 1
        if i < len(lines) and 'span.set_attribute' in lines[i]:
            i += 1
        if i < len(lines) and '}' in lines[i]:
            i += 1
        continue  # Skip these lines

    elif 'if let Some(ref arguments) = args.arguments' in line:
        i += 1
        if i < len(lines) and 'span.set_attribute' in lines[i]:
            i += 1
        if i < len(lines) and '}' in lines[i]:
            i += 1
        continue  # Skip these lines

    elif 'span.set_attribute("wait_for_window"' in line:
        i += 1
        continue  # Skip this line

    # Fix color format
    elif 'span.set_attribute("color", format!("#{:08X}", color));' in line:
        # This is correct for u32, keep it
        fixed_lines.append(line)

    # Fix option_name for select_option
    elif 'span.set_attribute("option_name", args.option_name.clone());' in line:
        # Check if we're in select_option function
        func_context = ""
        for j in range(max(0, i-20), i):
            if 'async fn select_option' in lines[j]:
                func_context = lines[j]
                break

        if 'select_option' in func_context:
            # Make sure it's properly indented
            fixed_lines.append('        span.set_attribute("option_name", args.option_name.clone());\n')
        else:
            # Skip for other functions
            i += 1
            continue

    # Fix cannot find value `args` in scope
    elif 'span.set_attribute("delay_ms", args.delay_ms.to_string());' in line:
        # Check if we're in the right function
        func_context = ""
        for j in range(max(0, i-30), i):
            if 'async fn' in lines[j]:
                func_context = lines[j]
                break

        if 'delay' not in func_context:
            i += 1
            continue  # Skip if not in delay function

        fixed_lines.append(line)

    else:
        fixed_lines.append(line)

    i += 1

# Write the fixed content back
with open('terminator-mcp-agent/src/server.rs', 'w', encoding='utf-8') as f:
    f.writelines(fixed_lines)

print("Applied final telemetry fixes")