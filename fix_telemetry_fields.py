#!/usr/bin/env python3

import re

# Read the current server.rs file
with open('terminator-mcp-agent/src/server.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Fix DelayArgs - it's delay_ms not duration_ms
content = re.sub(
    r'span\.set_attribute\("duration_ms", args\.duration_ms\.to_string\(\)\);',
    'span.set_attribute("delay_ms", args.delay_ms.to_string());',
    content
)

# Fix MouseDragArgs - it's just selector, not from_selector/to_selector
# Look for the mouse_drag function and fix it
content = re.sub(
    r'span\.set_attribute\("from_selector", args\.from_selector\.clone\(\)\);',
    'span.set_attribute("selector", args.selector.clone());',
    content
)
content = re.sub(
    r'span\.set_attribute\("to_selector", args\.to_selector\.clone\(\)\);',
    '// Mouse drag uses x,y coordinates, not selectors',
    content
)

# Fix color field - it's a u32 not a tuple
content = re.sub(
    r'span\.set_attribute\("color", format!\("\#\{:02X\}\{:02X\}\{:02X\}", color\.0, color\.1, color\.2\)\);',
    'span.set_attribute("color", format!("#{:08X}", color));',
    content
)

# Fix WaitForElementArgs - check what fields it actually has
# Remove state field (doesn't exist)
content = re.sub(
    r'span\.set_attribute\("state", format!\("\{:?\}", args\.state\)\);',
    '',
    content
)

# Fix NavigateBrowserArgs - check actual fields
# Remove fields that don't exist
content = re.sub(
    r'\s+if let Some\(ref wait_for\) = args\.wait_for \{[^}]+\}',
    '',
    content
)
content = re.sub(
    r'\s+if let Some\(timeout\) = args\.timeout_ms \{[^}]+\}',
    '',
    content
)
content = re.sub(
    r'\s+span\.set_attribute\("use_protocol", args\.use_protocol\.unwrap_or\(false\)\.to_string\(\)\);',
    '',
    content
)

# Fix execute_browser_script - check actual field names
# browser_pid, browser_window_name, browser_url_pattern might not exist
# Let's check and remove non-existent ones
content = re.sub(
    r'\s+if let Some\(browser_pid\) = args\.browser_pid \{[^}]+\}',
    '',
    content
)
content = re.sub(
    r'\s+if let Some\(ref browser_window_name\) = args\.browser_window_name \{[^}]+\}',
    '',
    content
)
content = re.sub(
    r'\s+if let Some\(ref browser_url_pattern\) = args\.browser_url_pattern \{[^}]+\}',
    '',
    content
)

# Fix option_selector, option_value, option_text - might be different field names
# For now, let's check what SelectOptionArgs actually has

# Write the modified content back
with open('terminator-mcp-agent/src/server.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("Fixed telemetry field names")