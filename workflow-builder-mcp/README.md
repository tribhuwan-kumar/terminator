# Workflow Builder MCP

Model Context Protocol (MCP) server for building and managing [Terminator](https://github.com/mediar-ai/terminator) workflow YAML files.

## Overview

This MCP provides AI agents with tools to create, read, edit, search, and validate Terminator workflow files. It's designed to work alongside the Terminator MCP, separating workflow authoring from execution concerns.

**Workflow Builder MCP** = Author, edit, validate workflows
**Terminator MCP** = Execute workflows, control desktop/browser

## Tools

### 📖 `read_workflow`
Read a workflow YAML file with line numbers (similar to Claude Code's `read` tool).

```json
{
  "file_path": "/path/to/workflow.yml"
}
```

### 📁 `list_workflows`
List all workflow files in a directory with metadata and validation status (similar to Claude Code's `glob` tool).

```json
{
  "directory": "/path/to/workflows",
  "pattern": "*.yml"  // optional
}
```

### 🔍 `search_workflows`
Search for text patterns across workflow files (similar to Claude Code's `grep` tool).

```json
{
  "directory": "/path/to/workflows",
  "pattern": "navigate_browser",
  "use_regex": false  // optional, default false
}
```

### ✏️ `edit_workflow`
Edit a workflow file using exact string replacement (similar to Claude Code's `edit` tool).

**Important**: The `old_string` must be unique in the file unless `replace_all` is true.

```json
{
  "file_path": "/path/to/workflow.yml",
  "old_string": "url: https://example.com",
  "new_string": "url: https://newsite.com",
  "replace_all": false  // optional, default false
}
```

### ✨ `create_workflow`
Create a new workflow YAML file (similar to Claude Code's `write` tool). Validates YAML syntax before creating.

```json
{
  "file_path": "/path/to/new_workflow.yml",
  "content": "tool_name: execute_sequence\narguments:\n  steps: []"
}
```

### ✅ `validate_workflow`
Validate a workflow file's YAML syntax and Terminator schema requirements.

```json
{
  "file_path": "/path/to/workflow.yml"
}
```

## Installation

### Claude Code

Install with a single command:

```bash
claude mcp add workflow-builder "npx -y @mediar-ai/workflow-builder-mcp@latest" -s user
```

### Claude Desktop / Other MCP Clients

```bash
npx -y @mediar-ai/workflow-builder-mcp
```

Or add to your MCP settings:

```json
{
  "mcpServers": {
    "workflow-builder": {
      "command": "npx",
      "args": ["-y", "@mediar-ai/workflow-builder-mcp@latest"]
    }
  }
}
```

### For HTTP Streamable Transport

Uses the MCP StreamableHTTP transport (SSE-based streaming over HTTP).

```bash
# Start HTTP server on port 3000 (default)
npx -y @mediar-ai/workflow-builder-mcp --http

# Or specify custom port
PORT=8080 npx -y @mediar-ai/workflow-builder-mcp --http
```

The server handles:
- **POST** `/mcp` - Send JSON-RPC messages
- **GET** `/mcp` - Establish SSE stream for responses
- **DELETE** `/mcp` - Close session

Configure in your MCP client:

```json
{
  "mcpServers": {
    "workflow-builder": {
      "url": "http://localhost:3000/mcp",
      "transport": "streamable-http"
    }
  }
}
```

### For Development

```bash
cd workflow-builder-mcp
npm install
npm run build

# Stdio mode (default)
npm start

# HTTP mode
npm run start:http
# or
node dist/index.js --http
```

## Workflow File Requirements

Valid Terminator workflow files must:
- Be valid YAML
- Have `tool_name: execute_sequence` at root level
- Have `arguments` object
- Have `arguments.steps` array

Example minimal workflow:

```yaml
tool_name: execute_sequence
arguments:
  steps:
    - tool_name: navigate_browser
      arguments:
        url: "https://example.com"
        browser: "chrome"
```

## Usage with AI

This MCP is designed to be used by AI agents to build RPA workflows. Example conversation:

```
AI: Let me create a new workflow for you.

[calls create_workflow with YAML content]

AI: I've created the workflow. Let me validate it.

[calls validate_workflow]

AI: The workflow is valid! Now you can execute it using the Terminator MCP.
```

## Architecture

```
┌─────────────────────────────────────┐
│       Claude Code / AI Agent         │
└────────────┬───────────┬────────────┘
             │           │
    ┌────────▼─────┐ ┌──▼──────────────┐
    │ Workflow     │ │ Terminator MCP  │
    │ Builder MCP  │ │  (Execution)    │
    └────────┬─────┘ └──┬──────────────┘
             │           │
    ┌────────▼─────┐ ┌──▼──────────────┐
    │ YAML Files   │ │ Desktop/Browser │
    └──────────────┘ └─────────────────┘
```

## License

MIT - See [LICENSE](../LICENSE) file

## Contributing

Part of the [Terminator](https://github.com/mediar-ai/terminator) project by Mediar AI.
