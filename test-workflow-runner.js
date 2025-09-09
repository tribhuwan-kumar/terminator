const fetch = require('node-fetch');
const fs = require('fs');
const yaml = require('js-yaml');
const path = require('path');

// MCP endpoint configuration
const MCP_URL = 'http://13.77.110.245:8080';
const WORKFLOW_FILE = './workflows/simple-notepad-test.yml';

async function runWorkflow() {
    try {
        console.log('📋 Loading workflow from:', WORKFLOW_FILE);
        
        // Read and parse the YAML workflow
        const workflowContent = fs.readFileSync(WORKFLOW_FILE, 'utf8');
        const workflow = yaml.load(workflowContent);
        
        console.log(`🚀 Running workflow: ${workflow.name}`);
        console.log(`📝 Description: ${workflow.description}`);
        
        // Prepare the execute_sequence request
        const request = {
            jsonrpc: '2.0',
            method: 'tools/call',
            params: {
                name: 'execute_sequence',
                arguments: {
                    steps: workflow.steps.map(step => ({
                        tool_name: step.tool_name,
                        arguments: step.arguments,
                        delay_ms: step.delay_ms,
                        continue_on_error: step.continue_on_error || false
                    })),
                    variables: workflow.variables,
                    inputs: {
                        test_text: workflow.variables?.test_text?.default || "Hello from AVD!"
                    },
                    stop_on_error: false,
                    include_detailed_results: true
                }
            },
            id: Date.now()
        };
        
        console.log('\n📡 Sending request to MCP endpoint:', MCP_URL);
        console.log('Request:', JSON.stringify(request, null, 2));
        
        // Send the request
        const response = await fetch(MCP_URL, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(request),
            timeout: 60000 // 60 second timeout
        });
        
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const result = await response.json();
        
        console.log('\n✅ Workflow execution completed!');
        console.log('Response:', JSON.stringify(result, null, 2));
        
        // Check for errors in the result
        if (result.error) {
            console.error('❌ Error:', result.error);
            return false;
        }
        
        // Parse the results
        if (result.result) {
            console.log('\n📊 Execution Summary:');
            if (result.result.success !== undefined) {
                console.log(`Status: ${result.result.success ? '✅ Success' : '❌ Failed'}`);
            }
            if (result.result.executed_steps) {
                console.log(`Executed steps: ${result.result.executed_steps}`);
            }
            if (result.result.failed_steps) {
                console.log(`Failed steps: ${result.result.failed_steps}`);
            }
        }
        
        return true;
        
    } catch (error) {
        console.error('❌ Failed to run workflow:', error.message);
        
        // Try a simple status check first
        console.log('\n🔍 Attempting to check MCP status...');
        try {
            const statusResponse = await fetch(`${MCP_URL}/status`, {
                method: 'GET',
                timeout: 5000
            });
            
            if (statusResponse.ok) {
                const status = await statusResponse.text();
                console.log('MCP Status:', status);
            } else {
                console.log('MCP endpoint returned:', statusResponse.status, statusResponse.statusText);
            }
        } catch (statusError) {
            console.log('Cannot reach MCP endpoint. Please check:');
            console.log('1. The Azure VMs are running');
            console.log('2. The MCP agent is installed and running');
            console.log('3. Network security groups allow traffic on port 8080');
            console.log('4. The load balancer is properly configured');
        }
        
        return false;
    }
}

// Check if required modules are installed
function checkDependencies() {
    const requiredModules = ['node-fetch', 'js-yaml'];
    const missing = [];
    
    for (const module of requiredModules) {
        try {
            require.resolve(module);
        } catch (e) {
            missing.push(module);
        }
    }
    
    if (missing.length > 0) {
        console.log('📦 Installing required dependencies:', missing.join(', '));
        const { execSync } = require('child_process');
        execSync(`npm install ${missing.join(' ')}`, { stdio: 'inherit' });
    }
}

// Main execution
console.log('🤖 Terminator MCP Workflow Runner');
console.log('================================\n');

checkDependencies();

runWorkflow().then(success => {
    if (success) {
        console.log('\n✨ Workflow completed successfully!');
    } else {
        console.log('\n⚠️ Workflow execution failed or encountered errors.');
    }
    process.exit(success ? 0 : 1);
});