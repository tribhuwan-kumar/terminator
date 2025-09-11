#!/usr/bin/env node
/**
 * Simple Workflow Execution with Console Streaming
 * 
 * Executes workflows and streams step-by-step progress to console
 * No external dependencies except MCP client
 * 
 * npm install mcp
 */

const { ClientSession, StdioServerParameters } = require('mcp');
const { stdio_client } = require('mcp/client/stdio');

class WorkflowStreamer {
    constructor() {
        this.session = null;
        this.startTime = null;
    }

    async connect(serverPath = 'target/release/terminator-mcp-agent') {
        console.log(`\n🔌 Connecting to MCP server...`);
        
        const serverParams = {
            command: serverPath,
            args: [],
            env: process.env
        };

        const transport = await stdio_client(serverParams);
        this.session = new ClientSession(transport[0], transport[1]);
        await this.session.initialize();
        
        const tools = await this.session.list_tools();
        console.log(`✅ Connected! ${tools.tools.length} tools available\n`);
    }

    log(emoji, message, data = null) {
        const timestamp = new Date().toISOString().split('T')[1].slice(0, -1);
        const elapsed = this.startTime ? `+${(Date.now() - this.startTime).toString().padStart(4)}ms` : '       ';
        
        console.log(`[${timestamp}] ${elapsed} ${emoji} ${message}`);
        if (data) {
            console.log(`${''.padStart(35)}└─ ${JSON.stringify(data)}`);
        }
    }

    async executeWorkflow(name, steps) {
        console.log('═'.repeat(70));
        console.log(`WORKFLOW: ${name}`);
        console.log('═'.repeat(70));
        
        this.startTime = Date.now();
        const results = [];
        const context = {};
        
        this.log('🚀', `Starting workflow with ${steps.length} steps`);
        
        for (let i = 0; i < steps.length; i++) {
            const step = steps[i];
            const stepNum = i + 1;
            
            console.log('─'.repeat(70));
            this.log('📍', `Step ${stepNum}/${steps.length}: ${step.tool_name || step.tool}`);
            
            try {
                // Log arguments if present
                if (step.arguments && Object.keys(step.arguments).length > 0) {
                    this.log('📝', 'Arguments:', step.arguments);
                }
                
                // Execute the tool
                this.log('⚙️', 'Executing...');
                const result = await this.session.call_tool(
                    step.tool_name || step.tool,
                    { arguments: step.arguments || {} }
                );
                
                // Store result if needed
                if (step.set_env) {
                    context[step.set_env] = this.extractResult(result);
                    this.log('💾', `Saved to context.${step.set_env}`);
                }
                
                // Log success
                this.log('✅', `Step ${stepNum} completed`);
                
                // Show partial result if text
                if (result && result.content) {
                    const text = this.extractText(result);
                    if (text) {
                        const preview = text.substring(0, 100);
                        this.log('📄', `Output: ${preview}${text.length > 100 ? '...' : ''}`);
                    }
                }
                
                results.push(result);
                
            } catch (error) {
                this.log('❌', `Step ${stepNum} failed: ${error.message}`);
                
                if (!step.continue_on_error) {
                    this.log('🛑', 'Workflow aborted');
                    throw error;
                }
                
                this.log('⚠️', 'Continuing despite error...');
            }
            
            // Small delay for readability
            if (i < steps.length - 1) {
                await new Promise(r => setTimeout(r, 100));
            }
        }
        
        const duration = Date.now() - this.startTime;
        console.log('═'.repeat(70));
        this.log('🎉', `Workflow completed in ${duration}ms`);
        console.log('═'.repeat(70));
        
        return results;
    }

    extractResult(result) {
        if (!result || !result.content) return null;
        
        const items = [];
        for (const item of result.content) {
            if (item.text) items.push(item.text);
            else if (item.data) items.push(item.data);
        }
        
        return items.length === 1 ? items[0] : items;
    }

    extractText(result) {
        if (!result || !result.content) return null;
        
        for (const item of result.content) {
            if (item.text) return item.text;
        }
        return null;
    }

    async cleanup() {
        if (this.session) {
            await this.session.close();
        }
    }
}

// Example workflows
const WORKFLOWS = {
    // Simple notepad automation
    notepad: {
        name: 'Notepad Hello World',
        steps: [
            {
                tool_name: 'screenshot',
                arguments: {}
            },
            {
                tool_name: 'launch_application',
                arguments: { app_name: 'notepad' }
            },
            {
                tool_name: 'wait',
                arguments: { delay_ms: 2000 }
            },
            {
                tool_name: 'type_text',
                arguments: { text: 'Hello from MCP workflow!\nThis is automated typing.\n' }
            },
            {
                tool_name: 'wait',
                arguments: { delay_ms: 1000 }
            },
            {
                tool_name: 'type_text',
                arguments: { text: 'Current time: ' + new Date().toLocaleString() }
            },
            {
                tool_name: 'screenshot',
                arguments: {}
            }
        ]
    },

    // Calculator automation
    calculator: {
        name: 'Calculator Demo',
        steps: [
            {
                tool_name: 'launch_application',
                arguments: { app_name: 'calc' }
            },
            {
                tool_name: 'wait',
                arguments: { delay_ms: 2000 }
            },
            {
                tool_name: 'click',
                arguments: { text: '7' }
            },
            {
                tool_name: 'click',
                arguments: { text: '+' }
            },
            {
                tool_name: 'click',
                arguments: { text: '3' }
            },
            {
                tool_name: 'click',
                arguments: { text: '=' }
            },
            {
                tool_name: 'screenshot',
                arguments: {}
            }
        ]
    },

    // Desktop interaction
    desktop: {
        name: 'Desktop Interaction',
        steps: [
            {
                tool_name: 'screenshot',
                arguments: {}
            },
            {
                tool_name: 'get_desktop_elements',
                arguments: {},
                set_env: 'elements'
            },
            {
                tool_name: 'move_mouse_to',
                arguments: { x: 100, y: 100 }
            },
            {
                tool_name: 'wait',
                arguments: { delay_ms: 500 }
            },
            {
                tool_name: 'move_mouse_to',
                arguments: { x: 500, y: 300 }
            },
            {
                tool_name: 'screenshot',
                arguments: {}
            }
        ]
    }
};

async function main() {
    const streamer = new WorkflowStreamer();
    
    try {
        await streamer.connect();
        
        // Get workflow from command line or use default
        const workflowName = process.argv[2] || 'notepad';
        const workflow = WORKFLOWS[workflowName];
        
        if (!workflow) {
            console.log('Available workflows:', Object.keys(WORKFLOWS).join(', '));
            return;
        }
        
        await streamer.executeWorkflow(workflow.name, workflow.steps);
        
    } catch (error) {
        console.error('\n❌ Error:', error.message);
    } finally {
        await streamer.cleanup();
    }
}

// Handle interrupts
process.on('SIGINT', () => {
    console.log('\n\n⚠️  Interrupted');
    process.exit(0);
});

if (require.main === module) {
    main();
}

module.exports = { WorkflowStreamer, WORKFLOWS };