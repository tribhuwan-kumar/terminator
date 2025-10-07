# Run Command Tool Documentation

## Overview

The `run_command` tool allows you to execute shell commands using GitHub Actions-style syntax. This provides a simpler, more intuitive interface compared to platform-specific commands.

## GitHub Actions-Style Syntax

The new syntax follows the [GitHub Actions convention](https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/add-scripts) for running scripts and commands.

### Tool Arguments

- **`run`** (optional): The command to execute. Can be a single command, multi-line script, or inline code when using `engine` mode. Either `run` or `script_file` must be provided.
- **`script_file`** (optional): Path to a script file to load and execute. Either `run` or `script_file` must be provided. When using `engine`, the file should contain JavaScript or Python code.
- **`env`** (optional): Environment variables to inject into the script (only works with `engine` mode). Variables are automatically available as proper JavaScript/Python types with smart JSON detection - JSON strings are parsed into objects/arrays. Variables can be accessed directly without any prefix.
- **`engine`** (optional): High-level engine to execute inline code with SDK bindings. Options:
  - `javascript`, `js`, `node`, `bun` - Execute JavaScript with terminator.js bindings
  - `python` - Execute Python with terminator.py bindings
  - When set, `run` or `script_file` must contain the code to execute
- **`shell`** (optional): The shell to use for execution (ignored when `engine` is used). Defaults to:
  - Windows: `powershell`
  - Unix/Linux/macOS: `bash`
- **`working_directory`** (optional): The directory where the command should be executed. Defaults to the current directory.

### Supported Shells

#### Windows
- `powershell` (default) - PowerShell Core or Windows PowerShell
- `pwsh` - PowerShell Core explicitly
- `cmd` - Windows Command Prompt
- `bash` - Git Bash or WSL Bash
- `sh` - Shell (typically Git Bash on Windows)

#### Unix/Linux/macOS
- `bash` (default) - Bash shell
- `sh` - POSIX shell
- `zsh` - Z shell
- `python` - Execute as Python code
- `node` - Execute as JavaScript code

## Examples

### Basic Command Execution

```json
{
  "run": "echo 'Hello, World!'"
}
```

### Multi-line Script

```json
{
  "run": "echo 'Starting process...'\nls -la\necho 'Process complete!'"
}
```

### Using a Specific Shell

```json
{
  "run": "Get-Process | Select-Object -First 5",
  "shell": "powershell"
}
```

### With Working Directory

```json
{
  "run": "npm install",
  "working_directory": "./my-project"
}
```

### Cross-Platform Script

```json
{
  "run": "python --version",
  "shell": "python"
}
```

### Loading JavaScript from External File

```json
{
  "engine": "javascript",
  "script_file": "C:\\scripts\\process_data.js",
  "env": {
    "input_folder": "C:\\data",
    "output_folder": "C:\\processed"
  }
}
```

### Using Environment Variables with Inline Code

```json
{
  "engine": "javascript",
  "run": "// Variables are directly available as proper types\nconsole.log(`Processing ${file_count} files from ${source_dir}`);\nreturn { set_env: { processed: true, timestamp: new Date().toISOString() } };",
  "env": {
    "file_count": 42,
    "source_dir": "C:\\input"
  }
}
```

## Platform-Specific Behavior

The tool automatically handles platform differences:

### Windows
- Default shell is PowerShell
- Working directory changes use PowerShell's `cd` command
- Commands are executed with appropriate escaping

### Unix/Linux/macOS
- Default shell is Bash
- Working directory changes use `cd` command
- Standard Unix command execution

## Output Format

The tool returns a JSON object with the following fields:

```json
{
  "exit_status": 0,
  "stdout": "Command output here",
  "stderr": "Any error output",
  "command": "echo 'Hello'",
  "shell": "bash",
  "working_directory": "/home/user"
}
```

## Migration from Old Syntax

### Old Syntax
```json
{
  "windows_command": "dir",
  "unix_command": "ls"
}
```

### New Syntax
```json
{
  "run": "ls"
}
```
The new syntax automatically detects the platform and uses the appropriate command execution method.

## Best Practices

