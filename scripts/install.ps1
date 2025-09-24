# Terminator One-Line Install Script for Windows
# Usage: iwr -useb https://raw.githubusercontent.com/mediar-ai/terminator/main/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

Write-Host "🚀 Installing Terminator..." -ForegroundColor Cyan
Write-Host ""

# Check for required tools
function Check-Command {
    param($cmd)
    if (!(Get-Command $cmd -ErrorAction SilentlyContinue)) {
        Write-Host "❌ $cmd is not installed. Please install it first." -ForegroundColor Red
        exit 1
    }
}

# Check prerequisites
Write-Host "📋 Checking prerequisites..." -ForegroundColor Yellow
Check-Command "cargo"
Check-Command "node"

# Install terminator-cli via cargo
Write-Host ""
Write-Host "📦 Installing terminator-cli..." -ForegroundColor Yellow
cargo install terminator-cli

# Run setup
Write-Host ""
Write-Host "🛠️ Running setup..." -ForegroundColor Yellow
terminator setup

Write-Host ""
Write-Host "✅ Installation complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:"
Write-Host "  1. Test MCP chat: terminator mcp chat --command `"npx -y terminator-mcp-agent`""
Write-Host "  2. Run examples: terminator mcp run https://raw.githubusercontent.com/mediar-ai/terminator/main/examples/notepad.yml"
Write-Host ""
Write-Host "For more information, visit: https://github.com/mediar-ai/terminator"