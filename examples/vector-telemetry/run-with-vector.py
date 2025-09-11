#!/usr/bin/env python3
"""
Run MCP workflow with Vector collecting telemetry

This example:
1. Starts Vector to collect OTLP traces
2. Runs a workflow through MCP server
3. Vector processes and displays the telemetry in real-time

Prerequisites:
- Vector installed (https://vector.dev/docs/setup/installation/)
- MCP server built with telemetry: cargo build --release --features telemetry
- Python packages: pip install mcp
"""

import asyncio
import subprocess
import time
import os
import sys
from pathlib import Path
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
from contextlib import AsyncExitStack


def start_vector():
    """Start Vector with our configuration"""
    config_path = Path(__file__).parent / "vector.toml"
    
    print("🚀 Starting Vector telemetry collector...")
    print(f"   Config: {config_path}")
    
    # Start Vector in background
    # Adjust the command based on how Vector is installed on your system
    vector_cmd = [
        "vector",  # or "vector.exe" on Windows, or full path
        "--config", str(config_path),
        "--quiet"  # Reduce Vector's own logging
    ]
    
    try:
        process = subprocess.Popen(
            vector_cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        # Give Vector time to start
        time.sleep(2)
        
        # Check if it's running
        if process.poll() is not None:
            stdout, stderr = process.communicate()
            print(f"❌ Vector failed to start:")
            print(stderr)
            return None
            
        print("✅ Vector is running (PID: {})".format(process.pid))
        return process
        
    except FileNotFoundError:
        print("❌ Vector not found. Please install Vector first:")
        print("   https://vector.dev/docs/setup/installation/")
        print("\n   Or specify the full path to vector executable")
        return None


async def run_workflow_with_telemetry():
    """Execute a workflow that will generate telemetry"""
    exit_stack = AsyncExitStack()
    
    # Configure MCP server to send telemetry to Vector
    env = os.environ.copy()
    env["OTEL_EXPORTER_OTLP_ENDPOINT"] = "http://localhost:4318"
    env["RUST_LOG"] = "info"
    
    # Path to MCP server (adjust as needed)
    mcp_server = "../../target/release/terminator-mcp-agent"
    if sys.platform == "win32":
        mcp_server += ".exe"
    
    server_params = StdioServerParameters(
        command=mcp_server,
        args=[],
        env=env
    )
    
    print("\n📡 Connecting to MCP server with telemetry enabled...")
    
    transport = await exit_stack.enter_async_context(
        stdio_client(server_params)
    )
    
    session = await exit_stack.enter_async_context(
        ClientSession(transport[0], transport[1])
    )
    
    await session.initialize()
    print("✅ Connected to MCP server")
    
    # Define a workflow with multiple steps
    workflow_steps = [
        {
            "tool_name": "delay",
            "arguments": {"delay_ms": 300},
            "id": "warm_up"
        },
        {
            "tool_name": "get_applications",
            "id": "list_apps"
        },
        {
            "tool_name": "delay", 
            "arguments": {"delay_ms": 200},
            "id": "pause_1"
        },
        {
            "tool_name": "get_focused_window_tree",
            "id": "get_window"
        },
        {
            "tool_name": "delay",
            "arguments": {"delay_ms": 100},
            "id": "final_wait"
        }
    ]
    
    print(f"\n🎬 Executing workflow with {len(workflow_steps)} steps:")
    for i, step in enumerate(workflow_steps, 1):
        print(f"   {i}. {step['tool_name']} (id: {step['id']})")
    
    print("\n" + "="*60)
    print("TELEMETRY OUTPUT FROM VECTOR:")
    print("="*60)
    
    start_time = time.time()
    
    try:
        # Execute the workflow - telemetry will be sent automatically
        result = await session.call_tool(
            "execute_sequence",
            arguments={"steps": workflow_steps}
        )
        
        duration = (time.time() - start_time) * 1000
        print("\n" + "="*60)
        print(f"✅ Workflow completed in {duration:.0f}ms")
        
        # Check result
        if result and hasattr(result, 'content'):
            for item in result.content:
                if hasattr(item, 'text'):
                    result_data = item.text
                    if "had_errors" in result_data:
                        import json
                        data = json.loads(result_data) if isinstance(result_data, str) else result_data
                        if data.get("had_errors"):
                            print("⚠️  Workflow had errors")
                        else:
                            print("✅ All steps succeeded")
                    break
                    
    except Exception as e:
        print(f"\n❌ Error executing workflow: {e}")
    
    await exit_stack.aclose()


async def main():
    print("="*70)
    print("MCP WORKFLOW TELEMETRY WITH VECTOR")
    print("="*70)
    
    # Check if Vector is installed
    vector_check = subprocess.run(
        ["vector", "--version"],
        capture_output=True,
        text=True
    )
    
    if vector_check.returncode != 0:
        print("\n❌ Vector is not installed or not in PATH")
        print("\nTo install Vector:")
        print("  Windows: scoop install vector")
        print("  macOS:   brew install vectordotdev/brew/vector")
        print("  Linux:   curl --proto '=https' --tlsv1.2 -sSf https://sh.vector.dev | bash")
        print("\nSee: https://vector.dev/docs/setup/installation/")
        return
    
    print(f"✅ Vector version: {vector_check.stdout.strip()}")
    
    # Start Vector
    vector_process = start_vector()
    if not vector_process:
        return
    
    try:
        # Run workflow
        await run_workflow_with_telemetry()
        
        # Give Vector time to process final traces
        print("\n⏳ Waiting for Vector to process traces...")
        await asyncio.sleep(3)
        
        print("\n" + "="*70)
        print("SUMMARY")
        print("="*70)
        print("✅ Telemetry was collected by Vector")
        print("📄 Check telemetry-output.json for full trace data")
        print("📊 Vector processed and displayed spans in real-time")
        
    finally:
        # Stop Vector
        if vector_process:
            print("\n🛑 Stopping Vector...")
            vector_process.terminate()
            vector_process.wait(timeout=5)
            print("✅ Vector stopped")


if __name__ == "__main__":
    asyncio.run(main())