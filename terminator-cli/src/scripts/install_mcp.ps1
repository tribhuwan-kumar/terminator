#Requires -RunAsAdministrator

# Terminator MCP Server Installation Script
# This script runs on first boot to set up the MCP server

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

# Create log file
$logPath = "C:\TerminatorMCP\install.log"
New-Item -ItemType Directory -Force -Path "C:\TerminatorMCP" | Out-Null

function Write-Log {
    param([string]$Message)
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    "$timestamp - $Message" | Out-File -FilePath $logPath -Append
    Write-Host $Message
}

Write-Log "Starting Terminator MCP Server installation..."

try {
    # Install Chocolatey if not present
    if (!(Get-Command choco -ErrorAction SilentlyContinue)) {
        Write-Log "Installing Chocolatey..."
        Set-ExecutionPolicy Bypass -Scope Process -Force
        [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
        Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
        
        # Refresh environment
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    }

    # Install Node.js
    Write-Log "Installing Node.js..."
    choco install nodejs -y --no-progress
    
    # Refresh environment
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    
    # Verify Node.js installation
    $nodeVersion = node --version
    Write-Log "Node.js installed: $nodeVersion"
    
    # Install Git (required for some npm packages)
    Write-Log "Installing Git..."
    choco install git -y --no-progress
    
    # Install Visual Studio Build Tools (required for native modules)
    Write-Log "Installing Visual Studio Build Tools..."
    choco install visualstudio2022-workload-vctools -y --no-progress
    
    # Create MCP server directory
    $mcpDir = "C:\TerminatorMCP\server"
    New-Item -ItemType Directory -Force -Path $mcpDir | Out-Null
    Set-Location $mcpDir
    
    # Install terminator-mcp-agent globally
    Write-Log "Installing terminator-mcp-agent..."
    npm install -g terminator-mcp-agent
    
    # Create MCP HTTP server that properly handles JSON-RPC
    $serverConfig = @'
const http = require('http');
const { spawn } = require('child_process');

const PORT = 3000;
const HOST = '0.0.0.0';

console.log('Starting Terminator MCP HTTP Server...');

// Create HTTP server
const server = http.createServer((req, res) => {
    // Enable CORS
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'POST, GET, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
    
    if (req.method === 'OPTIONS') {
        res.writeHead(200);
        res.end();
        return;
    }
    
    if (req.method === 'GET' && req.url === '/health') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ 
            status: 'healthy', 
            service: 'terminator-mcp-agent',
            timestamp: new Date().toISOString()
        }));
        return;
    }
    
    if (req.method === 'POST' && req.url === '/') {
        let body = '';
        
        req.on('data', chunk => {
            body += chunk.toString();
        });
        
        req.on('end', () => {
            console.log('Received request:', body);
            
            // Spawn MCP agent
            const mcp = spawn('npx', ['-y', 'terminator-mcp-agent'], {
                stdio: ['pipe', 'pipe', 'pipe'],
                shell: true,
                env: { ...process.env, FORCE_COLOR: '0' }
            });
            
            let response = '';
            let error = '';
            let responseComplete = false;
            
            mcp.stdout.on('data', (data) => {
                const chunk = data.toString();
                response += chunk;
                
                // Check if we have a complete JSON-RPC response
                try {
                    const lines = response.split('\n').filter(line => line.trim());
                    for (const line of lines) {
                        if (line.trim().startsWith('{') && line.trim().endsWith('}')) {
                            const parsed = JSON.parse(line);
                            if (parsed.jsonrpc === '2.0' && parsed.id !== undefined) {
                                // We have a complete response
                                res.writeHead(200, { 'Content-Type': 'application/json' });
                                res.end(line);
                                responseComplete = true;
                                mcp.kill();
                                return;
                            }
                        }
                    }
                } catch (e) {
                    // Not yet a complete JSON response
                }
            });
            
            mcp.stderr.on('data', (data) => {
                error += data.toString();
                console.error('MCP stderr:', data.toString());
            });
            
            mcp.on('close', (code) => {
                if (!responseComplete) {
                    console.error('MCP closed without response. Code:', code);
                    console.error('Stdout:', response);
                    console.error('Stderr:', error);
                    
                    res.writeHead(500, { 'Content-Type': 'application/json' });
                    res.end(JSON.stringify({
                        jsonrpc: '2.0',
                        error: {
                            code: -32603,
                            message: 'Internal error',
                            data: { stderr: error, stdout: response }
                        },
                        id: null
                    }));
                }
            });
            
            // Send the request to MCP
            mcp.stdin.write(body + '\n');
            mcp.stdin.end();
        });
    } else {
        res.writeHead(404, { 'Content-Type': 'text/plain' });
        res.end('Not Found');
    }
});

