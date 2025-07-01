# Azure VM Deployment for Terminator MCP Server

This guide explains how to use the Terminator CLI to deploy Windows VMs on Azure with the MCP server pre-installed.

## Prerequisites

1. **Azure Account**: You need an active Azure subscription
2. **Azure CLI**: Install the Azure CLI and authenticate:
   ```bash
   # Install Azure CLI (if not already installed)
   # On Windows:
   winget install Microsoft.AzureCLI
   # On macOS:
   brew install azure-cli
   # On Linux:
   curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash
   
   # Login to Azure
   az login
   ```

3. **Set Azure Subscription** (if you have multiple):
   ```bash
   # List subscriptions
   az account list --output table
   
   # Set default subscription
   az account set --subscription "YOUR_SUBSCRIPTION_ID"
   
   # Or export as environment variable
   export AZURE_SUBSCRIPTION_ID="YOUR_SUBSCRIPTION_ID"
   ```

## Quick Start

### Create a VM with MCP Server

```bash
# Basic deployment (uses defaults)
terminator azure create --subscription-id YOUR_SUBSCRIPTION_ID

# Custom deployment
terminator azure create \
  --subscription-id YOUR_SUBSCRIPTION_ID \
  --resource-group my-terminator-rg \
  --vm-name my-terminator-vm \
  --location westus2 \
  --vm-size Standard_D4s_v3 \
  --admin-username myadmin \
  --save-to vm-info.json
```

### Available Options

| Option | Description | Default |
|--------|-------------|---------|
| `--subscription-id` | Azure subscription ID | Required (or set AZURE_SUBSCRIPTION_ID) |
| `--resource-group` | Resource group name | terminator-rg-XXXX |
| `--vm-name` | Virtual machine name | terminator-vm-XXXX |
| `--location` | Azure region | eastus |
| `--vm-size` | VM size | Standard_D2s_v3 |
| `--admin-username` | Administrator username | terminatoradmin |
| `--admin-password` | Administrator password | Auto-generated |
| `--no-rdp` | Disable RDP access | RDP enabled |
| `--no-winrm` | Disable WinRM access | WinRM enabled |
| `--no-mcp` | Skip MCP server installation | MCP installed |
| `--save-to` | Save connection info to file | Not saved |

### VM Sizes

Common VM sizes for development/testing:

- `Standard_B2s`: 2 vCPUs, 4 GB RAM (cheapest)
- `Standard_D2s_v3`: 2 vCPUs, 8 GB RAM (default)
- `Standard_D4s_v3`: 4 vCPUs, 16 GB RAM
- `Standard_D8s_v3`: 8 vCPUs, 32 GB RAM

### Regions

Popular Azure regions:

- `eastus`: East US (Virginia)
- `westus2`: West US 2 (Washington)
- `centralus`: Central US (Iowa)
- `westeurope`: West Europe (Netherlands)
- `northeurope`: North Europe (Ireland)
- `southeastasia`: Southeast Asia (Singapore)

## Connecting to Your VM

After deployment, you'll receive connection information:

### RDP Connection (Windows Remote Desktop)

```bash
# Windows
mstsc /v:YOUR_VM_IP:3389

# macOS (install Microsoft Remote Desktop from App Store)
# Linux (use remmina or xfreerdp)
xfreerdp /v:YOUR_VM_IP /u:YOUR_USERNAME /p:YOUR_PASSWORD
```

### WinRM Connection (PowerShell Remoting)

```powershell
# From Windows PowerShell
$cred = Get-Credential YOUR_USERNAME
Enter-PSSession -ComputerName YOUR_VM_IP -Credential $cred

# Or use saved credentials
$password = ConvertTo-SecureString "YOUR_PASSWORD" -AsPlainText -Force
$cred = New-Object System.Management.Automation.PSCredential ("YOUR_USERNAME", $password)
Enter-PSSession -ComputerName YOUR_VM_IP -Credential $cred
```

## MCP Server Details

The MCP server is automatically installed as a Windows service on first boot:

- **Service Name**: Terminator MCP Server
- **Installation Path**: C:\TerminatorMCP\
- **Logs**: C:\TerminatorMCP\install.log
- **Status Check**: Run `Get-Service "Terminator MCP Server"` in PowerShell

### Verify Installation

1. Connect to the VM via RDP
2. Open PowerShell as Administrator
3. Run:
   ```powershell
   # Check service status
   Get-Service "Terminator MCP Server"
   
   # View installation logs
   Get-Content C:\TerminatorMCP\install.log -Tail 50
   
   # Test MCP agent
   npx -y terminator-mcp-agent --version
   ```

## Managing Resources

### Delete a VM and its Resources

```bash
# Delete entire resource group (removes VM and all associated resources)
terminator azure delete my-terminator-rg --subscription-id YOUR_SUBSCRIPTION_ID

# This will prompt for confirmation
```

### List Your VMs

```bash
# Using Azure CLI
az vm list --output table
az vm list --resource-group my-terminator-rg --output table
```

## Cost Management

- VMs are billed by the hour
- Remember to **stop or delete** VMs when not in use
- Use `az vm deallocate` to stop billing while preserving the VM
- Use `terminator azure delete` to completely remove resources

### Estimate Costs

- Standard_B2s: ~$30/month
- Standard_D2s_v3: ~$96/month
- Standard_D4s_v3: ~$192/month

(Prices vary by region and may change)

## Troubleshooting

### Authentication Issues

```bash
# Clear Azure CLI cache
az account clear

# Re-authenticate
az login

# Verify subscription
az account show
```

### VM Creation Fails

1. Check Azure quotas:
   ```bash
   az vm list-usage --location eastus --output table
   ```

2. Verify subscription permissions:
   ```bash
   az role assignment list --assignee $(az account show --query user.name -o tsv)
   ```

### MCP Server Not Running

1. Connect via RDP
2. Check installation log:
   ```powershell
   Get-Content C:\TerminatorMCP\install.log
   ```
3. Restart service:
   ```powershell
   Restart-Service "Terminator MCP Server"
   ```

## Security Best Practices

1. **Restrict Network Access**: After deployment, update the Network Security Group to limit access to your IP:
   ```bash
   az network nsg rule update \
     --resource-group my-terminator-rg \
     --nsg-name my-terminator-vm-nsg \
     --name RDP \
     --source-address-prefixes YOUR_IP_ADDRESS
   ```

2. **Use Strong Passwords**: The auto-generated passwords are secure, but you can specify your own

3. **Enable Azure Bastion**: For production use, consider Azure Bastion instead of direct RDP

4. **Regular Updates**: Keep Windows and the MCP server updated

## Example Workflow

```bash
# 1. Set up Azure credentials
export AZURE_SUBSCRIPTION_ID="your-subscription-id"

# 2. Create VM with MCP server
terminator azure create --save-to my-vm.json

# 3. Wait for deployment (usually 3-5 minutes)

# 4. Connect and verify
# (Use connection info from output or my-vm.json)

# 5. When done, clean up
terminator azure delete terminator-rg-1234
```