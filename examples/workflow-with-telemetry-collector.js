#!/usr/bin/env node
/**
 * Workflow Execution with Built-in OTLP Collector
 * 
 * This example:
 * 1. Starts a simple OTLP HTTP collector to receive traces from MCP server
 * 2. Executes workflows through MCP server (which sends telemetry automatically)
 * 3. Displays the collected telemetry data in the console
 * 
 * The MCP server must be built with telemetry feature:
 * cargo build --release --features telemetry
 * 
 * npm install mcp express body-parser
 */

const { ClientSession, StdioServerParameters } = require('mcp');
const { stdio_client } = require('mcp/client/stdio');
const express = require('express');
const bodyParser = require('body-parser');
const { spawn } = require('child_process');

class TelemetryCollector {
    constructor(port = 4318) {
        this.port = port;
        this.app = express();
        this.server = null;
        this.traces = [];
        this.spans = [];
        
        // Parse OTLP JSON
        this.app.use(bodyParser.json({ limit: '10mb' }));
        
        // OTLP HTTP endpoint
        this.app.post('/v1/traces', (req, res) => {
            this.handleTraces(req.body);
            res.status(200).json({ partialSuccess: {} });
        });
        
        // Health check
        this.app.get('/health', (req, res) => {
            res.json({ status: 'ok', traces: this.traces.length });
        });
    }
    
    handleTraces(data) {
        console.log('\n📡 Received telemetry data from MCP server');
        
        if (data.resourceSpans) {
            for (const resourceSpan of data.resourceSpans) {
                // Extract service info
                const service = this.extractServiceInfo(resourceSpan.resource);
                
                // Process each span
                if (resourceSpan.scopeSpans) {
                    for (const scopeSpan of resourceSpan.scopeSpans) {
                        if (scopeSpan.spans) {
                            for (const span of scopeSpan.spans) {
                                this.processSpan(span, service);
                            }
                        }
                    }
                }
            }
        }
        
        this.traces.push(data);
    }
    
    extractServiceInfo(resource) {
        const info = {};
        if (resource && resource.attributes) {
            for (const attr of resource.attributes) {
                if (attr.key === 'service.name') {
                    info.name = attr.value.stringValue;
                } else if (attr.key === 'service.version') {
                    info.version = attr.value.stringValue;
                }
            }
        }
        return info;
    }
    
    processSpan(span, service) {
        const spanInfo = {
            name: span.name,
            traceId: span.traceId,
            spanId: span.spanId,
            parentSpanId: span.parentSpanId,
            startTime: span.startTimeUnixNano,
            endTime: span.endTimeUnixNano,
            duration: span.endTimeUnixNano ? 
                (parseInt(span.endTimeUnixNano) - parseInt(span.startTimeUnixNano)) / 1000000 : 0,
            attributes: {},
            events: [],
            status: span.status
        };
        
        // Extract attributes
        if (span.attributes) {
            for (const attr of span.attributes) {
                spanInfo.attributes[attr.key] = this.extractValue(attr.value);
            }
        }
        
        // Extract events
        if (span.events) {
            for (const event of span.events) {
                const eventInfo = {
                    name: event.name,
                    time: event.timeUnixNano,
                    attributes: {}
                };
                
                if (event.attributes) {
                    for (const attr of event.attributes) {
                        eventInfo.attributes[attr.key] = this.extractValue(attr.value);
                    }
                }
                
                spanInfo.events.push(eventInfo);
            }
        }
        
        this.spans.push(spanInfo);
        this.displaySpan(spanInfo);
    }
    
    extractValue(value) {
        if (value.stringValue !== undefined) return value.stringValue;
        if (value.intValue !== undefined) return value.intValue;
        if (value.doubleValue !== undefined) return value.doubleValue;
        if (value.boolValue !== undefined) return value.boolValue;
        return value;
    }
    
    displaySpan(span) {
        const indent = span.parentSpanId ? '  └─' : '📊';
        const duration = span.duration ? `${span.duration.toFixed(2)}ms` : 'N/A';
        
        console.log(`${indent} ${span.name} [${duration}]`);
        
        // Show important attributes
        if (span.attributes['workflow.name']) {
            console.log(`     Workflow: ${span.attributes['workflow.name']}`);
        }
        if (span.attributes['step.tool']) {
            console.log(`     Tool: ${span.attributes['step.tool']}`);
        }
        if (span.attributes['step.number']) {
            console.log(`     Step: ${span.attributes['step.number']}/${span.attributes['step.total']}`);
        }
        
        // Show events
        for (const event of span.events) {
            console.log(`     📌 ${event.name}`);
        }
        
        // Show status
        if (span.status && span.status.code !== 0) {
            const statusIcon = span.status.code === 2 ? '❌' : '⚠️';
            console.log(`     ${statusIcon} Status: ${span.status.message || 'Error'}`);
        }
    }
    
    async start() {
        return new Promise((resolve) => {
            this.server = this.app.listen(this.port, () => {
                console.log(`🎯 OTLP Collector listening on http://localhost:${this.port}`);
                console.log(`   Endpoint: http://localhost:${this.port}/v1/traces`);
                resolve();
            });
        });
    }
    
    stop() {
        if (this.server) {
            this.server.close();
        }
    }
    
