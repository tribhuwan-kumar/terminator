#!/usr/bin/env python3
"""
Continuous Bridge Health Monitor with Self-Healing
Monitors WebSocket bridge and MCP server, attempts recovery on failures
"""

import asyncio
import json
import time
import sys
import subprocess
from datetime import datetime
import websockets
import aiohttp
from collections import deque

class BridgeHealthMonitor:
    def __init__(self):
        self.ws_url = "ws://127.0.0.1:17373"
        self.mcp_health_url = "http://127.0.0.1:8080/health"
        self.mcp_status_url = "http://127.0.0.1:8080/status"
        
        # Health tracking
        self.ws_connected = False
        self.ws_last_pong = None
        self.mcp_healthy = False
        self.consecutive_failures = 0
        self.connection_drops = 0
        self.total_pings = 0
        self.successful_pings = 0
        
        # History for pattern detection
        self.error_history = deque(maxlen=10)
        self.reconnect_history = deque(maxlen=10)
        
        # Self-healing thresholds
        self.MAX_CONSECUTIVE_FAILURES = 3
        self.PING_INTERVAL = 5  # seconds
        self.HEALTH_CHECK_INTERVAL = 10  # seconds
        
    def log(self, level, message):
        """Structured logging with timestamps"""
        timestamp = datetime.now().strftime("%H:%M:%S")
        symbols = {"INFO": "ℹ️", "SUCCESS": "✅", "ERROR": "❌", "WARNING": "⚠️", "HEAL": "🔧"}
        print(f"[{timestamp}] {symbols.get(level, '•')} {message}")
        
    async def check_mcp_health(self):
        """Check MCP server health endpoints"""
        try:
            async with aiohttp.ClientSession() as session:
                # Check /health
                async with session.get(self.mcp_health_url) as resp:
                    if resp.status == 200:
                        data = await resp.json()
                        if data.get("status") == "ok":
                            self.mcp_healthy = True
                        else:
                            self.mcp_healthy = False
                            
                # Check /status for details
                async with session.get(self.mcp_status_url) as resp:
                    if resp.status == 200:
                        status = await resp.json()
                        return status
        except Exception as e:
            self.mcp_healthy = False
            self.log("ERROR", f"MCP health check failed: {e}")
            return None
            
    async def monitor_websocket(self):
        """Continuously monitor WebSocket connection"""
        while True:
            try:
                self.log("INFO", f"Connecting to WebSocket bridge...")
                async with websockets.connect(self.ws_url) as ws:
                    self.ws_connected = True
                    self.consecutive_failures = 0
                    self.log("SUCCESS", "WebSocket connected")
                    
                    # Send hello
                    await ws.send(json.dumps({"type": "hello", "from": "monitor"}))
                    
                    # Ping loop
                    while True:
                        try:
                            # Send ping
                            await ws.send(json.dumps({"action": "ping"}))
                            self.total_pings += 1
                            
                            # Wait for pong with timeout
                            try:
                                response = await asyncio.wait_for(ws.recv(), timeout=2.0)
                                msg = json.loads(response)
                                if msg.get("type") == "pong":
                                    self.ws_last_pong = time.time()
                                    self.successful_pings += 1
                            except asyncio.TimeoutError:
                                self.log("WARNING", "Ping timeout - no pong received")
                                self.consecutive_failures += 1
                                
                            await asyncio.sleep(self.PING_INTERVAL)
                            
                        except websockets.ConnectionClosed:
                            break
                            
            except Exception as e:
                self.ws_connected = False
                self.connection_drops += 1
                self.consecutive_failures += 1
                self.error_history.append((datetime.now(), str(e)))
                self.log("ERROR", f"WebSocket error: {e}")
                
                # Self-healing decision
                if self.consecutive_failures >= self.MAX_CONSECUTIVE_FAILURES:
                    await self.attempt_recovery()
                    
                # Exponential backoff
                wait_time = min(2 ** self.consecutive_failures, 30)
                self.log("INFO", f"Reconnecting in {wait_time}s...")
                await asyncio.sleep(wait_time)
                
    async def attempt_recovery(self):
        """Attempt to recover the bridge connection"""
        self.log("HEAL", "Initiating self-healing sequence...")
        self.reconnect_history.append(datetime.now())
        
        # Step 1: Check if extension needs restart
        if len(self.reconnect_history) >= 3:
            recent_reconnects = [r for r in self.reconnect_history 
                               if (datetime.now() - r).seconds < 300]
            if len(recent_reconnects) >= 3:
                self.log("HEAL", "Frequent reconnects detected - extension may need restart")
                self.log("WARNING", "Manual intervention recommended: Reload Chrome extension")
                
        # Step 2: Check if port is still bound
        try:
            result = subprocess.run(
                ["netstat", "-an"], 
                capture_output=True, 
                text=True, 
                timeout=5
            )
            if "17373" in result.stdout and "LISTENING" in result.stdout:
                self.log("SUCCESS", "Port 17373 is still listening")
            else:
                self.log("ERROR", "Port 17373 not listening - bridge server may be down")
                # Could attempt to restart bridge here if we had permissions
        except Exception as e:
            self.log("ERROR", f"Failed to check port status: {e}")
            
        # Step 3: Reset failure counter after recovery attempt
        self.consecutive_failures = 0
        
    async def display_dashboard(self):
        """Display health dashboard every 10 seconds"""
        while True:
            await asyncio.sleep(self.HEALTH_CHECK_INTERVAL)
            
            # Check MCP health
            mcp_status = await self.check_mcp_health()
            
            # Calculate metrics
            ping_success_rate = (self.successful_pings / max(self.total_pings, 1)) * 100
            uptime = "Connected" if self.ws_connected else "Disconnected"
            
            # Display dashboard
            print("\n" + "="*60)
            print("📊 BRIDGE HEALTH DASHBOARD")
            print("="*60)
            print(f"WebSocket Bridge:  {('✅ ' + uptime) if self.ws_connected else '❌ Disconnected'}")
            print(f"MCP Server:        {'✅ Healthy' if self.mcp_healthy else '❌ Unhealthy'}")
            print(f"Ping Success Rate: {ping_success_rate:.1f}% ({self.successful_pings}/{self.total_pings})")
            print(f"Connection Drops:  {self.connection_drops}")
            print(f"Consecutive Fails: {self.consecutive_failures}")
            
            if mcp_status:
                print(f"MCP Active Reqs:   {mcp_status.get('activeRequests', 0)}")
                print(f"MCP Busy:          {'Yes' if mcp_status.get('busy') else 'No'}")
                
            if self.ws_last_pong:
                last_pong_ago = int(time.time() - self.ws_last_pong)
                print(f"Last Pong:         {last_pong_ago}s ago")
                
            # Show recent errors
            if self.error_history:
                print("\nRecent Errors:")
                for timestamp, error in list(self.error_history)[-3:]:
                    print(f"  [{timestamp.strftime('%H:%M:%S')}] {error[:50]}")
                    
            print("="*60)
            
            # Health score
            health_score = 0
            if self.ws_connected: health_score += 40
            if self.mcp_healthy: health_score += 30
            if ping_success_rate > 90: health_score += 20
            if self.consecutive_failures == 0: health_score += 10
            
            if health_score >= 90:
                self.log("SUCCESS", f"System Health: EXCELLENT ({health_score}%)")
            elif health_score >= 70:
                self.log("INFO", f"System Health: GOOD ({health_score}%)")
            elif health_score >= 50:
                self.log("WARNING", f"System Health: DEGRADED ({health_score}%)")
            else:
                self.log("ERROR", f"System Health: CRITICAL ({health_score}%)")
                
    async def run(self):
        """Run all monitoring tasks concurrently"""
        self.log("INFO", "Starting Bridge Health Monitor v1.0")
        self.log("INFO", f"Monitoring WebSocket: {self.ws_url}")
        self.log("INFO", f"Monitoring MCP: {self.mcp_health_url}")
        print("="*60)
        
        # Run monitoring tasks
        tasks = [
            asyncio.create_task(self.monitor_websocket()),
            asyncio.create_task(self.display_dashboard())
        ]
        
        try:
            await asyncio.gather(*tasks)
        except KeyboardInterrupt:
            self.log("INFO", "Shutting down monitor...")
            for task in tasks:
                task.cancel()

async def main():
    monitor = BridgeHealthMonitor()
    await monitor.run()

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n👋 Monitor stopped")
        sys.exit(0)