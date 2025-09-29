// Test for the enhanced health endpoint with UIAutomation check
import fetch from 'node-fetch';

const MCP_PORT = process.env.MCP_PORT || 3000;
const MCP_URL = `http://localhost:${MCP_PORT}`;

async function testHealthEndpoint() {
    console.log('Testing health endpoint with UIAutomation check...');
    console.log(`URL: ${MCP_URL}/health`);

    try {
        const response = await fetch(`${MCP_URL}/health`, {
            timeout: 10000 // 10 second timeout
        });

        const status = response.status;
        const data = await response.json();

        console.log('\n=== Health Check Response ===');
        console.log(`HTTP Status: ${status}`);
        console.log('Response Body:', JSON.stringify(data, null, 2));

        // Check platform-specific fields
        if (data.platform === 'windows') {
            console.log('\n=== Windows UIAutomation Status ===');
            const uiautomation = data.uiautomation;

            if (uiautomation) {
                console.log(`Available: ${uiautomation.available}`);
                console.log(`Can Access Desktop: ${uiautomation.can_access_desktop}`);
                console.log(`Can Enumerate Children: ${uiautomation.can_enumerate_children}`);
                console.log(`Check Duration: ${uiautomation.check_duration_ms}ms`);

                if (uiautomation.error_message) {
                    console.log(`Error: ${uiautomation.error_message}`);
                }

                if (uiautomation.diagnostics) {
                    console.log('\nDiagnostics:');
                    console.log(`  COM Initialized: ${uiautomation.diagnostics.com_initialized}`);
                    console.log(`  Is Headless: ${uiautomation.diagnostics.is_headless}`);
                    console.log(`  Desktop Children: ${uiautomation.diagnostics.desktop_child_count || 'N/A'}`);
                    console.log(`  Display Info: ${uiautomation.diagnostics.display_info || 'N/A'}`);
                }

                // Overall health assessment
                let healthStatus = 'UNKNOWN';
                if (uiautomation.available && uiautomation.can_access_desktop && uiautomation.can_enumerate_children) {
                    healthStatus = 'HEALTHY';
                } else if (uiautomation.available) {
                    healthStatus = 'DEGRADED (VM/RDP issues likely)';
                } else {
                    healthStatus = 'UNHEALTHY (UIAutomation API unavailable)';
                }

                console.log(`\n=== Overall UIAutomation Health: ${healthStatus} ===`);

                // Check HTTP status matches expected
                if (status === 200 && healthStatus !== 'HEALTHY') {
                    console.error('WARNING: HTTP status 200 but UIAutomation is not healthy!');
                } else if (status === 206 && healthStatus !== 'DEGRADED (VM/RDP issues likely)') {
                    console.error('WARNING: HTTP status 206 but UIAutomation is not degraded!');
                } else if (status === 503 && healthStatus !== 'UNHEALTHY (UIAutomation API unavailable)') {
                    console.error('WARNING: HTTP status 503 but UIAutomation is not unhealthy!');
                }
            } else {
                console.log('UIAutomation health data not present in response');
            }
        } else {
            console.log(`Platform: ${data.platform} (UIAutomation check not applicable)`);
        }

        // Check extension bridge status
        if (data.extension_bridge) {
            console.log('\n=== Extension Bridge Status ===');
            console.log(JSON.stringify(data.extension_bridge, null, 2));
        }

        return status === 200 || status === 206; // Accept OK or Partial Content as success

    } catch (error) {
        console.error('Error testing health endpoint:', error);
        return false;
    }
}

// Run the test
testHealthEndpoint().then(success => {
    if (success) {
        console.log('\n✓ Health endpoint test completed successfully');
        process.exit(0);
    } else {
        console.log('\n✗ Health endpoint test failed');
        process.exit(1);
    }
});