# Execute Browser Script Tool Documentation

## Overview

The `execute_browser_script` tool enables direct JavaScript execution in browser contexts through the Chrome extension bridge. It provides full access to the HTML DOM for data extraction, page analysis, manipulation, and bidirectional data flow within workflows.

## Prerequisites

- **Chrome Extension**: The Terminator browser extension must be installed
- **Active Browser**: A browser window must be open and accessible
- **Valid Selector**: You need a selector to target the browser window

## Basic Usage

```javascript
execute_browser_script({
  selector: "role:Window",
  script: "document.title",
});
```

## Parameters

| Parameter     | Type   | Required | Description                                     |
| ------------- | ------ | -------- | ----------------------------------------------- |
| `selector`    | string | Yes      | UI selector to locate the browser window        |
| `script`      | string | No\*     | JavaScript code to execute                      |
| `script_file` | string | No\*     | Path to JavaScript file to load and execute     |
| `env`         | object | No       | Environment variables to inject into the script |
| `outputs`     | object | No       | Outputs from previous workflow steps to inject  |
| `timeout_ms`  | number | No       | Timeout in milliseconds (default: varies)       |
| `retries`     | number | No       | Number of retry attempts                        |

\*Either `script` or `script_file` must be provided.

## Data Injection

When `env` or `outputs` parameters are provided, they are automatically injected as JavaScript variables with proper types:

```javascript
// Variables are automatically available as proper JavaScript types
// If a value was JSON in the workflow state, it's already parsed
var env = {...};     // Already parsed - objects/arrays are ready to use
var outputs = {...}; // Already parsed - no JSON.parse needed
// Your script code follows...
```

### Smart JSON Detection

The system automatically detects and parses JSON strings into proper JavaScript objects/arrays. All environment variables are injected directly into the script scope:

```javascript
// Variables are directly available - no env prefix needed
console.log(username);        // Direct string access
console.log(userData.name);   // Direct object access
console.log(items[0]);        // Direct array access
```

## Examples

### Basic DOM Extraction

```javascript
execute_browser_script({
  selector: "role:Window",
  script: `
    ({
      title: document.title,
      url: window.location.href,
      formCount: document.forms.length,
      linkCount: document.links.length
    })
  `,
});
```

### Using Environment Variables

```javascript
// Step 1: Set variables in a previous step
run_command({
  engine: "javascript",
  run: `
    return {
      set_env: {
        searchTerm: 'terminator automation',
        maxResults: '10'
      }
    };
  `,
});

// Step 2: Use variables in browser script
execute_browser_script({
  selector: "role:Window",
  env: {
    searchTerm: "{{env.searchTerm}}",
    maxResults: "{{env.maxResults}}",
  },
  script: `
    // Variables are directly available - no env prefix needed

    // Fill search form
    const searchInput = document.querySelector('input[name="q"]');
    searchInput.value = searchTerm;  // Direct access
    searchInput.form.submit();

    JSON.stringify({
      status: 'search_submitted',
      searchTerm: searchTerm
    });
  `,
});
```

### Loading Scripts from Files

Create a reusable script file:

```javascript
// scripts/extract_table_data.js
// Variables are directly available in the script scope
const table = document.querySelector(tableName || "table");
if (!table) {
  JSON.stringify({ error: "Table not found" });
} else {
  const rows = Array.from(table.querySelectorAll("tr"));
  const data = rows.map((row) => {
    const cells = Array.from(row.querySelectorAll("td, th"));
    return cells.map((cell) => cell.textContent.trim());
  });

  JSON.stringify({
    tableData: data,
    rowCount: rows.length,
    set_env: {
      extraction_complete: "true",
      row_count: rows.length.toString(),
    },
  });
}
```

Use the script file in your workflow:

```javascript
execute_browser_script({
  selector: "role:Window",
  script_file: "scripts/extract_table_data.js",
  env: {
    tableName: "#data-table",
  },
});
```

### Bidirectional Data Flow

```javascript
// Step 1: Browser extracts data and sets variables
execute_browser_script({
  selector: "role:Window",
  script: `
    const forms = Array.from(document.forms).map(f => ({
      id: f.id,
      action: f.action,
      method: f.method
    }));
    
    JSON.stringify({
      forms: forms,
      set_env: {
        form_count: forms.length.toString(),
        first_form_id: forms[0]?.id || 'none'
      }
    });
  `,
});

// Step 2: Process the data in JavaScript
run_command({
  engine: "javascript",
  run: `
    const formCount = parseInt('{{env.form_count}}');
    const firstFormId = '{{env.first_form_id}}';
    
    console.log(\`Found \${formCount} forms\`);
    console.log(\`First form ID: \${firstFormId}\`);
    
    return {
      set_env: {
        should_submit: formCount > 0 ? 'true' : 'false'
      }
    };
  `,
});

