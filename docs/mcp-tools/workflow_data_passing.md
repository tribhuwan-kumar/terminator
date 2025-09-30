# Workflow Data Passing Guide

## Overview

This guide explains how to pass data in Terminator MCP workflows:
1. **Initial inputs from CLI** - Pass values when starting a workflow via `--inputs` parameter
2. **Between workflow steps** - Use the `set_env` mechanism to pass data from one step to the next

## Passing Initial Inputs from CLI

When running a workflow from the command line, you can provide initial input values using the `--inputs` parameter:

```bash
# Pass inputs as JSON
terminator mcp run workflow.yml --inputs '{"username":"alice","api_key":"sk-123","debug":true}'
```

These inputs become available in your workflow as environment variables:

### Accessing CLI Inputs in JavaScript

```yaml
steps:
  - tool_name: run_command
    arguments:
      engine: javascript
      run: |
        // Variables are directly available as proper types
        console.log(`Username: ${username}`);
        console.log(`API Key: ${api_key}`);
        console.log(`Debug mode: ${debug}`);

        // All inputs are also available as an object
        console.log(`All inputs:`, JSON.stringify(inputs));

        // Use inputs in your logic
        if (debug) {
          console.log("Debug mode is enabled");
        }

        return {
          authenticated_user: username,
          debug_enabled: debug
        };
```

### Accessing CLI Inputs in Python

```yaml
steps:
  - tool_name: run_command
    arguments:
      engine: python
      run: |
        # Variables are available in the global scope
        print(f"Username: {username}")
        print(f"API Key: {api_key}")
        print(f"Debug mode: {debug}")

        # All inputs are also available as a dict
        print(f"All inputs: {inputs}")

        # Use in logic
        if debug:
            print("Debug mode is enabled")

        return {
            "authenticated_user": username,
            "debug_enabled": debug
        }
```

### Input Priority

When both CLI inputs and workflow-defined inputs exist:
- **CLI inputs take precedence** - Values from `--inputs` override defaults in the workflow
- **Smart type detection** - JSON strings are automatically parsed into proper objects/arrays
- **Direct access** - Each input is available directly without any prefix (e.g., `username`, not `env.username`)

## Passing Data Between Steps

This section explains how to pass data between steps in workflows using the `set_env` mechanism.

## Prerequisites

- **Engine Mode Required**: The `set_env` mechanism works with:
  - `run_command` tool when using the `engine` parameter with JavaScript or Python
  - `execute_browser_script` tool (always returns results that can include `set_env`)
- **Not Available for Shell Commands**: Regular shell commands (using `shell` parameter) cannot use `set_env`

## How It Works

When a workflow step executes with `engine` mode, it can set environment variables that subsequent steps can access. This creates a data pipeline through your workflow.

### Workflow Execution Context

1. Each workflow maintains an execution context with an `env` object
2. Steps can add or update values in this `env` object
3. Subsequent steps access these values through variable substitution

## Setting Environment Variables

### Method 1: Return Object with set_env (Recommended)

```javascript
{
  "tool_name": "run_command",
  "arguments": {
    "engine": "javascript",
    "run": "const data = { name: 'John', age: 30 };\n\n// Return set_env object\nreturn {\n  set_env: {\n    user_name: data.name,\n    user_age: data.age.toString(),\n    user_data: JSON.stringify(data)\n  },\n  status: 'success'\n};"
  }
}
```

### Method 2: GitHub Actions Style Console Output

```javascript
{
  "tool_name": "run_command",
  "arguments": {
    "engine": "javascript",
    "run": "const filePath = 'C:\\\\Users\\\\document.pdf';\nconst fileSize = 2048;\n\n// Use console.log with special syntax\nconsole.log(`::set-env name=file_path::${filePath}`);\nconsole.log(`::set-env name=file_size::${fileSize}`);\n\nreturn { status: 'variables_set' };"
  }
}
```

### Method 3: Combining Both Methods

You can use both methods in the same step for redundancy:

