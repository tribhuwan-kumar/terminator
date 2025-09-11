#!/usr/bin/env python3
"""
Simple Workflow Execution with Console Streaming

Executes workflows and streams step-by-step progress to console
No external dependencies except MCP client

pip install mcp
"""

import asyncio
import sys
import io

# Fix Windows console encoding for emojis
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')
import json
import time
from datetime import datetime
from typing import List, Dict, Any, Optional
from contextlib import AsyncExitStack

from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client


class WorkflowStreamer:
    """Simple workflow executor with console streaming"""
    
    def __init__(self):
        self.session: Optional[ClientSession] = None
        self.exit_stack = AsyncExitStack()
        self.start_time: Optional[float] = None
    
    async def connect(self, server_path: str = "terminator-mcp-agent/target/release/terminator-mcp-agent"):
        """Connect to MCP server"""
        print(f"\n🔌 Connecting to MCP server...")
        
        server_params = StdioServerParameters(
            command=server_path,
            args=[],
            env=None
        )
        
        transport = await self.exit_stack.enter_async_context(
            stdio_client(server_params)
        )
        
        self.session = await self.exit_stack.enter_async_context(
            ClientSession(transport[0], transport[1])
        )
        
        await self.session.initialize()
        
        tools_result = await self.session.list_tools()
        print(f"✅ Connected! {len(tools_result.tools)} tools available\n")
    
    def log(self, emoji: str, message: str, data: Any = None):
        """Log with timestamp and elapsed time"""
        timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        elapsed = f"+{int((time.time() - self.start_time) * 1000):4}ms" if self.start_time else "       "
        
        print(f"[{timestamp}] {elapsed} {emoji} {message}")
        if data is not None:
            print(f"{' ' * 35}└─ {json.dumps(data) if isinstance(data, dict) else data}")
    
    async def execute_workflow(self, name: str, steps: List[Dict[str, Any]]) -> List[Any]:
        """Execute workflow with streaming output"""
        print("═" * 70)
        print(f"WORKFLOW: {name}")
        print("═" * 70)
        
        self.start_time = time.time()
        results = []
        context = {}
        
        self.log("🚀", f"Starting workflow with {len(steps)} steps")
        
        for i, step in enumerate(steps):
            step_num = i + 1
            tool_name = step.get("tool_name") or step.get("tool")
            
            print("─" * 70)
            self.log("📍", f"Step {step_num}/{len(steps)}: {tool_name}")
            
            try:
                # Log arguments if present
                args = step.get("arguments", {})
                if args:
                    self.log("📝", "Arguments:", args)
                
                # Execute the tool
                self.log("⚙️", "Executing...")
                result = await self.session.call_tool(tool_name, arguments=args)
                
                # Store result if needed
                if "set_env" in step:
                    context[step["set_env"]] = self._extract_result(result)
                    self.log("💾", f"Saved to context.{step['set_env']}")
                
                # Log success
                self.log("✅", f"Step {step_num} completed")
                
                # Show partial result if text
                text = self._extract_text(result)
                if text:
                    preview = text[:100]
                    self.log("📄", f"Output: {preview}{'...' if len(text) > 100 else ''}")
                
                results.append(result)
                
            except Exception as error:
                self.log("❌", f"Step {step_num} failed: {error}")
                
                if not step.get("continue_on_error", False):
                    self.log("🛑", "Workflow aborted")
                    raise
                
                self.log("⚠️", "Continuing despite error...")
            
            # Small delay for readability
            if i < len(steps) - 1:
                await asyncio.sleep(0.1)
        
        duration = int((time.time() - self.start_time) * 1000)
        print("═" * 70)
        self.log("🎉", f"Workflow completed in {duration}ms")
        print("═" * 70)
        
        return results
    
    def _extract_result(self, result: Any) -> Any:
        """Extract data from MCP result"""
        if not result or not hasattr(result, 'content'):
            return None
        
        items = []
        for item in result.content:
            if hasattr(item, 'text'):
                items.append(item.text)
            elif hasattr(item, 'data'):
                items.append(item.data)
        
        return items[0] if len(items) == 1 else items
    
    def _extract_text(self, result: Any) -> Optional[str]:
        """Extract text from MCP result"""
        if not result or not hasattr(result, 'content'):
            return None
        
        for item in result.content:
            if hasattr(item, 'text'):
                return item.text
        return None
    
    async def cleanup(self):
        """Clean up resources"""
        await self.exit_stack.aclose()


