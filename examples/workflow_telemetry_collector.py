#!/usr/bin/env python3
"""
Workflow Execution with Built-in OTLP Collector

This example:
1. Starts a simple OTLP HTTP collector to receive traces from MCP server  
2. Executes workflows through MCP server (which sends telemetry automatically)
3. Displays the collected telemetry data in the console

The MCP server must be built with telemetry feature:
cargo build --release --features telemetry

pip install mcp flask
"""

import asyncio
import json
import time
from datetime import datetime
from typing import List, Dict, Any, Optional
from contextlib import AsyncExitStack
from threading import Thread
import base64

from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

from flask import Flask, request, jsonify


class TelemetryCollector:
    """Simple OTLP HTTP collector for receiving traces"""
    
    def __init__(self, port: int = 4318):
        self.port = port
        self.app = Flask(__name__)
        self.traces = []
        self.spans = []
        self.server_thread = None
        
        # Configure Flask logging
        import logging
        log = logging.getLogger('werkzeug')
        log.setLevel(logging.ERROR)
        
        # OTLP HTTP endpoint
        @self.app.route('/v1/traces', methods=['POST'])
        def receive_traces():
            data = request.get_json()
            self.handle_traces(data)
            return jsonify({"partialSuccess": {}}), 200
        
        # Health check
        @self.app.route('/health', methods=['GET'])
        def health():
            return jsonify({
                "status": "ok",
                "traces": len(self.traces),
                "spans": len(self.spans)
            })
    
    def handle_traces(self, data: Dict[str, Any]):
        """Process received OTLP trace data"""
        print("\n📡 Received telemetry data from MCP server")
        
        if "resourceSpans" in data:
            for resource_span in data["resourceSpans"]:
                # Extract service info
                service = self.extract_service_info(resource_span.get("resource", {}))
                
                # Process each span
                scope_spans = resource_span.get("scopeSpans", [])
                for scope_span in scope_spans:
                    for span in scope_span.get("spans", []):
                        self.process_span(span, service)
        
        self.traces.append(data)
    
    def extract_service_info(self, resource: Dict[str, Any]) -> Dict[str, str]:
        """Extract service information from resource attributes"""
        info = {}
        for attr in resource.get("attributes", []):
            if attr["key"] == "service.name":
                info["name"] = attr["value"].get("stringValue", "")
            elif attr["key"] == "service.version":
                info["version"] = attr["value"].get("stringValue", "")
        return info
    
    def process_span(self, span: Dict[str, Any], service: Dict[str, str]):
        """Process individual span and display it"""
        # Calculate duration if both times exist
        duration = 0
        if span.get("endTimeUnixNano") and span.get("startTimeUnixNano"):
            start = int(span["startTimeUnixNano"])
            end = int(span["endTimeUnixNano"])
            duration = (end - start) / 1_000_000  # Convert to milliseconds
        
        span_info = {
            "name": span.get("name", ""),
            "traceId": self.decode_trace_id(span.get("traceId", "")),
            "spanId": self.decode_span_id(span.get("spanId", "")),
            "parentSpanId": self.decode_span_id(span.get("parentSpanId", "")),
            "duration": duration,
            "attributes": {},
            "events": [],
            "status": span.get("status", {})
        }
        
        # Extract attributes
        for attr in span.get("attributes", []):
            span_info["attributes"][attr["key"]] = self.extract_value(attr["value"])
        
        # Extract events
        for event in span.get("events", []):
            event_info = {
                "name": event.get("name", ""),
                "attributes": {}
            }
            for attr in event.get("attributes", []):
                event_info["attributes"][attr["key"]] = self.extract_value(attr["value"])
            span_info["events"].append(event_info)
        
        self.spans.append(span_info)
        self.display_span(span_info)
    
    def decode_trace_id(self, trace_id: str) -> str:
        """Decode base64 trace ID to hex"""
        if not trace_id:
            return ""
        try:
            return base64.b64decode(trace_id).hex()
        except:
            return trace_id
    
    def decode_span_id(self, span_id: str) -> str:
        """Decode base64 span ID to hex"""
        if not span_id:
            return ""
        try:
            return base64.b64decode(span_id).hex()
        except:
            return span_id
    
    def extract_value(self, value: Dict[str, Any]) -> Any:
        """Extract value from OTLP attribute value"""
        if "stringValue" in value:
            return value["stringValue"]
        elif "intValue" in value:
            return int(value["intValue"])
        elif "doubleValue" in value:
            return float(value["doubleValue"])
        elif "boolValue" in value:
            return value["boolValue"]
        return value
    
    def display_span(self, span: Dict[str, Any]):
        """Display span information in console"""
        indent = "  └─" if span["parentSpanId"] else "📊"
        duration = f"{span['duration']:.2f}ms" if span['duration'] else "N/A"
        
        print(f"{indent} {span['name']} [{duration}]")
        
        # Show important attributes
        attrs = span["attributes"]
        if "workflow.name" in attrs:
            print(f"     Workflow: {attrs['workflow.name']}")
        if "step.tool" in attrs:
            print(f"     Tool: {attrs['step.tool']}")
        if "step.number" in attrs:
            print(f"     Step: {attrs['step.number']}/{attrs.get('step.total', '?')}")
        
        # Show events
        for event in span["events"]:
            print(f"     📌 {event['name']}")
        
        # Show status
        status = span.get("status", {})
        if status.get("code") == 2:
            print(f"     ❌ Status: {status.get('message', 'Error')}")
        elif status.get("code") == 1:
            print(f"     ⚠️  Status: {status.get('message', 'Unknown')}")
    
    def start(self):
        """Start the collector in a background thread"""
        def run_server():
            self.app.run(host='0.0.0.0', port=self.port, debug=False, use_reloader=False)
        
        self.server_thread = Thread(target=run_server, daemon=True)
        self.server_thread.start()
        
        print(f"🎯 OTLP Collector listening on http://localhost:{self.port}")
        print(f"   Endpoint: http://localhost:{self.port}/v1/traces")
        
        # Wait for server to be ready
        time.sleep(1)
    
    def get_summary(self):
        """Display summary of collected telemetry"""
        print("\n" + "═" * 70)
        print("TELEMETRY SUMMARY")
        print("═" * 70)
        print(f"Total traces received: {len(self.traces)}")
        print(f"Total spans collected: {len(self.spans)}")
        
        # Group spans by type
        workflows = [s for s in self.spans if s["name"].startswith("workflow.")]
        steps = [s for s in self.spans if s["name"].startswith("step.")]
        
        print(f"Workflows executed: {len(workflows)}")
        print(f"Steps executed: {len(steps)}")
        
        # Calculate average durations
        if workflows:
            avg_workflow = sum(s["duration"] for s in workflows) / len(workflows)
            print(f"Average workflow duration: {avg_workflow:.2f}ms")
        
        if steps:
            avg_step = sum(s["duration"] for s in steps) / len(steps)
            print(f"Average step duration: {avg_step:.2f}ms")
        
        print("═" * 70)


