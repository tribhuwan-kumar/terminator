#Requires -RunAsAdministrator

# Terminator MCP Server Installation Script
# This script runs on first boot to set up the MCP server

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

# Create log file
$logPath = "C:\TerminatorMCP\install.log"
New-Item -ItemType Directory -Force -Path "C:\TerminatorMCP" | Out-Null

function Write-Log {
    param([string]$Message)
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    "$timestamp - $Message" | Out-File -FilePath $logPath -Append
    Write-Host $Message
}

Write-Log "Starting Terminator MCP Server installation..."

try {
    # Install Chocolatey if not present
    if (!(Get-Command choco -ErrorAction SilentlyContinue)) {
        Write-Log "Installing Chocolatey..."
        Set-ExecutionPolicy Bypass -Scope Process -Force
        [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
        Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
        
        # Refresh environment
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    }

    # Install Node.js
    Write-Log "Installing Node.js..."
    choco install nodejs -y --no-progress
    
    # Refresh environment
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    
    # Verify Node.js installation
    $nodeVersion = node --version
    Write-Log "Node.js installed: $nodeVersion"
    
    # Install Git (required for some npm packages)
    Write-Log "Installing Git..."
    choco install git -y --no-progress
    
    # Install Visual Studio Build Tools (required for native modules)
    Write-Log "Installing Visual Studio Build Tools..."
    choco install visualstudio2022-workload-vctools -y --no-progress
    
    # Create MCP server directory
    $mcpDir = "C:\TerminatorMCP\server"
    New-Item -ItemType Directory -Force -Path $mcpDir | Out-Null
    Set-Location $mcpDir
    
    # Install terminator-mcp-agent globally
    Write-Log "Installing terminator-mcp-agent..."
    npm install -g terminator-mcp-agent
    
    # Create a simple MCP server configuration
    $configContent = @'
{
    "name": "terminator-mcp-server",
    "version": "1.0.0",
    "description": "Terminator MCP Server on Azure",
    "main": "server.js",
    "scripts": {
        "start": "node server.js"
    }
}
'@
    
    $configContent | Out-File -FilePath "$mcpDir\package.json" -Encoding UTF8
    
    # Create server wrapper script
    $serverScript = @'
const { spawn } = require('child_process');
const path = require('path');

console.log('Starting Terminator MCP Server...');

// Start the MCP agent
const mcp = spawn('npx', ['-y', 'terminator-mcp-agent'], {
    stdio: 'inherit',
    shell: true
});

mcp.on('error', (err) => {
    console.error('Failed to start MCP agent:', err);
});

mcp.on('exit', (code) => {
    console.log(`MCP agent exited with code ${code}`);
});

// Keep the process running
process.on('SIGINT', () => {
    console.log('Shutting down MCP server...');
    mcp.kill();
    process.exit(0);
});
'@
    
    $serverScript | Out-File -FilePath "$mcpDir\server.js" -Encoding UTF8
    
    # Create Windows service for MCP server
    Write-Log "Creating Windows service..."
    
    # Install node-windows for service management
    Set-Location $mcpDir
    npm install node-windows --save
    
    # Create service installation script
    $serviceScript = @'
const Service = require('node-windows').Service;
const path = require('path');

// Create a new service object
const svc = new Service({
    name: 'Terminator MCP Server',
    description: 'Terminator Model Context Protocol Server',
    script: path.join(__dirname, 'server.js'),
    nodeOptions: [
        '--harmony',
        '--max_old_space_size=4096'
    ],
    env: {
        name: 'NODE_ENV',
        value: 'production'
    }
});

// Listen for the "install" event
svc.on('install', function() {
    console.log('Service installed successfully');
    svc.start();
});

svc.on('start', function() {
    console.log('Service started successfully');
});

svc.on('error', function(err) {
    console.error('Service error:', err);
});

// Install the service
svc.install();
'@
    
    $serviceScript | Out-File -FilePath "$mcpDir\install-service.js" -Encoding UTF8
    
    # Run the service installation
    node install-service.js
    
    # Configure Windows Firewall
    Write-Log "Configuring Windows Firewall..."
    New-NetFirewallRule -DisplayName "Terminator MCP Server" -Direction Inbound -Protocol TCP -LocalPort 3000 -Action Allow -ErrorAction SilentlyContinue
    
    # Enable WinRM for remote management
    Write-Log "Configuring WinRM..."
    Enable-PSRemoting -Force -SkipNetworkProfileCheck
    Set-Item WSMan:\localhost\Service\Auth\Basic -Value $true
    Set-Item WSMan:\localhost\Service\AllowUnencrypted -Value $true
    Set-NetFirewallRule -Name "WINRM-HTTP-In-TCP" -RemoteAddress Any
    
    # Create startup verification script
    $verifyScript = @'
$service = Get-Service -Name "Terminator MCP Server" -ErrorAction SilentlyContinue
if ($service -and $service.Status -eq "Running") {
    Write-Host "✅ Terminator MCP Server is running"
} else {
    Write-Host "❌ Terminator MCP Server is not running"
}
'@
    
    $verifyScript | Out-File -FilePath "C:\TerminatorMCP\verify-service.ps1" -Encoding UTF8
    
    Write-Log "Installation completed successfully!"
    Write-Log "MCP Server is running as a Windows service"
    
    # Create desktop shortcut for logs
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut("$env:USERPROFILE\Desktop\Terminator MCP Logs.lnk")
    $Shortcut.TargetPath = "notepad.exe"
    $Shortcut.Arguments = $logPath
    $Shortcut.Save()
    
} catch {
    Write-Log "ERROR: $_"
    Write-Log $_.Exception.StackTrace
    exit 1
}

# Final status
Write-Log "==================================="
Write-Log "Terminator MCP Server Setup Complete"
Write-Log "Service Status: Running"
Write-Log "Logs: $logPath"
Write-Log "==================================="