# Example workflows
WORKFLOWS = {
    "notepad": {
        "name": "Notepad Hello World",
        "steps": [
            {
                "tool_name": "screenshot",
                "arguments": {}
            },
            {
                "tool_name": "launch_application",
                "arguments": {"app_name": "notepad"}
            },
            {
                "tool_name": "wait",
                "arguments": {"delay_ms": 2000}
            },
            {
                "tool_name": "type_text",
                "arguments": {"text": "Hello from MCP workflow!\nThis is automated typing.\n"}
            },
            {
                "tool_name": "wait",
                "arguments": {"delay_ms": 1000}
            },
            {
                "tool_name": "type_text",
                "arguments": {"text": f"Current time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}"}
            },
            {
                "tool_name": "screenshot",
                "arguments": {}
            }
        ]
    },
    
    "calculator": {
        "name": "Calculator Demo",
        "steps": [
            {
                "tool_name": "launch_application",
                "arguments": {"app_name": "calc"}
            },
            {
                "tool_name": "wait",
                "arguments": {"delay_ms": 2000}
            },
            {
                "tool_name": "click",
                "arguments": {"text": "7"}
            },
            {
                "tool_name": "click",
                "arguments": {"text": "+"}
            },
            {
                "tool_name": "click",
                "arguments": {"text": "3"}
            },
            {
                "tool_name": "click",
                "arguments": {"text": "="}
            },
            {
                "tool_name": "screenshot",
                "arguments": {}
            }
        ]
    },
    
    "desktop": {
        "name": "Desktop Interaction",
        "steps": [
            {
                "tool_name": "screenshot",
                "arguments": {}
            },
            {
                "tool_name": "get_desktop_elements",
                "arguments": {},
                "set_env": "elements"
            },
            {
                "tool_name": "move_mouse_to",
                "arguments": {"x": 100, "y": 100}
            },
            {
                "tool_name": "wait",
                "arguments": {"delay_ms": 500}
            },
            {
                "tool_name": "move_mouse_to",
                "arguments": {"x": 500, "y": 300}
            },
            {
                "tool_name": "screenshot",
                "arguments": {}
            }
        ]
    },
    
    "browser": {
        "name": "Browser Automation",
        "steps": [
            {
                "tool_name": "launch_application",
                "arguments": {"app_name": "chrome"}
            },
            {
                "tool_name": "wait",
                "arguments": {"delay_ms": 3000}
            },
            {
                "tool_name": "type_text",
                "arguments": {"text": "https://example.com"}
            },
            {
                "tool_name": "key",
                "arguments": {"key": "Return"}
            },
            {
                "tool_name": "wait",
                "arguments": {"delay_ms": 3000}
            },
            {
                "tool_name": "screenshot",
                "arguments": {}
            }
        ]
    }
}


async def interactive_mode(streamer: WorkflowStreamer):
    """Interactive mode to run custom workflows"""
    print("\n" + "=" * 70)
    print("INTERACTIVE WORKFLOW MODE")
    print("=" * 70)
    print("Enter steps one by one. Type 'run' to execute, 'exit' to quit")
    print("Format: tool_name [arg1=value1 arg2=value2]")
    print("Example: type_text text='Hello World'")
    print("=" * 70)
    
    steps = []
    
    while True:
        try:
            user_input = input(f"\nStep {len(steps) + 1}> ").strip()
            
            if user_input.lower() == 'exit':
                break
            elif user_input.lower() == 'run':
                if steps:
                    await streamer.execute_workflow("Custom Workflow", steps)
                    steps = []
                else:
                    print("No steps to run!")
            elif user_input:
                # Parse the input
                parts = user_input.split(maxsplit=1)
                tool_name = parts[0]
                
                # Parse arguments if present
                arguments = {}
                if len(parts) > 1:
                    arg_str = parts[1]
                    # Simple parsing (in production, use proper parser)
                    for arg in arg_str.split():
                        if '=' in arg:
                            key, value = arg.split('=', 1)
                            # Remove quotes if present
                            if value.startswith("'") and value.endswith("'"):
                                value = value[1:-1]
                            elif value.startswith('"') and value.endswith('"'):
                                value = value[1:-1]
                            # Try to convert to number
                            try:
                                value = int(value)
                            except ValueError:
                                try:
                                    value = float(value)
                                except ValueError:
                                    pass
                            arguments[key] = value
                
                step = {"tool_name": tool_name, "arguments": arguments}
                steps.append(step)
                print(f"Added: {step}")
                
        except KeyboardInterrupt:
            print("\n\nInterrupted!")
            break
        except Exception as e:
            print(f"Error: {e}")


async def main():
    """Main entry point"""
    import sys
    
    streamer = WorkflowStreamer()
    
    try:
        await streamer.connect()
        
        # Check command line arguments
        if len(sys.argv) > 1:
            workflow_name = sys.argv[1]
            
            if workflow_name == "interactive":
                await interactive_mode(streamer)
            elif workflow_name in WORKFLOWS:
                workflow = WORKFLOWS[workflow_name]
                await streamer.execute_workflow(workflow["name"], workflow["steps"])
            else:
                print(f"Unknown workflow: {workflow_name}")
                print(f"Available workflows: {', '.join(WORKFLOWS.keys())}")
                print("Or use 'interactive' for interactive mode")
        else:
            # Default to notepad workflow
            workflow = WORKFLOWS["notepad"]
            await streamer.execute_workflow(workflow["name"], workflow["steps"])
            
    except KeyboardInterrupt:
        print("\n\n⚠️  Interrupted")
    except Exception as error:
        print(f"\n❌ Error: {error}")
        import traceback
        traceback.print_exc()
    finally:
        await streamer.cleanup()


if __name__ == "__main__":
    # Run the async main function
    asyncio.run(main())