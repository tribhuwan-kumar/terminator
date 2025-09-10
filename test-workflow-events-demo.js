/**
 * Demonstration of workflow execution event streaming
 * This shows what the client should receive when executing a workflow
 */

const EventEmitter = require('events');

// Mock telemetry collector that would receive OpenTelemetry traces
class MockTelemetryCollector extends EventEmitter {
    constructor() {
        super();
        this.traces = [];
    }

    // Simulate receiving a trace span
    receiveSpan(span) {
        this.traces.push(span);
        this.emit('span', span);
    }
}

// Mock MCP Server that executes workflows and emits telemetry
class MockMCPServer {
    constructor(telemetry) {
        this.telemetry = telemetry;
    }

    async executeSequence(steps) {
        const workflowId = `workflow_${Date.now()}`;
        
        // Start workflow span
        this.telemetry.receiveSpan({
            type: 'workflow_start',
            spanId: workflowId,
            name: 'execute_sequence',
            timestamp: new Date().toISOString(),
            attributes: {
                total_steps: steps.length,
                request_id: workflowId
            }
        });

        const results = [];
        
        for (let i = 0; i < steps.length; i++) {
            const step = steps[i];
            const stepSpanId = `${workflowId}_step_${i}`;
            
            // Start step span
            this.telemetry.receiveSpan({
                type: 'step_start',
                spanId: stepSpanId,
                parentSpanId: workflowId,
                name: `step.${step.tool_name}`,
                timestamp: new Date().toISOString(),
                attributes: {
                    tool_name: step.tool_name,
                    step_id: step.id || `step_${i}`,
                    step_index: i,
                    arguments: step.arguments
                }
            });

            // Simulate tool execution
            await this.simulateToolExecution(step);
            
            // End step span
            const success = Math.random() > 0.1; // 90% success rate
            this.telemetry.receiveSpan({
                type: 'step_end',
                spanId: stepSpanId,
                parentSpanId: workflowId,
                timestamp: new Date().toISOString(),
                status: success ? 'OK' : 'ERROR',
                attributes: {
                    tool_name: step.tool_name,
                    step_id: step.id || `step_${i}`,
                    result: success ? 'completed' : 'failed',
                    error: success ? null : 'Simulated error'
                }
            });

            results.push({
                step: i,
                tool: step.tool_name,
                status: success ? 'success' : 'error',
                duration_ms: Math.floor(Math.random() * 1000)
            });
        }

        // End workflow span
        this.telemetry.receiveSpan({
            type: 'workflow_end',
            spanId: workflowId,
            timestamp: new Date().toISOString(),
            status: 'OK',
            attributes: {
                total_duration_ms: results.reduce((acc, r) => acc + r.duration_ms, 0),
                executed_steps: steps.length,
                results: results
            }
        });

        return results;
    }

    async simulateToolExecution(step) {
        const delay = step.arguments?.delay_ms || 100;
        await new Promise(resolve => setTimeout(resolve, delay));
    }
}

// Client that listens to telemetry events
class WorkflowEventClient {
    constructor(telemetry) {
        this.telemetry = telemetry;
        this.events = [];
        
        // Listen to telemetry spans and convert to events
        telemetry.on('span', (span) => {
            this.handleSpan(span);
        });
    }

    handleSpan(span) {
        let event = null;
        
        switch(span.type) {
            case 'workflow_start':
                event = {
                    type: 'WORKFLOW_STARTED',
                    timestamp: span.timestamp,
                    data: {
                        workflow_id: span.spanId,
                        total_steps: span.attributes.total_steps
                    }
                };
                console.log('\n🚀 WORKFLOW STARTED');
                console.log(`   ID: ${span.spanId}`);
                console.log(`   Total steps: ${span.attributes.total_steps}`);
                break;
                
            case 'step_start':
                event = {
                    type: 'STEP_STARTED',
                    timestamp: span.timestamp,
                    data: {
                        tool: span.attributes.tool_name,
                        step_id: span.attributes.step_id,
                        index: span.attributes.step_index
                    }
                };
                console.log(`\n⚙️  STEP ${span.attributes.step_index + 1} STARTED`);
                console.log(`   Tool: ${span.attributes.tool_name}`);
                console.log(`   ID: ${span.attributes.step_id}`);
                if (span.attributes.arguments) {
                    console.log(`   Args: ${JSON.stringify(span.attributes.arguments)}`);
                }
                break;
                
            case 'step_end':
                event = {
                    type: 'STEP_COMPLETED',
                    timestamp: span.timestamp,
                    data: {
                        tool: span.attributes.tool_name,
                        step_id: span.attributes.step_id,
                        status: span.status,
                        result: span.attributes.result
                    }
                };
                const statusIcon = span.status === 'OK' ? '✅' : '❌';
                console.log(`${statusIcon} STEP COMPLETED: ${span.attributes.tool_name}`);
                if (span.attributes.error) {
                    console.log(`   Error: ${span.attributes.error}`);
                }
                break;
                
            case 'workflow_end':
                event = {
                    type: 'WORKFLOW_COMPLETED',
                    timestamp: span.timestamp,
                    data: {
                        workflow_id: span.spanId,
                        duration_ms: span.attributes.total_duration_ms,
                        executed_steps: span.attributes.executed_steps,
                        status: span.status
                    }
                };
                console.log('\n🏁 WORKFLOW COMPLETED');
                console.log(`   Duration: ${span.attributes.total_duration_ms}ms`);
                console.log(`   Executed: ${span.attributes.executed_steps} steps`);
                console.log(`   Status: ${span.status}`);
                break;
        }
        
        if (event) {
            this.events.push(event);
        }
    }
}

