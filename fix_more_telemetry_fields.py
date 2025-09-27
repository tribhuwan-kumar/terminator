#!/usr/bin/env python3

import re

# Read the current server.rs file
with open('terminator-mcp-agent/src/server.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Fix SelectOptionArgs - it's option_name, not option_selector/option_value/option_text
content = re.sub(
    r'\s+if let Some\(ref option_selector\) = args\.option_selector \{[^}]+\}',
    '',
    content
)
content = re.sub(
    r'\s+if let Some\(ref option_value\) = args\.option_value \{[^}]+\}',
    '',
    content
)
content = re.sub(
    r'\s+if let Some\(ref option_text\) = args\.option_text \{[^}]+\}',
    '        span.set_attribute("option_name", args.option_name.clone());',
    content
)

# Clean up any double blank lines created by removals
content = re.sub(r'\n\n\n+', '\n\n', content)

# Remove unused operation_start variables
content = re.sub(
    r'let operation_start = std::time::Instant::now\(\);\n\s+',
    '',
    content
)

# Write the modified content back
with open('terminator-mcp-agent/src/server.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("Fixed more telemetry field names")