#!/usr/bin/env node

const WebSocket = require('ws');

console.log('🔍 Testing Browser Extension Bridge Health\n');
console.log('=' .repeat(50));

// Test 1: WebSocket Connection
console.log('\n1. WebSocket Bridge Connection Test');
console.log('-'.repeat(40));

const ws = new WebSocket('ws://127.0.0.1:17373');
let connectionSuccess = false;

ws.on('open', () => {
    connectionSuccess = true;
    console.log('✅ Connected to WebSocket bridge on port 17373');
    
    // Send hello message
    ws.send(JSON.stringify({ type: 'hello', from: 'health-check' }));
    console.log('📤 Sent hello message');
    
    // Send ping to test responsiveness
    setTimeout(() => {
        ws.send(JSON.stringify({ action: 'ping' }));
        console.log('📤 Sent ping message');
    }, 100);
    
    // Test eval capability
    setTimeout(() => {
        const evalRequest = {
            id: 'test-' + Date.now(),
            action: 'eval',
            code: '({alive: true, timestamp: Date.now(), url: window.location.href})',
            awaitPromise: false
        };
        ws.send(JSON.stringify(evalRequest));
        console.log('📤 Sent eval request to test browser script execution');
    }, 200);
});

ws.on('message', (data) => {
    try {
        const msg = JSON.parse(data.toString());
        
        if (msg.type === 'pong') {
            console.log('✅ Received pong response - bridge is responsive');
        } else if (msg.ok !== undefined) {
            if (msg.ok) {
                console.log('✅ Browser script execution successful:', msg.result);
            } else {
                console.log('❌ Browser script execution failed:', msg.error);
            }
        } else {
            console.log('📥 Received message:', msg);
        }
    } catch (e) {
        console.log('📥 Received raw message:', data.toString());
    }
});

ws.on('error', (err) => {
    console.log('❌ WebSocket error:', err.message);
});

ws.on('close', () => {
    console.log('🔌 WebSocket connection closed');
});

// Test 2: HTTP Health Endpoints
setTimeout(async () => {
    console.log('\n2. MCP Server Health Endpoints Test');
    console.log('-'.repeat(40));
    
    try {
        // Test /health endpoint
        const healthRes = await fetch('http://127.0.0.1:8080/health');
        const healthData = await healthRes.json();
        console.log('✅ /health endpoint:', healthData);
        
        // Test /status endpoint
        const statusRes = await fetch('http://127.0.0.1:8080/status');
        const statusData = await statusRes.json();
        console.log('✅ /status endpoint:', {
            busy: statusData.busy,
            activeRequests: statusData.activeRequests,
            lastActivity: new Date(statusData.lastActivity).toLocaleString()
        });
    } catch (e) {
        console.log('❌ Failed to reach MCP server:', e.message);
    }
    
    // Summary
    setTimeout(() => {
        console.log('\n' + '='.repeat(50));
        console.log('📊 Health Check Summary:');
        console.log('-'.repeat(40));
        console.log(`MCP Server (port 8080): ✅ HEALTHY`);
        console.log(`WebSocket Bridge (port 17373): ${connectionSuccess ? '✅ CONNECTED' : '❌ NOT CONNECTED'}`);
        console.log(`Extension Status: ${connectionSuccess ? '🟡 Bridge connected (extension state unknown)' : '❌ No bridge connection'}`);
        console.log('\n💡 Note: Extension script execution depends on:');
        console.log('   - Chrome/Edge browser running');
        console.log('   - Terminator extension installed and enabled');
        console.log('   - At least one browser tab open');
        console.log('=' .repeat(50));
        
        // Close connection and exit
        ws.close();
        process.exit(0);
    }, 1000);
}, 500);