```javascript
{
  "tool_name": "run_command",
  "arguments": {
    "engine": "javascript",
    "run": "const result = { count: 5, items: ['a', 'b', 'c'] };\n\n// Set using console.log\nconsole.log(`::set-env name=item_count::${result.count}`);\n\n// Also return set_env\nreturn {\n  set_env: {\n    item_count: result.count.toString(),\n    items_json: JSON.stringify(result.items)\n  },\n  result: result\n};"
  }
}
```

## Accessing Environment Variables

### Variable Substitution Syntax

Use `{{env.variable_name}}` to access environment variables in subsequent steps:

```javascript
{
  "tool_name": "run_command",
  "arguments": {
    "engine": "javascript",
    "run": "// Template substitution happens before execution\n// If user_data was JSON.stringify'd when set, it arrives as a valid JSON string\nconst filePath = '{{env.file_path}}';\nconst fileSize = parseInt('{{env.file_size}}');\nconst userData = JSON.parse('{{env.user_data}}');  // Parse if it was stringified when set\n\nconsole.log(`Processing file: ${filePath}`);\nconsole.log(`Size: ${fileSize} bytes`);\nconsole.log(`User: ${userData.name}`);\n\nreturn { processed: true };"
  }
}
```

## Complete Examples

### Example 1: File Processing Workflow

```yaml
steps:
  # Step 1: Find and read file
  - tool_name: run_command
    arguments:
      engine: javascript
      run: |
        const { execSync } = require('child_process');

        // Find JSON files
        const folder = 'C:\\Users\\data';
        const psCmd = `Get-ChildItem '${folder}' -Filter '*.json' | Select-Object -First 1 | ConvertTo-Json`;
        const result = execSync(`powershell -Command "${psCmd}"`, { encoding: 'utf8' });

        if (!result) {
          return { status: 'no_files_found' };
        }

        const fileInfo = JSON.parse(result);
        const filePath = fileInfo.FullName;
        const fileName = fileInfo.Name;

        // Read file content
        const content = execSync(`powershell -Command "Get-Content '${filePath}' -Raw"`, { encoding: 'utf8' });
        const data = JSON.parse(content);

        // Pass data to next step
        return {
          set_env: {
            file_path: filePath,
            file_name: fileName,
            entry_count: data.entries ? data.entries.length.toString() : '0'
          },
          status: 'file_read',
          entries: data.entries ? data.entries.length : 0
        };

  # Step 2: Process the file
  - tool_name: run_command
    arguments:
      engine: javascript
      run: |
        // Access data from step 1
        const filePath = '{{env.file_path}}';
        const fileName = '{{env.file_name}}';
        const entryCount = parseInt('{{env.entry_count}}');

        console.log(`Processing ${fileName}`);
        console.log(`Path: ${filePath}`);
        console.log(`Entries to process: ${entryCount}`);

        // Process entries...
        for (let i = 0; i < entryCount; i++) {
          console.log(`Processing entry ${i + 1}/${entryCount}`);
        }

        // Set status for next step
        return {
          set_env: {
            process_status: 'completed',
            processed_file: fileName
          },
          status: 'processed'
        };

  # Step 3: Move to processed folder
  - tool_name: run_command
    arguments:
      engine: javascript
      run: |
        const { execSync } = require('child_process');

        // Get file info from previous steps
        const filePath = '{{env.file_path}}';
        const fileName = '{{env.processed_file}}';
        const status = '{{env.process_status}}';

        if (status !== 'completed') {
          return { status: 'skip_move', reason: 'Processing not completed' };
        }

        // Move file
        const destination = `C:\\Users\\data\\processed\\${fileName}`;
        execSync(`powershell -Command "Move-Item '${filePath}' -Destination '${destination}' -Force"`);

        console.log(`File moved to: ${destination}`);
        return { status: 'moved', file: fileName };
```

### Example 2: Multi-Step Data Collection

