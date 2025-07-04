#Requires -RunAsAdministrator

# Terminator MCP Server Installation Script - BUILD FROM SOURCE
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

Write-Log "Starting Terminator MCP Server installation (Build from source)..."

try {
    # Install Chocolatey
    Write-Log "Installing Chocolatey..."
    Set-ExecutionPolicy Bypass -Scope Process -Force
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
    Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

    # Install Rust
    Write-Log "Installing Rust..."
    choco install rust -y --no-progress
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

    # Install Git
    Write-Log "Installing Git..."
    choco install git -y --no-progress
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    
    # Clone Terminator repository
    Write-Log "Cloning Terminator repository..."
    Set-Location "C:\TerminatorMCP"
    git clone https://github.com/mediar-ai/terminator.git
    Set-Location "C:\TerminatorMCP\terminator"

    # Build the agent
    Write-Log "Building terminator-mcp-agent..."
    cargo build --release --package terminator-mcp-agent

    # Configure Windows Firewall
    Write-Log "Configuring Windows Firewall..."
    New-NetFirewallRule -DisplayName "Terminator MCP HTTP Server" -Direction Inbound -Protocol TCP -LocalPort 3000 -Action Allow -ErrorAction SilentlyContinue

    # Launch the agent
    Write-Log "Launching terminator-mcp-agent..."
    $agentPath = "C:\TerminatorMCP\terminator\target\release\terminator-mcp-agent.exe"
    Start-Process -FilePath $agentPath -ArgumentList "--transport", "http", "--host", "0.0.0.0"
    
    Write-Log "Installation script finished."

} catch {
    Write-Log "ERROR: $_"
    Write-Log $_.Exception.StackTrace
    exit 1
}

# Final status
Write-Log "==================================="
Write-Log "Terminator MCP Server Setup Complete"
Write-Log "Service Status: Running"
Write-Log "HTTP Endpoint: http://0.0.0.0:3000"
Write-Log "Health Check: http://localhost:3000/health"
Write-Log "Logs: $logPath"
Write-Log "==================================="