class WorkflowExecutor:
    """Execute workflows through MCP server with telemetry"""
    
    def __init__(self, collector_port: int = 4318):
        self.session: Optional[ClientSession] = None
        self.exit_stack = AsyncExitStack()
        self.collector_port = collector_port
    
    async def connect_to_mcp(self):
        """Connect to MCP server with telemetry enabled"""
        print("\n🔌 Starting MCP server with telemetry enabled...")
        
        import os
        env = os.environ.copy()
        env["OTEL_EXPORTER_OTLP_ENDPOINT"] = f"http://localhost:{self.collector_port}"
        env["RUST_LOG"] = "info,terminator_mcp_agent=debug"
        
        server_params = StdioServerParameters(
            command="target/release/terminator-mcp-agent",
            args=[],
            env=env
        )
        
        transport = await self.exit_stack.enter_async_context(
            stdio_client(server_params)
        )
        
        self.session = await self.exit_stack.enter_async_context(
            ClientSession(transport[0], transport[1])
        )
        
        await self.session.initialize()
        
        tools_result = await self.session.list_tools()
        print(f"✅ MCP connected! {len(tools_result.tools)} tools available")
    
    async def execute_workflow(self, name: str, steps: List[Dict[str, Any]]):
        """Execute a workflow"""
        print(f"\n🚀 Executing workflow: {name}")
        print(f"📋 {len(steps)} steps to execute")
        
        start_time = time.time()
        
        # Try to use execute_sequence if available
        try:
            result = await self.session.call_tool(
                "execute_sequence",
                arguments={"steps": steps}
            )
            
            duration = (time.time() - start_time) * 1000
            print(f"✅ Workflow completed in {duration:.2f}ms")
            return result
            
        except Exception as e:
            # Fallback to executing steps individually
            print("ℹ️  execute_sequence not available, running steps individually")
            
            for i, step in enumerate(steps):
                print(f"  Step {i+1}/{len(steps)}: {step['tool_name']}")
                
                try:
                    await self.session.call_tool(
                        step["tool_name"],
                        arguments=step.get("arguments", {})
                    )
                except Exception as step_error:
                    print(f"  ❌ Step failed: {step_error}")
                    if not step.get("continue_on_error", False):
                        raise
            
            duration = (time.time() - start_time) * 1000
            print(f"✅ Workflow completed in {duration:.2f}ms")
    
    async def cleanup(self):
        """Clean up resources"""
        await self.exit_stack.aclose()


