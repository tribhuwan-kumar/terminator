# Bridge Health Monitor with Self-Healing
# Continuous monitoring script for WebSocket bridge and MCP server

$ErrorActionPreference = "SilentlyContinue"

# Configuration
$WS_PORT = 17373
$MCP_PORT = 8080
$CHECK_INTERVAL = 5  # seconds
$MAX_FAILURES = 3

# State tracking
$script:ConsecutiveFailures = 0
$script:ConnectionDrops = 0
$script:LastHealthCheck = Get-Date
$script:HealthHistory = @()

function Write-Log {
    param($Level, $Message)
    $timestamp = Get-Date -Format "HH:mm:ss"
    $symbols = @{
        "INFO" = "[i]"
        "SUCCESS" = "[+]"
        "ERROR" = "[!]"
        "WARNING" = "[?]"
        "HEAL" = "[*]"
    }
    $symbol = $symbols[$Level]
    
    $color = switch ($Level) {
        "SUCCESS" { "Green" }
        "ERROR" { "Red" }
        "WARNING" { "Yellow" }
        "HEAL" { "Cyan" }
        default { "White" }
    }
    
    Write-Host "[$timestamp] $symbol $Message" -ForegroundColor $color
}

function Test-Port {
    param($Port)
    $result = netstat -an | Select-String ":$Port.*LISTENING"
    return $null -ne $result
}

function Test-MCPHealth {
    try {
        $health = Invoke-RestMethod -Uri "http://127.0.0.1:$MCP_PORT/health" -TimeoutSec 2
        $status = Invoke-RestMethod -Uri "http://127.0.0.1:$MCP_PORT/status" -TimeoutSec 2
        
        return @{
            Healthy = ($health.status -eq "ok")
            Busy = $status.busy
            ActiveRequests = $status.activeRequests
            LastActivity = $status.lastActivity
        }
    } catch {
        return @{ Healthy = $false }
    }
}

function Test-WebSocketBridge {
    $listening = Test-Port -Port $WS_PORT
    $connections = netstat -an | Select-String ":$WS_PORT.*ESTABLISHED" | Measure-Object | Select -ExpandProperty Count
    
    return @{
        Listening = $listening
        Connections = $connections
    }
}

function Get-ExtensionProcesses {
    $processes = Get-Process | Where-Object { $_.Name -like "*terminator-mcp*" }
    return $processes.Count
}

function Invoke-SelfHealing {
    Write-Log "HEAL" "Initiating self-healing sequence..."
    
    # Check if port is blocked
    if (-not (Test-Port -Port $WS_PORT)) {
        Write-Log "ERROR" "WebSocket port $WS_PORT not listening"
        
        # Check for port conflicts
        $portUser = netstat -anb 2>$null | Select-String ":$WS_PORT" -Context 0,1
        if ($portUser) {
            Write-Log "WARNING" "Port may be in use by another process"
        }
        
        # Suggest restart
        Write-Log "HEAL" "Recommended action: Restart MCP agent or reload Chrome extension"
    }
    
    # Check Chrome/Edge running
    $browserRunning = (Get-Process chrome, msedge -ErrorAction SilentlyContinue).Count -gt 0
    if (-not $browserRunning) {
        Write-Log "WARNING" "No browser detected - extension cannot connect"
    }
    
    # Reset failure counter
    $script:ConsecutiveFailures = 0
}