1. **Use the default shell** when possible for better cross-platform compatibility
2. **Prefer simple commands** over complex scripts for reliability
3. **Test commands locally** before using in automation
4. **Use working_directory** instead of `cd` commands when possible
5. **Handle errors** by checking the `exit_status` field

## Pattern: Optional Element Detection

For optional UI elements (dialogs, popups, confirmation buttons) that may or may not appear, you have two approaches:

### Approach 1: validate_element (PREFERRED - Simpler)

Use the `validate_element` tool which never throws errors and returns structured existence information.

**Advantages:**
- **Built-in tool** - No JavaScript required
- **Never fails** - Returns `status: "success"` with `exists: true` OR `status: "failed"` with `exists: false`
- **Rich metadata** - Returns element details (role, name, enabled, bounds) when found
- **Retry logic** - Supports `alternative_selectors`, `fallback_selectors`, and configurable timeout
- **Simple conditionals** - Use `{step_id}_status` or `{step_id}_result.exists` in workflow `if` expressions

**Example:**
```yaml
- tool_name: validate_element
  id: check_dialog
  selector: "role:Button|name:Leave"
  timeout_ms: 1000

- tool_name: click_element
  if: 'check_dialog_status == "success"'
  selector: "role:Button|name:Leave"
```

### Approach 2: desktop.locator() with try/catch (For Window-Scoped Checks)

Use `desktop.locator()` with try/catch when you need to scope the search to a specific window to avoid false positives.

**Advantages:**
- **Window scoping** - Can check element exists within a specific window only
- **Performance** - `.first(0)` with try/catch is ~8x faster than `.all()` (0.5s vs 10.8s)
- **Programmatic control** - Multiple checks in one script, complex conditional logic

**When to use:**
- Element must be in a specific window (e.g., Chrome dialog vs Firefox dialog)
- Need to combine multiple element checks in one script
- Require complex conditional logic beyond simple existence

### Pattern Example - Window-Scoped (Recommended)

**Step 1: Check if optional element exists in specific window**
```javascript
{
  "tool_name": "run_command",
  "arguments": {
    "engine": "javascript",
    "run": "try {\n  const chromeWindow = await desktop.locator('role:Window|name:Chrome').first();\n  await chromeWindow.locator('role:Button|name:Leave').first();\n  return JSON.stringify({ dialog_exists: 'true' });\n} catch (e) {\n  return JSON.stringify({ dialog_exists: 'false' });\n}"
  }
}
```

### Pattern Example - Desktop-Wide Search

**Step 1: Check if optional element exists anywhere**
```javascript
{
  "tool_name": "run_command",
  "arguments": {
    "engine": "javascript",
    "run": "try {\n  await desktop.locator('role:Button|name:Leave').first();\n  return JSON.stringify({ dialog_exists: 'true' });\n} catch (e) {\n  return JSON.stringify({ dialog_exists: 'false' });\n}"
  }
}
```

**Step 2: In workflow YAML, use conditional execution**
```yaml
- tool_name: click_element
  if: 'dialog_exists == "true"'
  arguments:
    selector: "role:Button|name:Leave"
```

### Important Scoping Pattern

- **`desktop.locator()`** - Searches ALL windows/applications (desktop-wide)
- **`element.locator()`** - Searches only within that element's subtree
- **Always scope to specific window** when checking for window-specific dialogs to avoid false positives

### Common Use Cases

- Confirmation dialogs ("Are you sure?", "Unsaved changes", "Leave page")
- Session/login dialogs that depend on authentication state
- Browser restore prompts
- Password save dialogs
- Cookie consent banners
- Any conditionally-appearing UI element

### Complete Workflow Example

```yaml
# Step 1: Check for optional "Leave" button dialog (window-scoped)
- tool_name: run_command
  id: check_leave_dialog
  arguments:
    engine: javascript
    run: |
      try {
        // Scope to specific window to avoid false positives
        const chromeWindow = await desktop.locator('role:Window|name:Chrome').first();
        await chromeWindow.locator('role:Button|name:Leave').first();

        return JSON.stringify({
          dialog_exists: "true"
        });
      } catch (e) {
        return JSON.stringify({
          dialog_exists: "false"
        });
      }

# Step 2: Click button only if dialog exists
- tool_name: click_element
  id: click_leave
  if: 'dialog_exists == "true"'
  arguments:
    selector: "role:Button|name:Leave"
    timeout_ms: 3000
```