# Example workflows
WORKFLOWS = {
    "simple": {
        "name": "Simple Screenshot",
        "steps": [
            {"tool_name": "screenshot", "id": "capture"}
        ]
    },
    
    "notepad": {
        "name": "Notepad Automation",
        "steps": [
            {"tool_name": "screenshot", "id": "initial_screenshot"},
            {"tool_name": "launch_application", "arguments": {"app_name": "notepad"}, "id": "launch"},
            {"tool_name": "wait", "arguments": {"delay_ms": 2000}, "id": "wait_launch"},
            {"tool_name": "type_text", "arguments": {"text": "Hello from telemetry workflow!"}, "id": "type"},
            {"tool_name": "screenshot", "id": "final_screenshot"}
        ]
    },
    
    "desktop": {
        "name": "Desktop Interaction",
        "steps": [
            {"tool_name": "get_desktop_elements", "id": "get_elements"},
            {"tool_name": "move_mouse_to", "arguments": {"x": 100, "y": 100}, "id": "move1"},
            {"tool_name": "wait", "arguments": {"delay_ms": 500}, "id": "wait1"},
            {"tool_name": "move_mouse_to", "arguments": {"x": 500, "y": 300}, "id": "move2"},
            {"tool_name": "screenshot", "id": "capture"}
        ]
    }
}


async def main():
    """Main execution"""
    collector = TelemetryCollector()
    executor = WorkflowExecutor()
    
    try:
        # Start OTLP collector
        collector.start()
        
        # Connect to MCP server
        await executor.connect_to_mcp()
        
        # Execute workflows
        await executor.execute_workflow(WORKFLOWS["simple"]["name"], WORKFLOWS["simple"]["steps"])
        await executor.execute_workflow(WORKFLOWS["notepad"]["name"], WORKFLOWS["notepad"]["steps"])
        await executor.execute_workflow(WORKFLOWS["desktop"]["name"], WORKFLOWS["desktop"]["steps"])
        
        # Wait for telemetry to arrive
        print("\n⏳ Waiting for telemetry data...")
        await asyncio.sleep(3)
        
        # Show summary
        collector.get_summary()
        
    except KeyboardInterrupt:
        print("\n⚠️  Interrupted")
    except Exception as error:
        print(f"❌ Error: {error}")
        import traceback
        traceback.print_exc()
    finally:
        await executor.cleanup()


if __name__ == "__main__":
    print("═" * 70)
    print("WORKFLOW EXECUTION WITH TELEMETRY COLLECTION")
    print("═" * 70)
    print("\nMake sure MCP server is built with telemetry:")
    print("  cd terminator-mcp-agent")
    print("  cargo build --release --features telemetry")
    print("═" * 70)
    
    asyncio.run(main())