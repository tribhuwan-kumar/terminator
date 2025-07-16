# Terminator CLI

The Terminator CLI is a powerful command-line tool for managing the Terminator project, including version management, releases, **Azure VM deployment**, and **MCP server interaction**.

## Features

- 📦 **Version Management**: Bump and sync versions across all packages
- 🏷️ **Release Automation**: Tag and release with a single command
- ☁️ **Azure VM Deployment**: One-liner to deploy Windows VMs with MCP server
- 🤖 **MCP Client**: Chat with MCP servers over HTTP or stdio
- 🔄 **Workflow Execution**: Run automation workflows from YAML/JSON files
- 🔧 **Tool Execution**: Execute individual MCP tools directly
- 🔒 **Secure by Default**: Auto-generated passwords, configurable security rules

## Installation

From the workspace root:

```bash
# Build the CLI
cargo build --release --bin terminator

# Install globally (optional)
cargo install --path terminator-cli
```

## Quick Start

### Version Management

```bash
# Bump version
terminator patch      # x.y.Z+1
terminator minor      # x.Y+1.0
terminator major      # X+1.0.0

# Sync all package versions
terminator sync

# Show current status
terminator status

# Tag and push
terminator tag

# Full release (bump + tag + push)
terminator release        # patch release
terminator release minor  # minor release
```

### MCP Workflow Execution

Execute automation workflows from YAML or JSON files:

```bash
# Execute a workflow file
terminator mcp run workflow.yml

# Execute with verbose logging
terminator mcp run workflow.yml --verbose

# Dry run (validate without executing)
terminator mcp run workflow.yml --dry-run

# Use specific MCP server command
terminator mcp run workflow.yml --command "npx -y terminator-mcp-agent@latest"

# Use HTTP MCP server
terminator mcp run workflow.yml --url http://localhost:3000/mcp
```

**Example workflow file** (`workflow.yml`):
```yaml
tool_name: execute_sequence
arguments:
  steps:
    - tool_name: navigate_browser
      arguments:
        url: "https://example.com"
    - tool_name: click_element
      arguments:
        selector: "role:Button|name:Submit"
    - tool_name: get_focused_window_tree
      id: capture_result
  output_parser:
    ui_tree_source_step_id: capture_result
    item_container_definition:
      node_conditions:
        - property: role
          op: equals
          value: CheckBox
    fields_to_extract:
      name:
        from_self:
          extract_property: name
```

### MCP Tool Execution

Execute individual MCP tools directly:

```bash
# Execute a single tool
terminator mcp exec get_applications

# Execute with arguments
terminator mcp exec click_element '{"selector": "role:Button|name:OK"}'

# Use different MCP server
terminator mcp exec --url http://localhost:3000/mcp validate_element '{"selector": "#button"}'
```

### Interactive MCP Chat

Chat with MCP servers interactively:

```bash
# Start chat session (uses local MCP server by default)
terminator mcp chat

# Chat with remote MCP server
terminator mcp chat --url https://your-server.com/mcp

# Chat with specific MCP server command
terminator mcp chat --command "node my-mcp-server.js"
```

### Control Remote Computer Through Chat

1. Run the MCP server on your remote machine
2. Open port or use ngrok
3. Connect via CLI:

```bash
terminator mcp chat --url https://xxx/mcp
```

<img width="1512" alt="Screenshot 2025-07-04 at 1 49 10 PM" src="https://github.com/user-attachments/assets/95355099-0130-4702-bd11-0278db181253" />

## Advanced Usage

### MCP Server Connection Options

The CLI supports multiple ways to connect to MCP servers:

```bash
# Local MCP server (default - uses @latest for compatibility)
terminator mcp run workflow.yml

# Specific version
terminator mcp run workflow.yml --command "npx -y terminator-mcp-agent@0.9.0"

# HTTP server
terminator mcp run workflow.yml --url http://localhost:3000/mcp

# Custom server command
terminator mcp run workflow.yml --command "python my_mcp_server.py"
```

### Workflow File Formats

The CLI supports both YAML and JSON workflow files:

**Direct workflow (workflow.yml)**:
```yaml
steps:
  - tool_name: navigate_browser
    arguments:
      url: "https://example.com"
stop_on_error: true
```

**Tool call wrapper (workflow.json)**:
```json
{
  "tool_name": "execute_sequence",
  "arguments": {
    "steps": [
      {
        "tool_name": "navigate_browser",
        "arguments": {
          "url": "https://example.com"
        }
      }
    ]
  }
}
```

### Error Handling

```bash
# Continue on errors
terminator mcp run workflow.yml --continue-on-error

# Custom timeout
terminator mcp run workflow.yml --timeout 30000

# Detailed error output
terminator mcp run workflow.yml --verbose
```

## Configuration

### Environment Variables

- `RUST_LOG`: Set logging level (e.g., `debug`, `info`, `warn`, `error`)
- `MCP_SERVER_URL`: Default MCP server URL
- `MCP_SERVER_COMMAND`: Default MCP server command

### Default Behavior

- **MCP Server**: Uses `npx -y terminator-mcp-agent@latest` by default
- **Logging**: Info level by default, debug with `--verbose`
- **Error Handling**: Stops on first error by default
- **Format**: Auto-detects YAML/JSON from file extension

## Troubleshooting

### Version Mismatch Issues

If you encounter "missing field" errors, ensure you're using the latest MCP server:

```bash
# Force latest version
terminator mcp run workflow.yml --command "npx -y terminator-mcp-agent@latest"

# Clear npm cache if needed
npm cache clean --force
```

### Connection Issues

```bash
# Test MCP server connectivity
terminator mcp exec get_applications

# Use verbose logging for debugging
terminator mcp run workflow.yml --verbose

# Test with dry run first
terminator mcp run workflow.yml --dry-run
```

For more examples and advanced usage, see the [Terminator MCP Agent documentation](../terminator-mcp-agent/README.md).



