# Terminator CLI

The Terminator CLI is a powerful command-line tool for managing the Terminator project, including version management, releases, **Azure VM deployment**, and **MCP server interaction**.

## Features

- 📦 **Version Management**: Bump and sync versions across all packages
- 🏷️ **Release Automation**: Tag and release with a single command
- ☁️ **Azure VM Deployment**: One-liner to deploy Windows VMs with MCP server
- 🤖 **MCP Client**: Chat with MCP servers over HTTP or stdio
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

### 🚀 One-Liner Azure VM + MCP Deployment

```bash
# Prerequisites: Azure CLI installed and logged in
az login

# Deploy VM with MCP server in one command
terminator azure create --subscription-id YOUR_SUB_ID --save-to vm.json

# Chat with the deployed MCP server (wait ~2-3 minutes for VM to boot)
terminator mcp chat --url http://$(jq -r .public_ip vm.json):3000
```

That's it! You now have a Windows VM running the Terminator MCP server.

## Usage

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

### Azure VM Deployment

#### Prerequisites

```bash
# Install Azure CLI
# Windows: winget install Microsoft.AzureCLI
# macOS: brew install azure-cli
# Linux: curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash

# Login to Azure
az login
```

#### Deploy VM with MCP Server

```bash
# Basic deployment (auto-generates secure password)
terminator azure create --subscription-id YOUR_SUBSCRIPTION_ID

# Custom deployment
terminator azure create \
  --subscription-id YOUR_SUBSCRIPTION_ID \
  --resource-group my-rg \
  --vm-name my-vm \
  --location westus2 \
  --vm-size Standard_D4s_v3 \
  --save-to vm-info.json
```

The VM will:
- Install Node.js, Git, and build tools
- Deploy MCP server as a Windows service
- Open port 3000 for HTTP access
- Start automatically on boot

#### Delete Resources

```bash
terminator azure delete RESOURCE_GROUP_NAME --subscription-id YOUR_SUB_ID
```

### MCP Client

The MCP client supports two transport modes:
- **HTTP**: Connect to remote MCP servers
- **stdio**: Launch and connect to local MCP servers

#### Interactive Chat Mode

```bash
# Connect via HTTP
terminator mcp chat --url http://VM_IP:3000

# Connect via stdio (launches local server)
terminator mcp chat --command "npx -y terminator-mcp-agent"

# In chat mode, you can:
# - Type tool names with arguments
# - Type 'help' to see all tools
# - Type 'exit' to quit

# Examples:
> get_desktop_info
> open_application notepad
> type_text "Hello from Terminator!"
> take_screenshot
```

#### Execute Single Command

```bash
# Via HTTP
terminator mcp exec --url http://VM_IP:3000 get_desktop_info

# Via stdio
terminator mcp exec --command "npx -y terminator-mcp-agent" get_desktop_info

# With arguments
terminator mcp exec --url http://VM_IP:3000 open_application notepad
terminator mcp exec --url http://VM_IP:3000 type_text "Hello World"

# With JSON arguments
terminator mcp exec --url http://VM_IP:3000 click '{"x": 100, "y": 200}'
```

## Complete Workflow Example

```bash
# 1. Deploy VM with MCP server
terminator azure create --subscription-id $AZURE_SUBSCRIPTION_ID --save-to vm.json

# 2. Wait for VM to boot (2-3 minutes)
echo "Waiting for VM to initialize..."
sleep 180

# 3. Get the VM IP
VM_IP=$(jq -r .public_ip vm.json)
MCP_URL="http://$VM_IP:3000"

# 4. Test MCP connection
terminator mcp exec --url $MCP_URL get_desktop_info

# 5. Start interactive session
terminator mcp chat --url $MCP_URL

# 6. Clean up when done
terminator azure delete $(jq -r .resource_group vm.json) --subscription-id $AZURE_SUBSCRIPTION_ID
```

## Local Development with MCP

For local development, you can use the stdio transport:

```bash
# Chat with local MCP agent
terminator mcp chat --command "npx -y terminator-mcp-agent"

# Execute single command
terminator mcp exec --command "npx -y terminator-mcp-agent" list_applications

# Use a different MCP server
terminator mcp chat --command "npx -y @modelcontextprotocol/server-everything"
```

## MCP Server Details

When deployed on Azure, the MCP server:
- Runs on port 3000 (HTTP)
- Provides health endpoint: `http://VM_IP:3000/health`
- Supports JSON-RPC 2.0 protocol
- Logs to: `C:\TerminatorMCP\install.log`

## Security Notes

- VMs are accessible from the internet by default
- Auto-generated passwords meet Azure complexity requirements
- Consider restricting NSG rules to your IP address
- For production, use Azure Bastion instead of direct RDP

## Cost Management

- VMs are billed hourly
- Default VM size (Standard_D2s_v3): ~$96/month
- Remember to delete resources when not in use
- Use `--no-wait` flag for faster deletion

## Troubleshooting

### Azure Issues

```bash
# Not logged in
az login

# Wrong subscription
az account set --subscription YOUR_SUB_ID

# Check quotas
az vm list-usage --location eastus --output table
```

### MCP Connection Issues

```bash
# Check if server is running (HTTP)
curl http://VM_IP:3000/health

# Enable debug logging
export RUST_LOG=debug
terminator mcp chat --url http://VM_IP:3000

# Check firewall on VM (via RDP)
Get-NetFirewallRule -DisplayName "*MCP*"

# Check service status
Get-Service "Terminator MCP Server"
```

## License

MIT License - see [LICENSE](../LICENSE) for details.