function Show-Dashboard {
    param($WSStatus, $MCPStatus, $HealthScore)
    
    Clear-Host
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host "               BRIDGE HEALTH MONITOR v1.0                  " -ForegroundColor Cyan
    Write-Host "============================================================" -ForegroundColor Cyan
    
    # WebSocket Status
    $wsIcon = if ($WSStatus.Listening) { "[OK]" } else { "[!!]" }
    $wsColor = if ($WSStatus.Listening) { "Green" } else { "Red" }
    Write-Host "WebSocket Bridge (17373): " -NoNewline
    Write-Host "$wsIcon" -ForegroundColor $wsColor -NoNewline
    Write-Host " Listening: $($WSStatus.Listening), Connections: $($WSStatus.Connections)"
    
    # MCP Status
    $mcpIcon = if ($MCPStatus.Healthy) { "[OK]" } else { "[!!]" }
    $mcpColor = if ($MCPStatus.Healthy) { "Green" } else { "Red" }
    Write-Host "MCP Server (8080):       " -NoNewline
    Write-Host "$mcpIcon" -ForegroundColor $mcpColor -NoNewline
    Write-Host " Healthy: $($MCPStatus.Healthy), Busy: $($MCPStatus.Busy)"
    
    if ($MCPStatus.ActiveRequests) {
        Write-Host "  Active Requests: $($MCPStatus.ActiveRequests)"
    }
    
    # Processes
    $procCount = Get-ExtensionProcesses
    Write-Host "MCP Processes:            [$procCount] running"
    
    # Stats
    Write-Host ""
    Write-Host "Connection Drops:         $($script:ConnectionDrops)"
    Write-Host "Consecutive Failures:     $($script:ConsecutiveFailures)"
    Write-Host "Last Check:              " -NoNewline
    Write-Host "$($script:LastHealthCheck.ToString('HH:mm:ss'))" -ForegroundColor Gray
    
    # Health Score
    Write-Host ""
    Write-Host "============================================================" -ForegroundColor Cyan
    $scoreColor = if ($HealthScore -ge 80) { "Green" } 
                  elseif ($HealthScore -ge 60) { "Yellow" } 
                  else { "Red" }
    Write-Host "OVERALL HEALTH SCORE: " -NoNewline
    Write-Host "$HealthScore%" -ForegroundColor $scoreColor
    
    # Recent Issues
    if ($script:HealthHistory.Count -gt 0) {
        Write-Host ""
        Write-Host "Recent Events:" -ForegroundColor Yellow
        $script:HealthHistory | Select-Object -Last 5 | ForEach-Object {
            Write-Host "  $_" -ForegroundColor Gray
        }
    }
    
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host "Press Ctrl+C to stop monitoring" -ForegroundColor Gray
}

# Main monitoring loop
Write-Log "INFO" "Starting Bridge Health Monitor"
Write-Log "INFO" "Monitoring WebSocket port: $WS_PORT"
Write-Log "INFO" "Monitoring MCP port: $MCP_PORT"
Write-Host ""

$lastWSStatus = $null
$lastMCPStatus = $null

while ($true) {
    $script:LastHealthCheck = Get-Date
    
    # Test components
    $wsStatus = Test-WebSocketBridge
    $mcpStatus = Test-MCPHealth
    
    # Calculate health score
    $healthScore = 0
    if ($wsStatus.Listening) { $healthScore += 40 }
    if ($wsStatus.Connections -gt 0) { $healthScore += 20 }
    if ($mcpStatus.Healthy) { $healthScore += 30 }
    if ($script:ConsecutiveFailures -eq 0) { $healthScore += 10 }
    
    # Detect changes
    if ($null -ne $lastWSStatus) {
        if ($wsStatus.Listening -ne $lastWSStatus.Listening) {
            if ($wsStatus.Listening) {
                Write-Log "SUCCESS" "WebSocket bridge recovered"
                $script:HealthHistory += "$(Get-Date -Format 'HH:mm:ss') - Bridge recovered"
            } else {
                Write-Log "ERROR" "WebSocket bridge lost"
                $script:ConnectionDrops++
                $script:ConsecutiveFailures++
                $script:HealthHistory += "$(Get-Date -Format 'HH:mm:ss') - Bridge lost"
            }
        }
        
        if ($wsStatus.Connections -ne $lastWSStatus.Connections) {
            $diff = $wsStatus.Connections - $lastWSStatus.Connections
            if ($diff -gt 0) {
                Write-Log "INFO" "New connection established (Total: $($wsStatus.Connections))"
            } elseif ($diff -lt 0) {
                Write-Log "WARNING" "Connection dropped (Total: $($wsStatus.Connections))"
            }
        }
    }
    
    if ($null -ne $lastMCPStatus) {
        if ($mcpStatus.Healthy -ne $lastMCPStatus.Healthy) {
            if ($mcpStatus.Healthy) {
                Write-Log "SUCCESS" "MCP server recovered"
            } else {
                Write-Log "ERROR" "MCP server unhealthy"
                $script:ConsecutiveFailures++
            }
        }
    }
    
    # Self-healing trigger
    if ($script:ConsecutiveFailures -ge $MAX_FAILURES) {
        Invoke-SelfHealing
    }
    
    # Update dashboard
    Show-Dashboard -WSStatus $wsStatus -MCPStatus $mcpStatus -HealthScore $healthScore
    
    # Store last status
    $lastWSStatus = $wsStatus
    $lastMCPStatus = $mcpStatus
    
    # Wait before next check
    Start-Sleep -Seconds $CHECK_INTERVAL
}