// Step 3: Use the decision in browser
execute_browser_script({
  selector: "role:Window",
  env: {
    shouldSubmit: "{{env.should_submit}}",
    formId: "{{env.first_form_id}}",
  },
  script: `
    // Variables are directly available

    if (shouldSubmit === 'true') {
      const form = document.getElementById(formId);
      if (form) {
        console.log('Submitting form:', formId);
        // form.submit(); // Uncomment to actually submit
      }
    }

    JSON.stringify({
      action: shouldSubmit === 'true' ? 'would_submit' : 'skipped',
      formId: formId
    });
  `,
});
```

### Handling Large DOM Responses

```javascript
execute_browser_script({
  selector: "role:Window",
  script: `
    const html = document.documentElement.outerHTML;
    const maxLength = 30000; // MCP response size limit
    
    ({
      url: window.location.href,
      title: document.title,
      htmlLength: html.length,
      html: html.length > maxLength 
        ? html.substring(0, maxLength) + '... [truncated]'
        : html,
      truncated: html.length > maxLength
    })
  `,
});
```

### Error Handling

```javascript
execute_browser_script({
  selector: "role:Window",
  script: `
    try {
      const data = document.querySelector('#data-container');
      if (!data) {
        throw new Error('Data container not found');
      }
      
      const result = {
        status: 'success',
        data: data.textContent,
        timestamp: new Date().toISOString()
      };
      
      JSON.stringify(result);
    } catch (error) {
      JSON.stringify({
        status: 'error',
        error: error.message,
        stack: error.stack
      });
    }
  `,
});
```

## Best Practices

### 1. Always Use Safe Variable Access

Due to how Terminator injects variables with `var` declarations, always use typeof checks:

```javascript
// ✅ CORRECT - Safe access pattern that prevents errors
const username = (typeof username !== 'undefined') ? username : 'guest';
const items = (typeof items !== 'undefined') ? items : [];
const userData = (typeof userData !== 'undefined') ? userData : {};

// Then use the variables normally
console.log(`User: ${username}`);
if (items.length > 0) console.log(items[0]);
if (userData.name) console.log(userData.name);

// ❌ WRONG - Direct access can cause "already declared" errors
const username = username;  // Error if username was injected
console.log(env.username);  // Wrong - no env prefix exists
```

**Note**: Variables are automatically parsed from JSON when injected, so you don't need `JSON.parse()` on incoming data.

### 2. Return JSON Strings

Browser scripts must return serializable data. Use `JSON.stringify()`:

```javascript
// ✅ Good
JSON.stringify({ status: "success", data: values });

// ❌ Bad - returns object directly
return { status: "success", data: values };
```

### 3. Use set_env for Workflow Variables

To pass data to subsequent workflow steps:

```javascript
JSON.stringify({
  result: "success",
  set_env: {
    key1: "value1",
    key2: "value2",
  },
});
```

### 4. Handle Missing Elements Gracefully

```javascript
const element = document.querySelector("#target");
if (!element) {
  JSON.stringify({ error: "Element not found", selector: "#target" });
} else {
  // Process element
}
```

### 5. Respect Size Limits

The MCP protocol has a ~30KB response limit. Truncate large data:

```javascript
const data = getLargeData();
const maxSize = 25000; // Leave buffer for wrapper JSON
const truncated = JSON.stringify(data).substring(0, maxSize);
```

## Common Patterns

### Form Filling

```javascript
execute_browser_script({
  selector: "role:Window",
  env: {
    username: "{{env.username}}",
    email: "{{env.email}}",
  },
  script: `
    // Direct access - variables are available without prefix
    document.querySelector('#username').value = username;
    document.querySelector('#email').value = email;

    JSON.stringify({ filled: true });
  `,
});
```

### Data Extraction

```javascript
execute_browser_script({
  selector: "role:Window",
  script: `
    const rows = Array.from(document.querySelectorAll('table tr'));
    const data = rows.map(row => {
      const cells = Array.from(row.querySelectorAll('td'));
      return cells.map(cell => cell.textContent.trim());
    });
    
    JSON.stringify({
      rowCount: rows.length,
      data: data,
      set_env: {
        total_rows: rows.length.toString()
      }
    });
  `,
});
```

### Navigation

```javascript
execute_browser_script({
  selector: "role:Window",
  env: {
    targetUrl: "{{env.target_url}}",
  },
  script: `
    // Direct access - variables are available without prefix
    window.location.href = targetUrl;
    JSON.stringify({ navigating: true });
  `,
});
```

## Troubleshooting

### Issue: "env is not defined" or "variable is not defined"

**Cause**: The variable was not provided through the `env` parameter or doesn't exist in the workflow state.

**Solution**: Always use typeof checks to safely access variables:

```javascript
// ✅ Safe variable access pattern
const username = (typeof username !== 'undefined') ? username : 'guest';
const items = (typeof items !== 'undefined') ? items : [];
const config = (typeof app_config !== 'undefined') ? app_config : {};

