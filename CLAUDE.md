# Terminator MCP Commands

```bash
# Interactive chat with MCP server
terminator mcp chat --url http://localhost:3000
terminator mcp chat --command "npx -y terminator-mcp-agent"

# Execute a single MCP tool
terminator mcp exec --url http://localhost:3000 click_element '{"selector": "role:Button|name:Submit"}'
terminator mcp exec --command "npx -y terminator-mcp-agent" get_applications
terminator mcp exec --url http://localhost:3000 type_into_element '{"selector": "role:Edit|name:Email", "text_to_type": "user@example.com"}'
terminator mcp exec --url http://localhost:3000 press_key '{"selector": "role:Window", "key": "{Enter}"}'
terminator mcp exec --url http://localhost:3000 get_window_tree '{"pid": 12345}'
terminator mcp exec --url http://localhost:3000 navigate_browser '{"url": "https://example.com"}'
terminator mcp exec --url http://localhost:3000 take_screenshot '{"selector": "role:Window|name:Chrome"}'

# Run workflows from files or gists
terminator mcp run workflow.yml --url http://localhost:3000
terminator mcp run workflow.json --command "npx -y terminator-mcp-agent"
terminator mcp run https://gist.github.com/user/id --url http://localhost:3000

# Run with options
terminator mcp run workflow.yml --dry-run                  # Validate without executing
terminator mcp run workflow.yml --verbose                  # Verbose output
terminator mcp run workflow.yml --no-stop-on-error        # Continue on errors
terminator mcp run workflow.yml --no-retry                # Skip retry logic
terminator mcp run workflow.yml --no-detailed-results     # Minimal output

# Partial execution
terminator mcp run workflow.yml --start-from-step "step_3"
terminator mcp run workflow.yml --end-at-step "step_5"
terminator mcp run workflow.yml --start-from-step "step_2" --end-at-step "step_5"
terminator mcp run workflow.yml --follow-fallback              # Follow fallback_id beyond boundaries
terminator mcp run workflow.yml --end-at-step "step_5" --execute-jumps-at-end  # Execute jumps at boundary

# Validate workflow output
terminator mcp validate output.json
terminator mcp validate output.json --score    # Show quality score
cat output.json | terminator mcp validate      # From stdin

# With debugging
LOG_LEVEL=debug terminator mcp chat --command "npx -y terminator-mcp-agent"
```