#!/bin/bash

# Test that telemetry doesn't block startup
echo "Testing telemetry non-blocking behavior..."
echo "Starting MCP agent (should start immediately, not wait for collector)..."

# Run with timeout and capture first few lines
timeout 3s cargo run --package terminator-mcp-agent -- -t stdio 2>&1 | head -20

echo ""
echo "Test complete. If you saw the server startup message quickly, telemetry is non-blocking."