# Python Automation Examples

This folder contains examples of using the Terminator MCP Python engine for desktop and browser automation.

## Prerequisites

- Terminator MCP agent built and available
- Python 3.8+ installed
- The `terminator` Python package (automatically installed when running)

## Examples

### 1. Desktop Analysis (`desktop_analysis.py`)

Analyzes the current desktop state and returns information about open windows and applications.

**Run:**
```bash
terminator mcp run run-desktop-analysis.yml --command "target/release/terminator-mcp-agent"
```

**Features:**
- Enumerates all open windows
- Groups windows by application
- Identifies the focused window
- Returns structured data about the desktop state

### 2. Browser Automation (`browser_automation.py`)

Automates browser interactions including navigation and data extraction.

**Run:**
```bash
terminator mcp run run-browser-automation.yml --command "target/release/terminator-mcp-agent"
```

**With custom parameters:**
```bash
terminator mcp run run-browser-automation.yml \
  --command "target/release/terminator-mcp-agent" \
  --inputs '{"browser_name": "edge", "target_url": "https://github.com"}'
```

**Features:**
- Opens specified browser
- Navigates to URLs
- Captures screenshots
- Extracts text from pages
- Returns page information

### 3. Notepad Automation (`notepad_automation.py`)

Demonstrates text editor automation with keyboard shortcuts.

**Run:**
```bash
terminator mcp run run-notepad-automation.yml --command "target/release/terminator-mcp-agent"
```

**Features:**
- Opens Notepad application
- Types text programmatically
- Uses keyboard shortcuts (Ctrl+A, etc.)
- Accesses application menus
- Retrieves text content

## Python Engine Features

The Python scripts in these examples demonstrate:

### Built-in Objects and Functions

- `desktop` - Main automation object for UI interaction
- `log(message)` - Logging function for debug output
- `sleep(milliseconds)` - Async sleep function
- `set_env(key, value)` - Set environment variables for workflow

### Async/Await Support

All automation methods use async/await syntax:
```python
windows = await desktop.locator("role:Window").all()
await element.click()
await sleep(1000)
```

### Selector Syntax

Elements are located using role and name attributes:
```python
# Find by role and name
await desktop.locator("role:Button|name:Submit").first()

# Find by role only
await desktop.locator("role:Edit").all()

# Chain selectors
await window.locator("role:Document").first()
```

### Return Values

Python scripts can return data that gets passed to subsequent workflow steps:
```python
return {
    "status": "success",
    "data": result_data,
    "message": "Operation complete"
}
```

## Running from Python Files vs Inline

### From File:
```yaml
- id: run_script
  tool_name: run_command
  arguments:
    engine: "python"
    script_file: "./my_script.py"
```

### Inline:
```yaml
- id: run_inline
  tool_name: run_command
  arguments:
    engine: "python"
    run: |
      log("Inline Python code")
      windows = await desktop.locator("role:Window").all()
      return {"window_count": len(windows)}
```

## Environment Variables

Pass variables to Python scripts:
```yaml
- id: run_with_env
  tool_name: run_command
  arguments:
    engine: "python"
    script_file: "./script.py"
    env:
      MY_VAR: "value"
      COUNT: "10"
```

Access in Python:
```python
import os
my_var = os.environ.get('MY_VAR', 'default')
count = int(os.environ.get('COUNT', '0'))
```

## Error Handling

Python scripts should handle errors gracefully:
```python
try:
    element = await desktop.locator("role:Button").first()
    if element:
        await element.click()
        return {"status": "success"}
    else:
        return {"status": "error", "message": "Button not found"}
except Exception as e:
    log(f"Error: {e}")
    return {"status": "error", "message": str(e)}
```

## Tips

1. **Use logging** - The `log()` function helps debug automation scripts
2. **Add delays** - Use `await sleep(ms)` between actions for reliability
3. **Check elements exist** - Always verify elements before interacting
4. **Return structured data** - Return dictionaries with clear status/data fields
5. **Handle failures gracefully** - Use try/except blocks and return error states

## Troubleshooting

### Script hangs or times out
- Check that elements exist before interacting
- Add appropriate delays between actions
- Verify application is in expected state

### Element not found
- Use more specific selectors
- Try alternative role types
- Check if application UI has changed

### Python package issues
- The `terminator` package is auto-installed
- For other packages, install manually first
- Check Python version compatibility (3.8+)