// Run demonstration
async function runDemo() {
    console.log('═'.repeat(60));
    console.log('WORKFLOW EXECUTION EVENT STREAM DEMONSTRATION');
    console.log('═'.repeat(60));
    console.log('\nThis demonstrates what a client receives when executing');
    console.log('a workflow through the MCP server with OpenTelemetry.');
    console.log('═'.repeat(60));

    // Set up components
    const telemetry = new MockTelemetryCollector();
    const server = new MockMCPServer(telemetry);
    const client = new WorkflowEventClient(telemetry);

    // Define a test workflow
    const workflow = [
        {
            tool_name: 'screenshot',
            id: 'capture_initial',
            arguments: {}
        },
        {
            tool_name: 'click',
            id: 'click_button',
            arguments: {
                selector: '#submit-button'
            }
        },
        {
            tool_name: 'wait',
            id: 'wait_for_load',
            arguments: {
                delay_ms: 500
            }
        },
        {
            tool_name: 'type_text',
            id: 'enter_username',
            arguments: {
                selector: '#username',
                text: 'test_user'
            }
        },
        {
            tool_name: 'screenshot',
            id: 'capture_final',
            arguments: {}
        }
    ];

    console.log('\nExecuting workflow with', workflow.length, 'steps...');
    console.log('─'.repeat(60));

    // Execute workflow
    const results = await server.executeSequence(workflow);

    console.log('\n═'.repeat(60));
    console.log('EVENT STREAM SUMMARY');
    console.log('─'.repeat(60));
    console.log(`Total events received: ${client.events.length}`);
    console.log('\nEvent types:');
    const eventCounts = {};
    client.events.forEach(e => {
        eventCounts[e.type] = (eventCounts[e.type] || 0) + 1;
    });
    Object.entries(eventCounts).forEach(([type, count]) => {
        console.log(`  ${type}: ${count}`);
    });

    console.log('\n═'.repeat(60));
    console.log('RAW EVENT STREAM (JSON format):');
    console.log('─'.repeat(60));
    client.events.forEach((event, i) => {
        console.log(`\nEvent ${i + 1}:`, JSON.stringify(event, null, 2));
    });

    console.log('\n═'.repeat(60));
    console.log('OPENTELEMETRY TRACE SPANS:');
    console.log('─'.repeat(60));
    console.log(`Total spans: ${telemetry.traces.length}`);
    console.log('\nSpan hierarchy:');
    telemetry.traces.forEach(span => {
        const indent = span.parentSpanId ? '  └─ ' : '├─ ';
        console.log(`${indent}${span.name || span.type} [${span.spanId}]`);
    });

    console.log('\n═'.repeat(60));
    console.log('WHAT THIS MEANS:');
    console.log('─'.repeat(60));
    console.log('1. Each workflow execution creates a parent span');
    console.log('2. Each step in the sequence creates a child span');
    console.log('3. Spans include timing, status, and context');
    console.log('4. OpenTelemetry exports these to any compatible backend');
    console.log('5. Clients can subscribe to real-time updates via:');
    console.log('   - Direct OTLP subscription');
    console.log('   - Backend-specific APIs (Jaeger, Zipkin, etc.)');
    console.log('   - Custom event streaming from the telemetry data');
    console.log('\n═'.repeat(60));
}

// Run the demonstration
runDemo().catch(console.error);