server.listen(PORT, HOST, () => {
    console.log(`MCP HTTP Server running at http://${HOST}:${PORT}`);
    console.log('Health check: http://localhost:3000/health');
});

// Graceful shutdown
process.on('SIGINT', () => {
    console.log('Shutting down MCP HTTP server...');
    server.close(() => {
        process.exit(0);
    });
});
'@
    
    $serverConfig | Out-File -FilePath "$mcpDir\server.js" -Encoding UTF8
    
    # Create package.json
    $packageJson = @'
{
    "name": "terminator-mcp-http-server",
    "version": "1.0.0",
    "description": "HTTP wrapper for Terminator MCP Agent",
    "main": "server.js",
    "scripts": {
        "start": "node server.js"
    },
    "dependencies": {}
}
'@
    
    $packageJson | Out-File -FilePath "$mcpDir\package.json" -Encoding UTF8
    
    # Create Windows service for MCP HTTP server
    Write-Log "Creating Windows service..."
    
    # Install node-windows for service management
    Set-Location $mcpDir
    npm install node-windows --save
    
    # Create service installation script
    $serviceScript = @'
const Service = require('node-windows').Service;
const path = require('path');

// Create a new service object
const svc = new Service({
    name: 'Terminator MCP Server',
    description: 'Terminator Model Context Protocol HTTP Server',
    script: path.join(__dirname, 'server.js'),
    nodeOptions: [
        '--harmony',
        '--max_old_space_size=4096'
    ],
    env: {
        name: 'NODE_ENV',
        value: 'production'
    }
});

// Listen for the "install" event
svc.on('install', function() {
    console.log('Service installed successfully');
    svc.start();
});

svc.on('start', function() {
    console.log('Service started successfully');
});

svc.on('error', function(err) {
    console.error('Service error:', err);
});

// Install the service
svc.install();
'@
    
    $serviceScript | Out-File -FilePath "$mcpDir\install-service.js" -Encoding UTF8
    
    # Run the service installation
    node install-service.js
    
    # Configure Windows Firewall for MCP HTTP port
    Write-Log "Configuring Windows Firewall..."
    New-NetFirewallRule -DisplayName "Terminator MCP HTTP Server" -Direction Inbound -Protocol TCP -LocalPort 3000 -Action Allow -ErrorAction SilentlyContinue
    
    # Enable WinRM for remote management
    Write-Log "Configuring WinRM..."
    Enable-PSRemoting -Force -SkipNetworkProfileCheck
    Set-Item WSMan:\localhost\Service\Auth\Basic -Value $true
    Set-Item WSMan:\localhost\Service\AllowUnencrypted -Value $true
    Set-NetFirewallRule -Name "WINRM-HTTP-In-TCP" -RemoteAddress Any
    
    # Create startup verification script
    $verifyScript = @'
$service = Get-Service -Name "Terminator MCP Server" -ErrorAction SilentlyContinue
if ($service -and $service.Status -eq "Running") {
    Write-Host "✅ Terminator MCP Server is running"
    
    # Test HTTP endpoint
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:3000/health" -UseBasicParsing
        Write-Host "✅ MCP HTTP endpoint is responding: $($response.Content)"
    } catch {
        Write-Host "❌ MCP HTTP endpoint is not responding"
    }
} else {
    Write-Host "❌ Terminator MCP Server is not running"
}
'@
    
    $verifyScript | Out-File -FilePath "C:\TerminatorMCP\verify-service.ps1" -Encoding UTF8
    
    Write-Log "Installation completed successfully!"
    Write-Log "MCP HTTP Server is running on port 3000"
    Write-Log "Access the server at: http://<VM-IP>:3000"
    
    # Create desktop shortcut for logs
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut("$env:USERPROFILE\Desktop\Terminator MCP Logs.lnk")
    $Shortcut.TargetPath = "notepad.exe"
    $Shortcut.Arguments = $logPath
    $Shortcut.Save()
    
} catch {
    Write-Log "ERROR: $_"
    Write-Log $_.Exception.StackTrace
    exit 1
}

# Final status
Write-Log "==================================="
Write-Log "Terminator MCP Server Setup Complete"
Write-Log "Service Status: Running"
Write-Log "HTTP Endpoint: http://0.0.0.0:3000"
Write-Log "Health Check: http://localhost:3000/health"
Write-Log "Logs: $logPath"
Write-Log "==================================="