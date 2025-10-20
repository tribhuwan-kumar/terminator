# Workflow Builder MCP - Usage Guide

## Quick Start

### Installation for Claude Desktop

Add to your Claude Desktop MCP configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "workflow-builder": {
      "command": "npx",
      "args": ["-y", "@mediar-ai/workflow-builder-mcp"]
    }
  }
}
```

### Local Development

```bash
cd workflow-builder-mcp
npm install
npm run build
npm start
```

## Tool Examples

### 1. Read a Workflow

```typescript
// AI calls this tool
read_workflow({
  file_path: "/Users/you/workflows/my_workflow.yml"
})

// Returns: File content with line numbers
```

### 2. List All Workflows in Directory

```typescript
list_workflows({
  directory: "/Users/you/workflows",
  pattern: "*.yml"  // optional
})

// Returns: Array of files with metadata and validation status
```

### 3. Search Across Workflows

```typescript
search_workflows({
  directory: "/Users/you/workflows",
  pattern: "navigate_browser",
  use_regex: false
})

// Returns: Files containing matches with line numbers
```

### 4. Edit a Workflow

**Important**: `old_string` must be unique unless `replace_all: true`

```typescript
edit_workflow({
  file_path: "/Users/you/workflows/my_workflow.yml",
  old_string: "url: https://example.com",
  new_string: "url: https://newsite.com",
  replace_all: false
})

// Returns: Success message with validation result
```

### 5. Create a New Workflow

```typescript
create_workflow({
  file_path: "/Users/you/workflows/new_workflow.yml",
  content: `tool_name: execute_sequence
arguments:
  steps:
    - tool_name: navigate_browser
      arguments:
        url: "https://example.com"
        browser: "chrome"`
})

// Returns: Success message if valid YAML
```

### 6. Validate a Workflow

```typescript
validate_workflow({
  file_path: "/Users/you/workflows/my_workflow.yml"
})

// Returns: Validation status with details
```

## AI Agent Workflow

Here's how an AI agent would use this MCP to build a workflow:

1. **Create Initial Workflow**
   ```
   AI: "I'll create a workflow to automate your login process."
   [calls create_workflow with basic structure]
   ```

2. **Validate Created Workflow**
   ```
   AI: "Let me validate the workflow structure."
   [calls validate_workflow]
   ```

3. **Edit to Add Steps**
   ```
   AI: "Now I'll add the login steps."
   [calls edit_workflow to insert new steps]
   ```

4. **Search for Similar Patterns**
   ```
   AI: "Let me check if you have other workflows using this pattern."
   [calls search_workflows to find similar code]
   ```

5. **Final Validation**
   ```
   AI: "Validating the complete workflow."
   [calls validate_workflow]
   ```

6. **Execution** (via Terminator MCP)
   ```
   AI: "The workflow is ready. I'll execute it using Terminator MCP."
   [calls Terminator MCP's execute_sequence tool]
   ```

## Design Philosophy

This MCP is inspired by Claude Code's file editing tools:

- **`read_workflow`** = Claude Code's `read` (with line numbers)
- **`edit_workflow`** = Claude Code's `edit` (exact string replacement)
- **`create_workflow`** = Claude Code's `write` (create new file)
- **`list_workflows`** = Claude Code's `glob` (find files by pattern)
- **`search_workflows`** = Claude Code's `grep` (search file contents)

The key difference: This MCP is **workflow-specific** with built-in YAML validation for Terminator workflows.

## Error Handling

### File Not Found
```json
{
  "error": "File not found: /path/to/workflow.yml"
}
```

### Invalid YAML
```json
{
  "error": "Invalid workflow YAML: Workflow must have 'tool_name: execute_sequence' at root level"
}
```

### String Not Unique (for edit_workflow)
```json
{
  "error": "String appears 3 times in file. Use replace_all: true to replace all occurrences, or provide a more unique string."
}
```

### File Already Exists (for create_workflow)
```json
{
  "error": "File already exists: /path/to/workflow.yml. Use edit_workflow to modify existing files."
}
```

## Workflow Schema Requirements

Valid Terminator workflows must have:

```yaml
tool_name: execute_sequence  # Required
arguments:                   # Required
  steps: []                  # Required (can be empty)

  # Optional but common:
  variables: {}
  inputs: {}
```

## Integration with Terminator MCP

This MCP focuses on **authoring**. For **execution**, use Terminator MCP:

```
Workflow Builder MCP → Creates/edits YAML files
Terminator MCP → Executes workflows (execute_sequence tool)
```

The AI agent orchestrates both MCPs:
1. Uses Workflow Builder MCP to create/edit workflows
2. Uses Terminator MCP to execute them
3. Uses Workflow Builder MCP to update based on results

## Tips for AI Agents

1. **Always validate after editing** - Call `validate_workflow` after `edit_workflow` or `create_workflow`
2. **Use specific strings for editing** - Make `old_string` as unique as possible to avoid ambiguity
3. **Search before creating** - Use `search_workflows` to find existing patterns before creating new workflows
4. **List before editing** - Use `list_workflows` to see what's available
5. **Read before editing** - Use `read_workflow` to see line numbers and current content

## Troubleshooting

### MCP Not Starting
```bash
# Check if node is installed
node --version

# Check if dependencies are installed
cd workflow-builder-mcp
npm install

# Rebuild if needed
npm run build

# Test manually
npm start
```

### YAML Validation Failures
- Ensure proper indentation (2 spaces)
- Check for required fields: `tool_name`, `arguments`, `steps`
- Validate YAML syntax at https://www.yamllint.com/

## Contributing

This MCP is part of the [Terminator](https://github.com/mediar-ai/terminator) project. Please submit issues and PRs to the main repository.