```yaml
steps:
  # Collect system info
  - tool_name: run_command
    arguments:
      engine: javascript
      run: |
        const os = require('os');

        const systemInfo = {
          hostname: os.hostname(),
          platform: os.platform(),
          memory: Math.round(os.totalmem() / 1024 / 1024 / 1024)
        };

        return {
          set_env: {
            system_hostname: systemInfo.hostname,
            system_platform: systemInfo.platform,
            system_memory_gb: systemInfo.memory.toString()
          },
          info: systemInfo
        };

  # Collect application info
  - tool_name: get_applications
    arguments: {}

  # Generate report using collected data
  - tool_name: run_command
    arguments:
      engine: javascript
      run: |
        // Access all collected data
        const hostname = '{{env.system_hostname}}';
        const platform = '{{env.system_platform}}';
        const memory = '{{env.system_memory_gb}}';

        const report = {
          timestamp: new Date().toISOString(),
          system: {
            hostname: hostname,
            platform: platform,
            memory_gb: parseInt(memory)
          },
          status: 'collected'
        };

        console.log('System Report:');
        console.log(JSON.stringify(report, null, 2));

        return { report: report };
```

## Important Limitations and Workarounds

### 1. Backslash Escaping Issues

**Problem**: Windows file paths with backslashes may not survive variable substitution correctly.

**Solution**: Double-escape backslashes when setting environment variables:

```javascript
// Original path: C:\Users\file.txt
const filePath = "C:\\Users\\file.txt";

// Escape for set_env
const escapedPath = filePath.replace(/\\/g, "\\\\");

return {
  set_env: {
    file_path: escapedPath, // Will be: C:\\Users\\file.txt
  },
};
```

### 2. Complex Data Structures

**Note**: Environment variables can only store strings, so complex objects must be stringified when setting.

**Solution**: JSON stringify complex objects when setting, parse when reading via template substitution:

```javascript
// Step 1: Stringify complex data when setting
const data = { users: ["Alice", "Bob"], count: 2 };
return {
  set_env: {
    user_data: JSON.stringify(data),  // Must stringify when setting
  },
};

// Step 2: Parse when using template substitution
// Template substitution replaces {{env.user_data}} with the JSON string
const userData = JSON.parse('{{env.user_data}}');
console.log(`Users: ${userData.users.join(", ")}`);
```

### 3. Variable Not Found

**Problem**: If a variable doesn't exist, the literal `{{env.variable}}` string appears.

**Solution**: Check for substitution failure:

```javascript
const value = "{{env.might_not_exist}}";

if (value.startsWith("{{env.")) {
  console.log("Variable was not set in previous steps");
  // Use default value or handle error
  const defaultValue = "default";
} else {
  console.log(`Value: ${value}`);
}
```

### 4. Shell Commands Can't Use set_env

**Problem**: Regular shell commands can't set environment variables for the workflow.

**Solution**: Use JavaScript with `execSync` to run shell commands:

```javascript
{
  "engine": "javascript",
  "run": "const { execSync } = require('child_process');\n\n// Run shell command\nconst result = execSync('dir C:\\\\', { encoding: 'utf8' });\n\n// Now you can use set_env\nreturn {\n  set_env: {\n    dir_output: result.substring(0, 100)\n  }\n};"
}
```

## Best Practices

1. **Always Use Engine Mode**: Remember that `set_env` only works with `engine: "javascript"` or `engine: "python"`

2. **Validate Data**: Check if variables were properly substituted before using them

3. **JSON Handling**:
   - **When setting**: Use `JSON.stringify()` for complex objects/arrays
   - **When reading via env parameter**: Variables are automatically parsed - use directly
   - **When reading via template substitution**: Use `JSON.parse()` if the value was stringified

4. **Escape Special Characters**: Particularly important for Windows file paths

5. **Combine Steps When Possible**: If data passing becomes too complex, consider combining related operations into a single step

6. **Document Your Variables**: Add comments explaining what each environment variable contains

7. **Error Handling**: Always check if critical variables exist before using them

## Browser Script Example

The `execute_browser_script` tool can also set environment variables:

```javascript
{
  "tool_name": "execute_browser_script",
  "arguments": {
    "selector": "role:Window",
    "script": "// Extract data from the page\nconst pageData = {\n  title: document.title,\n  url: window.location.href,\n  formCount: document.forms.length\n};\n\n// Return data and set environment variables\nJSON.stringify({\n  pageData: pageData,\n  set_env: {\n    page_title: pageData.title,\n    page_url: pageData.url,\n    form_count: pageData.formCount.toString()\n  }\n});"
  }
}
```

### Passing Data TO Browser Scripts

The `execute_browser_script` tool can receive data through the `env` parameter:

```yaml
steps:
  # Step 1: Set some data
  - tool_name: run_command
    arguments:
      engine: javascript
      run: |
        return {
          set_env: {
            search_term: 'automation testing',
            max_results: '50'
          }
        };

  # Step 2: Use the data in browser script
  - tool_name: execute_browser_script
    arguments:
      selector: "role:Window"
      env:
        searchTerm: "{{env.search_term}}"
        maxResults: "{{env.max_results}}"
      script: |
        // Variables are directly available as proper types
        // No parsing needed - smart JSON detection handles it

        // Use the data directly
        const searchBox = document.querySelector('input[type="search"]');
        searchBox.value = searchTerm;

        // Return results
        JSON.stringify({
          status: 'search_configured',
          searchTerm: searchTerm,
          set_env: {
            search_executed: 'true'
          }
        });

  # Step 3: Check if search was executed
  - tool_name: run_command
    arguments:
      engine: javascript
      run: |
        const searchExecuted = '{{env.search_executed}}';
        console.log(`Search executed: ${searchExecuted}`);
        return { status: 'workflow_complete' };
```

## Python Example

The `set_env` mechanism also works with Python:

```python
{
  "tool_name": "run_command",
  "arguments": {
    "engine": "python",
    "run": "import json\nimport os\n\n# Collect data\ndata = {\n    'user': os.environ.get('USERNAME', 'unknown'),\n    'path': os.getcwd()\n}\n\n# Return set_env\nresult = {\n    'set_env': {\n        'current_user': data['user'],\n        'working_dir': data['path']\n    },\n    'status': 'collected'\n}\n\nprint(json.dumps(result))\nreturn result"
  }
}
```

## Troubleshooting

### Issue: Variables not passing between steps

- **Check**: Is `engine` parameter set to "javascript" or "python"?
- **Check**: Is the return object properly formatted with `set_env` key?
- **Check**: Are you using the correct substitution syntax `{{env.variable_name}}`?

### Issue: Backslashes disappearing

- **Solution**: Double-escape backslashes: `path.replace(/\\/g, '\\\\')`

### Issue: Complex data not passing correctly

- **Solution**:
  - Use `JSON.stringify()` when setting complex data to env
  - When using `env` parameter: Data is automatically parsed - use directly
  - When using template substitution `{{env.var}}`: Use `JSON.parse()` if it was stringified

### Issue: Variable shows as literal string

- **Cause**: Variable was not set in any previous step
- **Solution**: Add error checking or ensure the variable is set

## Summary

The `set_env` mechanism provides a powerful way to pass data between workflow steps, enabling complex automation scenarios. It works with:

- `run_command` tool when using `engine` mode (JavaScript/Python)
- `execute_browser_script` tool (which can both set and receive environment variables)

Key points about data handling:

- **Smart JSON detection**: When using `env` parameter, JSON strings are automatically parsed into proper JavaScript/Python objects
- **Direct variable access**: Variables are available directly without `env.` prefix
- **Setting complex data**: Use `JSON.stringify()` when setting objects/arrays to env
- **Template substitution**: When using `{{env.variable}}`, parse JSON if it was stringified
- **Handle special characters carefully** (especially backslashes in Windows paths)
- **Consider combining related operations** into a single step if data passing becomes too complex

The combination of `run_command` and `execute_browser_script` with environment variables enables sophisticated browser automation workflows with clean data flow between browser and server-side processing.