// Use the variables safely
console.log(`User: ${username}`);
if (items.length > 0) {
  console.log(`First item: ${items[0]}`);
}
```

### Issue: "Identifier 'x' has already been declared"

**Cause**: Terminator injects environment variables using `var` declarations at the top of your script. If your code tries to redeclare them with `const`, `let`, or `var`, you'll get this error.

**Solution**: Always use the typeof check pattern to avoid redeclaration:

```javascript
// ❌ WRONG - Will fail if journal_entries was injected
const journal_entries = [];  // Error: already declared

// ✅ CORRECT - Safe access that won't conflict
const journalEntries = (typeof journal_entries !== 'undefined') ? journal_entries : [];
const errorFound = (typeof error_found !== 'undefined') ? error_found === 'true' : false;
```

**Note**: Terminator has a "smart replacement" feature that tries to fix simple cases automatically by converting `const/let/var x =` to `x =`, but it's not 100% reliable. Always use typeof checks for safety.

### Issue: Script returns "[object Object]"

**Cause**: Returning an object directly instead of a JSON string.

**Solution**: Always use `JSON.stringify()`:

```javascript
// Wrong
return { data: "value" };

// Correct
JSON.stringify({ data: "value" });
```

### Issue: Large DOM truncation

**Cause**: Response exceeds MCP's ~30KB limit.

**Solution**: Implement pagination or selective extraction:

```javascript
// Extract specific parts instead of full DOM
const mainContent = document.querySelector("main").innerHTML;
const sidebarContent = document.querySelector("aside").innerHTML;

JSON.stringify({
  main: mainContent.substring(0, 10000),
  sidebar: sidebarContent.substring(0, 5000),
});
```

## Security Considerations

1. **Sensitive Data**: Be cautious when extracting passwords, tokens, or personal information
2. **XSS Prevention**: Avoid executing user-provided code directly
3. **Scope Limitation**: Scripts run in the page context and can access all page data
4. **CORS**: Scripts are subject to the same-origin policy

## Integration with Other Tools

The `execute_browser_script` tool works seamlessly with other MCP tools:

1. Use `navigate_browser` to open specific pages
2. Use `click_element` or `type_into_element` for interactions
3. Use `run_command` with engine mode to process extracted data
4. Use `wait_for_element` to ensure page readiness

## Advanced Examples

### Multi-step Data Processing

```yaml
steps:
  # Navigate to target page
  - tool_name: navigate_browser
    arguments:
      url: "https://example.com/data"

  # Wait for content to load
  - tool_name: wait_for_element
    arguments:
      selector: "role:Table"
      condition: "visible"

  # Extract and process data
  - tool_name: execute_browser_script
    arguments:
      selector: "role:Window"
      script_file: "scripts/extract_complex_data.js"
      env:
        dateFormat: "{{env.date_format}}"
        filterCriteria: "{{env.filter_criteria}}"

  # Process extracted data
  - tool_name: run_command
    arguments:
      engine: "javascript"
      run: |
        const extractedData = '{{env.extracted_data}}';
        const parsed = JSON.parse(extractedData);

        // Process and transform data
        const processed = parsed.map(item => ({
          ...item,
          processed: true,
          timestamp: new Date().toISOString()
        }));

        return {
          set_env: {
            processed_count: processed.length.toString(),
            processed_data: JSON.stringify(processed)
          }
        };
```

This comprehensive documentation covers all aspects of the enhanced `execute_browser_script` tool, providing users with the knowledge needed to leverage its full capabilities.
