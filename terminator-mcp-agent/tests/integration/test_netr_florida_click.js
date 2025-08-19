#!/usr/bin/env node
/**
 * NETR Online Florida Click Test
 * 
 * Tests clicking on the Florida hyperlink within NETR Online page in Chrome
 * using application-scoped search for better precision.
 * 
 * Usage:
 *   node test_netr_florida_click.js
 */

import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StreamableHTTPClientTransport } from '@modelcontextprotocol/sdk/client/streamableHttp.js';
import { spawn } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import { setTimeout } from 'timers/promises';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

class NetrFloridaClickTest {
  constructor() {
    this.client = null;
    this.transport = null;
    this.serverProcess = null;
  }

  async startMcpServer(port = 3001) {
    console.log(`🚀 Starting MCP server on port ${port}...`);
    
    // Find the MCP binary
    const possiblePaths = [
      path.join(__dirname, '../../../target/release/terminator-mcp-agent.exe'),  // Correct path from integration folder
      path.join(__dirname, '../../target/release/terminator-mcp-agent.exe'),
      path.join(__dirname, '../target/release/terminator-mcp-agent.exe'),
      'C:/Users/screenpipe-windows/terminator/target/release/terminator-mcp-agent.exe',  // Absolute path fallback
      'target/release/terminator-mcp-agent.exe',
      'target/release/terminator-mcp-agent',
    ];
    
    let binaryPath = null;
    for (const p of possiblePaths) {
      if (fs.existsSync(p)) {
        binaryPath = p;
        break;
      }
    }
    
    if (!binaryPath) {
      throw new Error('❌ MCP binary not found. Build with: cargo build --release --bin terminator-mcp-agent');
    }
    
    console.log(`📁 Using binary: ${binaryPath}`);
    
    // Start the server process
    this.serverProcess = spawn(binaryPath, [
      '--transport', 'http',
      '--port', port.toString()
    ], {
      stdio: ['ignore', 'pipe', 'pipe'],
      env: {
        ...process.env,
        RUST_LOG: 'debug',  // Set to debug to see the search details
        RUST_BACKTRACE: '1'
      }
    });
    
    // Log server output for debugging
    this.serverProcess.stdout?.on('data', (data) => {
      const output = data.toString().trim();
      if (output.includes('DEBUG') || output.includes('searching elements')) {
        console.log(`[SERVER DEBUG] ${output}`);
      } else {
        console.log(`[SERVER] ${output}`);
      }
    });
    
    this.serverProcess.stderr?.on('data', (data) => {
      console.error(`[SERVER ERROR] ${data.toString().trim()}`);
    });
    
    this.serverProcess.on('exit', (code) => {
      console.log(`[SERVER] Process exited with code ${code}`);
    });
    
    // Wait for server to start
    console.log('⏳ Waiting for server to initialize...');
    await setTimeout(3000);
    
    // Test server health
    try {
      const healthUrl = `http://127.0.0.1:${port}/health`;
      const response = await fetch(healthUrl, {
        method: 'GET',
        signal: AbortSignal.timeout(5000)
      });
      
      if (!response.ok) {
        throw new Error(`Health check failed: ${response.status}`);
      }
      console.log('✅ Server health check passed');
    } catch (error) {
      throw new Error(`Cannot reach MCP server: ${error}`);
    }
  }

  async connect(port = 3001) {
    const httpUrl = `http://127.0.0.1:${port}/mcp`;
    console.log(`🔌 Connecting to MCP server at ${httpUrl}...`);
    
    try {
      this.transport = new StreamableHTTPClientTransport(new URL(httpUrl));
      this.client = new Client(
        {
          name: "netr-florida-click-test",
          version: "1.0.0",
        },
        {
          capabilities: {
            tools: {},
          },
        }
      );
      
      await this.client.connect(this.transport);
      await setTimeout(500);
      
      console.log('✅ Connected to MCP server');
    } catch (error) {
      console.error('❌ Failed to connect:', error);
      throw error;
    }
  }

  async callTool(name, arguments_) {
    if (!this.client) {
      throw new Error('MCP client not connected');
    }
    
    console.log(`🛠️  Calling tool: ${name}`);
    if (arguments_ && Object.keys(arguments_).length > 0) {
      console.log(`   Arguments:`, JSON.stringify(arguments_, null, 2));
    }
    
    // Start timing the operation
    const startTime = performance.now();
    
    try {
      const result = await this.client.callTool({
        name,
        arguments: arguments_ || {},
      });
      
      // Calculate duration
      const endTime = performance.now();
      const duration = (endTime - startTime).toFixed(2);
      
      console.log(`✅ Tool ${name} completed successfully`);
      console.log(`⏱️  Execution time: ${duration}ms`);
      
      return { content: result.content, duration };
    } catch (error) {
      // Calculate duration even for failures
      const endTime = performance.now();
      const duration = (endTime - startTime).toFixed(2);
      
      console.error(`❌ Tool ${name} failed after ${duration}ms:`, error);
      throw error;
    }
  }