    getSummary() {
        console.log('\n' + '═'.repeat(70));
        console.log('TELEMETRY SUMMARY');
        console.log('═'.repeat(70));
        console.log(`Total traces received: ${this.traces.length}`);
        console.log(`Total spans collected: ${this.spans.length}`);
        
        // Group spans by type
        const workflows = this.spans.filter(s => s.name.startsWith('workflow.'));
        const steps = this.spans.filter(s => s.name.startsWith('step.'));
        
        console.log(`Workflows executed: ${workflows.length}`);
        console.log(`Steps executed: ${steps.length}`);
        
        // Calculate average durations
        if (workflows.length > 0) {
            const avgWorkflow = workflows.reduce((sum, s) => sum + s.duration, 0) / workflows.length;
            console.log(`Average workflow duration: ${avgWorkflow.toFixed(2)}ms`);
        }
        
        if (steps.length > 0) {
            const avgStep = steps.reduce((sum, s) => sum + s.duration, 0) / steps.length;
            console.log(`Average step duration: ${avgStep.toFixed(2)}ms`);
        }
        
        console.log('═'.repeat(70));
    }
}

class WorkflowExecutor {
    constructor(collectorPort = 4318) {
        this.session = null;
        this.collectorPort = collectorPort;
    }
    
    async connectToMCP() {
        console.log('\n🔌 Starting MCP server with telemetry enabled...');
        
        const serverParams = {
            command: 'target/release/terminator-mcp-agent',
            args: [],
            env: {
                ...process.env,
                OTEL_EXPORTER_OTLP_ENDPOINT: `http://localhost:${this.collectorPort}`,
                RUST_LOG: 'info,terminator_mcp_agent=debug'
            }
        };
        
        const transport = await stdio_client(serverParams);
        this.session = new ClientSession(transport[0], transport[1]);
        await this.session.initialize();
        
        const tools = await this.session.list_tools();
        console.log(`✅ MCP connected! ${tools.tools.length} tools available`);
    }
    
    async executeWorkflow(name, steps) {
        console.log(`\n🚀 Executing workflow: ${name}`);
        console.log(`📋 ${steps.length} steps to execute`);
        
        const startTime = Date.now();
        
        // Execute using execute_sequence tool if available
        try {
            const result = await this.session.call_tool('execute_sequence', {
                arguments: { steps }
            });
            
            const duration = Date.now() - startTime;
            console.log(`✅ Workflow completed in ${duration}ms`);
            
            return result;
        } catch (error) {
            // Fallback to executing steps individually
            console.log('ℹ️  execute_sequence not available, running steps individually');
            
            for (let i = 0; i < steps.length; i++) {
                const step = steps[i];
                console.log(`  Step ${i+1}/${steps.length}: ${step.tool_name}`);
                
                try {
                    await this.session.call_tool(
                        step.tool_name,
                        { arguments: step.arguments || {} }
                    );
                } catch (e) {
                    console.log(`  ❌ Step failed: ${e.message}`);
                    if (!step.continue_on_error) throw e;
                }
            }
            
            const duration = Date.now() - startTime;
            console.log(`✅ Workflow completed in ${duration}ms`);
        }
    }
    
    async cleanup() {
        if (this.session) {
            await this.session.close();
        }
    }
}

// Example workflows
const WORKFLOWS = {
    simple: {
        name: 'Simple Screenshot',
        steps: [
            { tool_name: 'screenshot', id: 'capture' }
        ]
    },
    
    notepad: {
        name: 'Notepad Automation',
        steps: [
            { tool_name: 'screenshot', id: 'initial_screenshot' },
            { tool_name: 'launch_application', arguments: { app_name: 'notepad' }, id: 'launch' },
            { tool_name: 'wait', arguments: { delay_ms: 2000 }, id: 'wait_launch' },
            { tool_name: 'type_text', arguments: { text: 'Hello from telemetry workflow!' }, id: 'type' },
            { tool_name: 'screenshot', id: 'final_screenshot' }
        ]
    },
    
    desktop: {
        name: 'Desktop Interaction',
        steps: [
            { tool_name: 'get_desktop_elements', id: 'get_elements' },
            { tool_name: 'move_mouse_to', arguments: { x: 100, y: 100 }, id: 'move1' },
            { tool_name: 'wait', arguments: { delay_ms: 500 }, id: 'wait1' },
            { tool_name: 'move_mouse_to', arguments: { x: 500, y: 300 }, id: 'move2' },
            { tool_name: 'screenshot', id: 'capture' }
        ]
    }
};

async function main() {
    const collector = new TelemetryCollector();
    const executor = new WorkflowExecutor();
    
    try {
        // Start OTLP collector
        await collector.start();
        
        // Wait a bit for collector to be ready
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        // Connect to MCP server
        await executor.connectToMCP();
        
        // Execute workflows
        await executor.executeWorkflow(WORKFLOWS.simple.name, WORKFLOWS.simple.steps);
        await executor.executeWorkflow(WORKFLOWS.notepad.name, WORKFLOWS.notepad.steps);
        await executor.executeWorkflow(WORKFLOWS.desktop.name, WORKFLOWS.desktop.steps);
        
        // Wait for telemetry to arrive
        console.log('\n⏳ Waiting for telemetry data...');
        await new Promise(resolve => setTimeout(resolve, 3000));
        
        // Show summary
        collector.getSummary();
        
    } catch (error) {
        console.error('❌ Error:', error.message);
    } finally {
        await executor.cleanup();
        collector.stop();
    }
}

// Handle interrupts
process.on('SIGINT', () => {
    console.log('\n⚠️  Interrupted');
    process.exit(0);
});

if (require.main === module) {
    // Check if MCP server has telemetry
    console.log('═'.repeat(70));
    console.log('WORKFLOW EXECUTION WITH TELEMETRY COLLECTION');
    console.log('═'.repeat(70));
    console.log('\nMake sure MCP server is built with telemetry:');
    console.log('  cd terminator-mcp-agent');
    console.log('  cargo build --release --features telemetry');
    console.log('═'.repeat(70));
    
    main();
}

module.exports = { TelemetryCollector, WorkflowExecutor };