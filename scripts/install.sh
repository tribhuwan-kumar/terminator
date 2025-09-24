#!/bin/bash
# Terminator One-Line Install Script
# Usage: curl -sSL https://raw.githubusercontent.com/mediar-ai/terminator/main/scripts/install.sh | bash

set -e

echo "🚀 Installing Terminator..."
echo ""

# Detect OS
OS="$(uname -s)"
case "${OS}" in
    Linux*)     OS_TYPE=linux;;
    Darwin*)    OS_TYPE=macos;;
    MINGW*|MSYS*|CYGWIN*) OS_TYPE=windows;;
    *)          echo "Unsupported OS: ${OS}"; exit 1;;
esac

# Check for required tools
check_command() {
    if ! command -v "$1" &> /dev/null; then
        echo "❌ $1 is not installed. Please install it first."
        exit 1
    fi
}

# Check prerequisites
echo "📋 Checking prerequisites..."
check_command "cargo"
check_command "node"

# Install terminator-cli via cargo
echo ""
echo "📦 Installing terminator-cli..."
cargo install terminator-cli

# Run setup
echo ""
echo "🛠️ Running setup..."
terminator setup --skip-vcredist

echo ""
echo "✅ Installation complete!"
echo ""
echo "Next steps:"
echo "  1. Test MCP chat: terminator mcp chat --command \"npx -y terminator-mcp-agent\""
echo "  2. Run examples: terminator mcp run https://raw.githubusercontent.com/mediar-ai/terminator/main/examples/notepad.yml"
echo ""
echo "For more information, visit: https://github.com/mediar-ai/terminator"