### Choosing Between validate_element and desktop.locator()

**✅ Use validate_element (simpler, preferred for most cases):**
```yaml
- tool_name: validate_element
  id: check_dialog
  selector: "role:Button|name:Leave"
  timeout_ms: 1000

- tool_name: click_element
  if: 'check_dialog_status == "success"'
  selector: "role:Button|name:Leave"
```
**Advantages:** Built-in, no JavaScript, returns `status: "success"/"failed"` with `exists: true/false`, includes element metadata

**✅ Use desktop.locator() (for window-scoped checks):**
```yaml
- tool_name: run_command
  id: check_dialog_in_chrome
  arguments:
    engine: javascript
    run: |
      try {
        const chromeWindow = await desktop.locator('role:Window|name:Chrome').first(0);
        await chromeWindow.locator('role:Button|name:Leave').first(0);
        return JSON.stringify({ exists: "true" });
      } catch (e) {
        return JSON.stringify({ exists: "false" });
      }

- tool_name: click_element
  if: 'exists == "true"'
  selector: "role:Button|name:Leave"
```
**Advantages:** Window scoping prevents false positives (e.g., finding button in wrong window), faster (0.5s), allows complex multi-check logic

## Error Handling

Commands that fail will return:
- Non-zero `exit_status`
- Error details in `stderr`
- Original command information for debugging

## Passing Data Between Workflow Steps (Engine Mode Only)

When using `engine` mode (JavaScript or Python), you can pass data between workflow steps using the `set_env` mechanism. This allows subsequent steps to access data from previous steps. Additionally, you can now inject environment variables directly into scripts using the `env` parameter.

### How It Works

**Important:** The `set_env` mechanism and `env` parameter only work when using the `engine` parameter with JavaScript or Python. They do NOT work with shell commands.

### Injecting Environment Variables into Scripts

Environment variables can come from two sources:

1. **CLI inputs** - Passed via `--inputs` parameter when running the workflow
2. **Tool-specific env** - Passed directly to the tool via `env` parameter

Both are merged and available in your scripts, with tool-specific values taking precedence.

#### ⚠️ Critical: Variable Declaration Safety

Terminator injects environment variables using `var` declarations at the beginning of your script. This can cause "variable already declared" errors if your code tries to redeclare them with `const` or `let`.

**Always use the typeof check pattern to safely access variables:**

```javascript
// ✅ CORRECT - Safe variable access
const myVar = (typeof env_var_name !== 'undefined') ? env_var_name : 'default_value';
const isActive = (typeof is_active !== 'undefined') ? is_active === 'true' : false;
const errorMsg = (typeof error_message !== 'undefined' && error_message !== null) ? error_message : '';
const count = parseInt(retry_count || '0');

// ❌ WRONG - Will fail if variable was already declared with var
const myVar = env_var_name;  // Error: env_var_name already declared
let isActive = is_active === 'true';  // Error: is_active already declared
```

This pattern works whether:
- The variable exists or doesn't exist
- Terminator's smart replacement succeeds or fails
- The variable is used in any scope (global, function, block)

#### Example: Using CLI Inputs

Run workflow with inputs:
```bash
terminator mcp run workflow.yml --inputs '{"api_key":"sk-123","user":"alice"}'
```

Access in your script:
```javascript
{
  "engine": "javascript",
  "run": "// Variables are directly available\nconsole.log(`User ${user} with key ${api_key}`); return { authenticated: true };"
}
```

#### Example: Combining CLI Inputs and Tool Env

Use the `env` parameter to pass additional data or override CLI inputs:

```javascript
{
  "engine": "javascript",
  "script_file": "process.js",
  "env": {
    "api_endpoint": "https://api.example.com",
    "max_retries": 3,
    "user_data": { "name": "John", "id": 123 }
  }
}
```

