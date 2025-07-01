# Terminator CLI

The Terminator CLI is a powerful command-line tool for managing the Terminator project, including version management, releases, and **Azure VM deployment for MCP servers**.

## Features

- 📦 **Version Management**: Bump and sync versions across all packages
- 🏷️ **Release Automation**: Tag and release with a single command
- ☁️ **Azure VM Deployment**: Create Windows VMs with pre-installed MCP server
- 🤖 **MCP Server Setup**: Automated installation and configuration
- 🔒 **Secure by Default**: Auto-generated passwords, configurable security rules

## Installation

From the workspace root:

```bash
# Build the CLI
cargo build --release --bin terminator

# Install globally (optional)
cargo install --path terminator-cli
```

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
terminator release major  # major release
```

### Azure VM Deployment

#### Prerequisites

1. Install Azure CLI:
   ```bash
   # Windows
   winget install Microsoft.AzureCLI
   
   # macOS
   brew install azure-cli
   
   # Linux
   curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash
   ```

2. Login to Azure:
   ```bash
   az login
   ```

3. Set your subscription (if you have multiple):
   ```bash
   export AZURE_SUBSCRIPTION_ID="your-subscription-id"
   ```

#### Create a VM with MCP Server

```bash
# Basic deployment (uses defaults)
terminator azure create

# Custom deployment
terminator azure create \
  --resource-group my-terminator-rg \
  --vm-name my-terminator-vm \
  --location westus2 \
  --vm-size Standard_D4s_v3 \
  --admin-username myadmin \
  --save-to vm-info.json
```

#### Available Options

| Option | Description | Default |
|--------|-------------|---------|
| `--subscription-id` | Azure subscription ID | From env or prompt |
| `--resource-group` | Resource group name | `terminator-rg-XXXX` |
| `--vm-name` | Virtual machine name | `terminator-vm-XXXX` |
| `--location` | Azure region | `eastus` |
| `--vm-size` | VM size | `Standard_D2s_v3` |
| `--admin-username` | Administrator username | `terminatoradmin` |
| `--admin-password` | Administrator password | Auto-generated |
| `--no-rdp` | Disable RDP access | RDP enabled |
| `--no-winrm` | Disable WinRM access | WinRM enabled |
| `--no-mcp` | Skip MCP server installation | MCP installed |
| `--save-to` | Save connection info to file | Not saved |

#### Connect to Your VM

After deployment, you'll receive connection information:

**RDP (Remote Desktop):**
```bash
# Windows
mstsc /v:YOUR_VM_IP:3389

# macOS (use Microsoft Remote Desktop from App Store)
# Linux
xfreerdp /v:YOUR_VM_IP /u:YOUR_USERNAME /p:'YOUR_PASSWORD'
```

**PowerShell Remoting:**
```powershell
$cred = Get-Credential YOUR_USERNAME
Enter-PSSession -ComputerName YOUR_VM_IP -Credential $cred
```

#### Delete Resources

```bash
# Delete entire resource group
terminator azure delete my-terminator-rg
```

## MCP Server Details

The MCP server is automatically installed as a Windows service:

- **Service Name**: Terminator MCP Server
- **Installation Path**: `C:\TerminatorMCP\`
- **Logs**: `C:\TerminatorMCP\install.log`
- **Configuration**: Runs `terminator-mcp-agent` via npx

To verify installation after connecting to the VM:

```powershell
# Check service status
Get-Service "Terminator MCP Server"

# View installation logs
Get-Content C:\TerminatorMCP\install.log -Tail 50

# Test MCP agent
npx -y terminator-mcp-agent --version
```

## Examples

See the [`examples`](../terminator/examples) directory for complete examples:

- [`azure_vm_deployment.rs`](../terminator/examples/azure_vm_deployment.rs) - Full Azure VM deployment workflow

## Cost Management

- VMs are billed hourly - remember to delete when not in use
- Typical costs:
  - Standard_B2s: ~$30/month
  - Standard_D2s_v3: ~$96/month (default)
  - Standard_D4s_v3: ~$192/month

## Security Best Practices

1. **Restrict Network Access**: Update NSG rules to limit access to your IP
2. **Use Strong Passwords**: Auto-generated passwords meet Azure requirements
3. **Enable MFA**: Use Azure Bastion for production deployments
4. **Regular Updates**: Keep Windows and MCP server updated

## Troubleshooting

### Azure CLI Issues

```bash
# Clear cache and re-authenticate
az account clear
az login

# Verify subscription
az account show
```

### VM Creation Fails

Check Azure quotas:
```bash
az vm list-usage --location eastus --output table
```

### MCP Server Not Running

1. Connect via RDP
2. Check logs: `Get-Content C:\TerminatorMCP\install.log`
3. Restart service: `Restart-Service "Terminator MCP Server"`

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for development guidelines.

## License

MIT License - see [LICENSE](../LICENSE) for details.