  async testNetrFloridaClick() {
    console.log('\n' + '='.repeat(60));
    console.log('🌴 NETR ONLINE FLORIDA CLICK TEST WITH PERFORMANCE METRICS');
    console.log('='.repeat(60));
    console.log('');
    console.log('This test will click on the Florida hyperlink in NETR Online');
    console.log('using Chrome application-scoped search for precision.');
    console.log('');
    console.log('Prerequisites:');
    console.log('• Chrome browser is open');
    console.log('• NETR Online page is loaded');
    console.log('• Florida link is visible on the page');
    console.log('');
    
    // Track timing for each test
    const timings = [];
    
    try {
      // Test 1: Try with Chrome application scope (most precise)
      console.log('🎯 Test 1: Clicking Florida with Chrome application scope...');
      console.log('   Selector: role:Application|name:contains:Chrome >> role:Pane|name:contains:NETR Online >> role:hyperlink|name:Florida');
      
      try {
        const chromeClickResult = await this.callTool('click_element', {
          selector: 'role:Application|name:contains:Chrome >> role:Pane|name:contains:NETR Online >> role:hyperlink|name:Florida',
          highlight_before_action: {
            enabled: true,
            duration_ms: 1000,
            color: 0x00FF00,  // Green highlight
            text: 'CLICKING FLORIDA',
            text_position: 'Top'
          }
        });
        
        timings.push({
          test: 'Chrome App → NETR Pane → Florida',
          duration: chromeClickResult.duration,
          success: true
        });
        
        if (chromeClickResult.content && chromeClickResult.content.length > 0) {
          const result = JSON.parse(chromeClickResult.content[0].text);
          console.log(`✅ Successfully clicked Florida link!`);
          console.log(`   Element: ${result.element.role} "${result.element.name}"`);
          console.log(`   Search scope: Chrome Application → NETR Pane → Florida link`);
        }
      } catch (error) {
        // Extract duration from error message if available
        const durationMatch = error.message?.match(/failed after (\d+\.?\d*)ms/);
        if (durationMatch) {
          timings.push({
            test: 'Chrome App → NETR Pane → Florida',
            duration: durationMatch[1],
            success: false
          });
        }
        console.log(`⚠️  Chrome-scoped search failed, trying alternative...`);
        
        // Test 2: Try with Window scope (fallback)
        console.log('\n🎯 Test 2: Trying with Window scope...');
        console.log('   Selector: role:Window|name:contains:NETR Online >> role:hyperlink|name:Florida');
        
        try {
          const windowClickResult = await this.callTool('click_element', {
            selector: 'role:Window|name:contains:NETR Online >> role:hyperlink|name:Florida',
            highlight_before_action: {
              enabled: true,
              duration_ms: 1000,
              color: 0x00FFFF,  // Yellow highlight
              text: 'CLICKING FLORIDA',
              text_position: 'Top'
            }
          });
          
          timings.push({
            test: 'NETR Window → Florida',
            duration: windowClickResult.duration,
            success: true
          });
          
          if (windowClickResult.content && windowClickResult.content.length > 0) {
            const result = JSON.parse(windowClickResult.content[0].text);
            console.log(`✅ Successfully clicked Florida link!`);
            console.log(`   Element: ${result.element.role} "${result.element.name}"`);
            console.log(`   Search scope: NETR Window → Florida link`);
          }
        } catch (error) {
          // Extract duration from error message if available
          const durationMatch = error.message?.match(/failed after (\d+\.?\d*)ms/);
          if (durationMatch) {
            timings.push({
              test: 'NETR Window → Florida',
              duration: durationMatch[1],
              success: false
            });
          }
          console.log(`⚠️  Window-scoped search failed, trying simplest approach...`);
          
          // Test 3: Try direct hyperlink search within Chrome
          console.log('\n🎯 Test 3: Direct hyperlink search in Chrome...');
          console.log('   Selector: role:Application|name:contains:Chrome >> role:hyperlink|name:Florida');
          
          const directClickResult = await this.callTool('click_element', {
            selector: 'role:Application|name:contains:Chrome >> role:hyperlink|name:Florida',
            highlight_before_action: {
              enabled: true,
              duration_ms: 1000,
              color: 0xFF00FF,  // Magenta highlight
              text: 'CLICKING FLORIDA',
              text_position: 'Top'
            }
          });
          
          timings.push({
            test: 'Chrome App → Florida (direct)',
            duration: directClickResult.duration,
            success: true
          });
          
          if (directClickResult.content && directClickResult.content.length > 0) {
            const result = JSON.parse(directClickResult.content[0].text);
            console.log(`✅ Successfully clicked Florida link!`);
            console.log(`   Element: ${result.element.role} "${result.element.name}"`);
            console.log(`   Search scope: Chrome Application → Florida link (direct)`);
          }
        }
      }
      
      // Wait to see the result
      await setTimeout(2000);
      
      // Summary
      console.log('\n' + '='.repeat(60));
      console.log('📊 TEST SUMMARY & PERFORMANCE METRICS');
      console.log('='.repeat(60));
      console.log('');
      console.log('The test attempted to click the Florida link using:');
      console.log('1. Chrome App → NETR Pane → Florida (most precise)');
      console.log('2. NETR Window → Florida (fallback)');
      console.log('3. Chrome App → Florida (simplest)');
      console.log('');
      
      // Performance metrics
      console.log('⏱️  PERFORMANCE METRICS:');
      console.log('─'.repeat(50));
      
      if (timings.length > 0) {
        // Find the fastest and slowest operations
        const sortedTimings = [...timings].sort((a, b) => parseFloat(a.duration) - parseFloat(b.duration));
        const fastest = sortedTimings[0];
        const slowest = sortedTimings[sortedTimings.length - 1];
        
        timings.forEach((timing, index) => {
          const status = timing.success ? '✅' : '❌';
          const timeMs = parseFloat(timing.duration);
          const timeStr = timeMs < 1000 ? `${timeMs}ms` : `${(timeMs/1000).toFixed(2)}s`;
          
          console.log(`${index + 1}. ${status} ${timing.test}: ${timeStr}`);
        });
      
        console.log('');
        console.log('📈 ANALYSIS:');
        console.log(`⚡ Fastest: ${fastest.test} (${fastest.duration}ms)`);
        console.log(`🐌 Slowest: ${slowest.test} (${slowest.duration}ms)`);
        
        // Calculate average
        const successfulTimings = timings.filter(t => t.success);
        if (successfulTimings.length > 0) {
          const avgTime = successfulTimings.reduce((sum, t) => sum + parseFloat(t.duration), 0) / successfulTimings.length;
          console.log(`📊 Average successful search time: ${avgTime.toFixed(2)}ms`);
        }
        
        const failedTimings = timings.filter(t => !t.success);
        if (failedTimings.length > 0) {
          const avgFailTime = failedTimings.reduce((sum, t) => sum + parseFloat(t.duration), 0) / failedTimings.length;
          console.log(`⏳ Average failed search time: ${avgFailTime.toFixed(2)}ms`);
        }
      }
      
      console.log('');
      console.log('💡 Tips for reliable clicking:');
      console.log('• Use application scope to avoid desktop-wide searches');
      console.log('• Include intermediate containers (Pane/Window) for precision');
      console.log('• Use "contains:" for partial name matching');
      console.log('• Enable highlighting to visually confirm the target');
      console.log('');
      console.log('🚀 OPTIMIZATION NOTES:');
      console.log('• Chain selectors now use find_element recursively (early exit)');
      console.log('• Each step finds FIRST match and stops immediately');
      console.log('• No more searching ALL elements at depth 50 across desktop');
      
    } catch (error) {
      console.error('❌ NETR Florida click test failed:', error);
      throw error;
    }
  }

