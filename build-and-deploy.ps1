# Build and Deploy Script for Terminator MCP Agent
# This script builds the MCP agent and deploys it to the mediar bin location

param(
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

Write-Host "Building terminator-mcp-agent..." -ForegroundColor Cyan

# Build the MCP agent in release mode
cargo build --release --bin terminator-mcp-agent

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Build completed successfully" -ForegroundColor Green

# Define paths
$SourceBinary = "target\release\terminator-mcp-agent.exe"
$MediarBinPath = "C:\Users\screenpipe-windows\AppData\Local\mediar\bin\terminator-mcp-agent.exe"

# Verify source binary exists
if (-not (Test-Path $SourceBinary)) {
    Write-Host "Built binary not found at: $SourceBinary" -ForegroundColor Red
    exit 1
}

Write-Host "Deploying binary to mediar location..." -ForegroundColor Cyan

# Ensure destination directory exists
$MediarBinDir = Split-Path $MediarBinPath -Parent
if (-not (Test-Path $MediarBinDir)) {
    Write-Host "Creating directory: $MediarBinDir" -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $MediarBinDir -Force | Out-Null
}

# Copy the binary
Copy-Item $SourceBinary $MediarBinPath -Force

if ($?) {
    Write-Host "Binary deployed to: $MediarBinPath" -ForegroundColor Green
} else {
    Write-Host "Failed to deploy binary" -ForegroundColor Red
    exit 1
}

# Verify the copied file
if (Test-Path $MediarBinPath) {
    $FileInfo = Get-Item $MediarBinPath
    $SizeMB = [math]::Round($FileInfo.Length / 1MB, 2)
    Write-Host "Deployed binary size: $SizeMB MB" -ForegroundColor Cyan
    Write-Host "Last modified: $($FileInfo.LastWriteTime)" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "Build and deployment completed successfully!" -ForegroundColor Green
Write-Host "  Source: $SourceBinary" -ForegroundColor Gray
Write-Host "  Destination: $MediarBinPath" -ForegroundColor Gray
