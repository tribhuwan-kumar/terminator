Write-Host "installing Terminator MCP..."

try {
    # get latest version
    $releases = Invoke-RestMethod "https://api.github.com/repos/mediar-ai/terminator/releases"
    $latestRelease = $releases | Where-Object { -not $_.prerelease } | Select-Object -First 1
    if (-not $latestRelease) {
        throw "no releases found"
    }

    # find the Windows asset
    $asset = $latestRelease.assets | Where-Object { $_.name -like "terminator-mcp-windows-x86_64.zip" } | Select-Object -First 1
    if (-not $asset) {
        throw "no Terminator MCP release found in version $($latestRelease.tag_name)"
    }

    $url = $asset.browser_download_url

    $installDir = "$env:USERPROFILE\.terminator"
    $tempZip = "$env:TEMP\terminator-mcp-windows-x86_64.zip"

    # download and extract
    Write-Host "downloading latest version ($($latestRelease.tag_name)) from $url..."
    Invoke-WebRequest -Uri $url -OutFile $tempZip

    # create install directory if it doesn't exist
    if (!(Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir | Out-Null
    }

    Expand-Archive -Path $tempZip -DestinationPath $installDir -Force

    # add to PATH if not already there
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$installDir\*") {
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$installDir\", "User")
        $env:Path = [Environment]::GetEnvironmentVariable("Path", "User")
    }

    # verify installation
    $binPath = Join-Path $installDir "terminator-mcp-agent.exe"
    if (!(Test-Path $binPath)) {
        throw "terminator-mcp-agent.exe not found in $binPath after installation"
    }

    Write-Host "Terminator MCP added to the PATH"

    Write-Host "Initializing Terminator MCP setup for desktop apps"
    Write-Host "Which app do you want to configure Terminator MCP for?"
    Write-Host "1. Cursor"
    Write-Host "2. Claude"
    $choice = Read-Host "Enter your choice (1 or 2)"

    switch ($choice) {
        1 {
            # configure for Cursor
            $cursorConfigDir = "$env:USERPROFILE\.cursor"
            if (!(Test-Path $cursorConfigDir)) {
                New-Item -ItemType Directory -Path $cursorConfigDir | Out-Null
            }
            $cursorConfigFile = Join-Path $cursorConfigDir "mcp.json"

            if (Test-Path $cursorConfigFile) {
                $cursorConfigContent = Get-Content -Path $cursorConfigFile -Raw | ConvertFrom-Json
            } else {
                $cursorConfigContent = @{
                    mcpServers = @{}
                }
            }

            # ensure mcpServers key exists
            if (-not $cursorConfigContent.PSObject.Properties["mcpServers"]) {
                $cursorConfigContent.mcpServers = @{}
            }

            $cursorConfigContent.mcpServers["terminator-mcp-agent"] = @{
                command = "terminator-mcp-agent.exe"
                args = @()
            }

            $cursorConfigContent | ConvertTo-Json -Depth 10 | Set-Content -Path $cursorConfigFile
            Write-Host "Cursor configuration saved to $cursorConfigFile"
        }
        2 {
            # Configure for Claude
            $claudeConfigFile = "$env:APPDATA\Claude\claude_desktop_config.json"
            if (!(Test-Path (Split-Path $claudeConfigFile))) {
                New-Item -ItemType Directory -Path (Split-Path $claudeConfigFile) | Out-Null
            }

            if (Test-Path $claudeConfigFile) {
                $claudeConfigContent = Get-Content -Path $claudeConfigFile -Raw | ConvertFrom-Json
            } else {
                $claudeConfigContent = @{
                    mcpServers = @{}
                }
            }

            if (-not $claudeConfigContent.PSObject.Properties["mcpServers"]) {
                $claudeConfigContent.mcpServers = @{}
            }

            $claudeConfigContent.mcpServers["terminator-mcp-agent"] = @{
                command = "terminator-mcp-agent.exe"
                args = @()
            }

            $claudeConfigContent | ConvertTo-Json -Depth 10 | Set-Content -Path $claudeConfigFile
            Write-Host "Claude configuration saved to $claudeConfigFile"
        }
        Default {
            Write-Host "Invalid choice. Skipping app configuration."
        }
    }

    # cleanup
    Remove-Item $tempZip -Force
    Write-Host "terminator-mcp-agent installed successfully, restart the app"
} catch {
    Write-Host "installation failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
