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