  async cleanup() {
    console.log('\n🧹 Cleaning up...');
    
    try {
      if (this.client) {
        await this.client.close();
        this.client = null;
      }
      
      if (this.transport) {
        await this.transport.close();
        this.transport = null;
      }
      
      if (this.serverProcess) {
        console.log('🛑 Stopping MCP server...');
        this.serverProcess.kill('SIGTERM');
        
        await new Promise((resolve) => {
          const timeoutId = globalThis.setTimeout(() => {
            console.log('⚠️  Force killing server process...');
            this.serverProcess?.kill('SIGKILL');
            resolve();
          }, 5000);
          
          this.serverProcess?.on('exit', () => {
            globalThis.clearTimeout(timeoutId);
            resolve();
          });
        });
        
        this.serverProcess = null;
      }
      
      console.log('✅ Cleanup completed');
      
    } catch (error) {
      console.error('⚠️  Error during cleanup:', error);
    }
  }
}

async function main() {
  console.log('🧪 MCP NETR Online Florida Click Test');
  console.log('Testing application-scoped element clicking\n');
  
  const client = new NetrFloridaClickTest();
  
  try {
    // Start the MCP server
    await client.startMcpServer(3001);
    
    // Connect to the server
    await client.connect(3001);
    
    // Run the test
    await client.testNetrFloridaClick();
    
    console.log('\n🎉 Test completed successfully!');
    
  } catch (error) {
    console.error('\n💥 Test failed:', error);
    process.exit(1);
  } finally {
    await client.cleanup();
  }
}

// Handle process signals for cleanup
process.on('SIGINT', async () => {
  console.log('\n⚠️  Received SIGINT, cleaning up...');
  process.exit(0);
});

process.on('SIGTERM', async () => {
  console.log('\n⚠️  Received SIGTERM, cleaning up...');
  process.exit(0);
});

// Run the test
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main().catch(error => {
    console.error('💥 Unhandled error:', error);
    process.exit(1);
  });
}

export { NetrFloridaClickTest };

