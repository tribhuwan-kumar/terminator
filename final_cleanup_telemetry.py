#!/usr/bin/env python3

import re

# Read the current server.rs file
with open('terminator-mcp-agent/src/server.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Fix SetToggledArgs - it has state, not toggled
content = re.sub(
    r'span\.set_attribute\("toggled", args\.toggled\.to_string\(\)\);',
    'span.set_attribute("state", args.state.to_string());',
    content
)

# Fix SetSelectedArgs - it has state, not selected
content = re.sub(
    r'span\.set_attribute\("selected", args\.selected\.to_string\(\)\);',
    'span.set_attribute("state", args.state.to_string());',
    content
)

# Fix ZoomArgs - it only has level, not selector/steps/retries
# Remove these attributes from zoom_in and zoom_out
def remove_zoom_attrs(content, func_name):
    # Find the function
    func_pattern = rf'(async fn {func_name}\b.*?let mut span = StepSpan::new\("{func_name}", None\);)(.*?)(?=\n\s*async fn|\n\s*pub async fn|\Z)'
    match = re.search(func_pattern, content, re.DOTALL)
    if match:
        func_content = match.group(2)
        # Remove incorrect attributes
        func_content = re.sub(r'\n\s+span\.set_attribute\("selector", args\.selector\.clone\(\)\);', '', func_content)
        func_content = re.sub(r'\n\s+if let Some\(steps\) = args\.steps \{[^}]+\}', '', func_content)
        func_content = re.sub(r'\n\s+if let Some\(retries\) = args\.retries \{[^}]+\}', '', func_content)
        # Add correct attribute
        if 'span.set_attribute("level"' not in func_content:
            func_content = '\n\n        // Add comprehensive telemetry attributes\n        span.set_attribute("level", args.level.to_string());' + func_content
        content = content[:match.end(1)] + func_content + content[match.end(1) + len(match.group(2)):]
    return content

content = remove_zoom_attrs(content, 'zoom_in')
content = remove_zoom_attrs(content, 'zoom_out')

# Fix SetZoomArgs - it has percentage, not zoom_level or selector
def fix_set_zoom(content):
    func_pattern = r'(async fn set_zoom\b.*?let mut span = StepSpan::new\("set_zoom", None\);)(.*?)(?=\n\s*async fn|\n\s*pub async fn|\Z)'
    match = re.search(func_pattern, content, re.DOTALL)
    if match:
        func_content = match.group(2)
        # Remove incorrect attributes
        func_content = re.sub(r'\n\s+span\.set_attribute\("selector", args\.selector\.clone\(\)\);', '', func_content)
        func_content = re.sub(r'\n\s+span\.set_attribute\("zoom_level", args\.zoom_level\.to_string\(\)\);', '', func_content)
        func_content = re.sub(r'\n\s+if let Some\(timeout\) = args\.timeout_ms \{[^}]+\}', '', func_content)
        func_content = re.sub(r'\n\s+if let Some\(retries\) = args\.retries \{[^}]+\}', '', func_content)
        # Add correct attribute
        if 'span.set_attribute("percentage"' not in func_content:
            func_content = '\n\n        // Add comprehensive telemetry attributes\n        span.set_attribute("percentage", args.percentage.to_string());' + func_content
        content = content[:match.end(1)] + func_content + content[match.end(1) + len(match.group(2)):]
    return content

content = fix_set_zoom(content)

# Write the modified content back
with open('terminator-mcp-agent/src/server.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("Applied final cleanup for telemetry")