In your script, access both CLI inputs and tool env:
```javascript
// process.js
// Use typeof checks to safely access variables
// api_key comes from CLI --inputs
// api_endpoint comes from tool's env parameter
const apiKey = (typeof api_key !== 'undefined') ? api_key : '';
const apiEndpoint = (typeof api_endpoint !== 'undefined') ? api_endpoint : 'https://api.example.com';
const maxRetries = (typeof max_retries !== 'undefined') ? max_retries : 3;
const userData = (typeof user_data !== 'undefined') ? user_data : { name: 'Unknown' };

console.log(`Using API key: ${apiKey}`);
console.log(`Connecting to: ${apiEndpoint}`);
console.log(`Max retries: ${maxRetries}`);
console.log(`User: ${userData.name}`);  // Objects are already parsed
```

### Setting Environment Variables

There are two ways to set environment variables for subsequent steps:

#### Method 1: Return Object with set_env
```javascript
{
  "engine": "javascript",
  "run": "const data = { name: 'John', age: 30 };\nreturn { set_env: { user_data: JSON.stringify(data) } };"
}
```

#### Method 2: GitHub Actions Style Console Output
```javascript
{
  "engine": "javascript", 
  "run": "const filePath = 'C:\\\\Users\\\\file.txt';\nconsole.log(`::set-env name=file_path::${filePath}`);"
}
```

### Accessing Environment Variables in Subsequent Steps

Use the `{{env.variable_name}}` syntax in your workflow steps:

```javascript
{
  "engine": "javascript",
  "run": "const filePath = '{{env.file_path}}';\nconsole.log(`Processing file: ${filePath}`);"
}
```

### Complete Example: Reading and Moving Files

```json
{
  "steps": [
    {
      "tool_name": "run_command",
      "arguments": {
        "engine": "javascript",
        "run": "const { execSync } = require('child_process');\n\n// Find JSON file\nconst folder = 'C:\\\\data';\nconst result = execSync(`powershell -Command \\\"Get-ChildItem '${folder}' -Filter '*.json' | Select-Object -First 1 | ConvertTo-Json\\\"`, { encoding: 'utf8' });\nconst fileInfo = JSON.parse(result);\n\n// Set env vars for next step\nconsole.log(`::set-env name=file_path::${fileInfo.FullName}`);\nconsole.log(`::set-env name=file_name::${fileInfo.Name}`);\n\nreturn { status: 'found', file: fileInfo.Name };"
      }
    },
    {
      "tool_name": "run_command",
      "arguments": {
        "engine": "javascript",
        "run": "// Access data from previous step\nconst filePath = '{{env.file_path}}';\nconst fileName = '{{env.file_name}}';\n\nconsole.log(`Moving ${fileName} to processed folder...`);\n\nconst { execSync } = require('child_process');\nexecSync(`powershell -Command \\\"Move-Item '${filePath}' -Destination 'C:\\\\processed\\\\'\\\"`);\n\nreturn { status: 'moved', file: fileName };"
      }
    }
  ]
}
```

### Important Limitations

1. **Backslash Escaping**: When passing Windows file paths, backslashes may need extra escaping:
   ```javascript
   // Original: C:\Users\file.txt
   // May need: C:\\Users\\file.txt or C:\\\\Users\\\\file.txt
   const escapedPath = filePath.replace(/\\/g, '\\\\');
   ```

2. **Variable Substitution**: The `{{env.variable}}` substitution happens before the JavaScript executes, so:
   - Variables must be set in a previous step
   - Complex data needs JSON.stringify() when setting to env, but is automatically parsed when reading
   - Consider combining related operations in a single step if data passing becomes complex

3. **Engine Mode Required**: Remember that `set_env` ONLY works with:
   - `engine: "javascript"` (or `"js"`, `"node"`, `"bun"`)
   - `engine: "python"`
   - It does NOT work with shell commands (`shell: "powershell"`, etc.)

## Security Considerations

- Commands are executed with the same privileges as the MCP server
- Avoid executing untrusted input
- Use proper escaping for user-provided values
- Consider using working_directory to limit file system access
- Be careful with sensitive data in environment variables
