param(
    [string]$App
)

Get-WmiObject Win32_Process | Where-Object { $_.ExecutablePath -like '*terminator-mcp-agent*' } | ForEach-Object {
  taskkill.exe /T /F /PID $_.ProcessId *>$null
}
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

    # Minimal pretty-print JSON function
    function Format-Json($Json, $Indentation = 4) {
        $indent = 0
        $regexUnlessQuoted = '(?=([^"\"]*\"[^"\"]*\")*[^"\"]*$)'
        if ($Json -notmatch '\r?\n') {
            $Json = ($Json | ConvertFrom-Json) | ConvertTo-Json -Depth 100
        }
        $result = $Json -split '\r?\n' |
            ForEach-Object {
                if ($_ -match "[}\]]$regexUnlessQuoted") {
                    $indent = [Math]::Max($indent - $Indentation, 0)
                }
                $line = (' ' * $indent) + ($_.TrimStart() -replace ":\s+$regexUnlessQuoted", ': ')
                if ($_ -match "[\{\[]$regexUnlessQuoted") {
                    $indent += $Indentation
                }
                $line
            }
        return $result -Join [Environment]::NewLine
    }

    function Save-PrettyJson($Path, $Object) {
        $json = $Object | ConvertTo-Json -Depth 10
        $pretty = Format-Json $json 4
        Set-Content -Path $Path -Value $pretty
    }

    # Helper: Load or initialize MCP config file
    function Load-Or-Init-McpConfig($ConfigFile) {
        if (-not (Test-Path $ConfigFile)) {
            New-Item -ItemType File -Path $ConfigFile -Force | Out-Null
        }
        $configContent = $null
        try {
            $configContent = Get-Content -Raw -Path $ConfigFile | ConvertFrom-Json
        } catch {
            $configContent = $null
        }
        if (-not $configContent -or $configContent.PSObject.Properties.Value.Count -eq 0) {
            $configContent = [PSCustomObject]@{ mcpServers = @{} }
        }
        if (-not $configContent.mcpServers) {
            $configContent.mcpServers = @{}
        } elseif (-not ($configContent.mcpServers -is [hashtable])) {
            $tempHashtable = @{}
            foreach ($property in $configContent.mcpServers.PSObject.Properties) {
                $tempHashtable[$property.Name] = $property.Value
            }
            $configContent.mcpServers = $tempHashtable
        }
        return $configContent
    }

    # Helper: Ensure a directory exists (DRY)
    function Ensure-Directory($Path) {
        if (!(Test-Path $Path)) {
            New-Item -ItemType Directory -Path $Path -Force | Out-Null
        }
    }

    # Helper: Ensure a file exists (DRY)
    function Ensure-File($Path) {
        if (-not (Test-Path $Path)) {
            New-Item -ItemType File -Path $Path -Force | Out-Null
        }
    }

    # Helper: Add/Update MCP agent in config and save, ensuring dir/file if needed
    function Add-McpAgent-To-Config($ConfigFile, $EnsureDir = $false) {
        if ($EnsureDir) {
            Ensure-Directory (Split-Path $ConfigFile)
        }
        Ensure-File $ConfigFile
        $configContent = Load-Or-Init-McpConfig $ConfigFile
        $configContent.mcpServers["terminator-mcp-agent"] = @{
            command = "terminator-mcp-agent.exe"
            args = @()
        }
        Save-PrettyJson -Path $ConfigFile -Object $configContent
    }

    if ($App) {
        switch ($App.ToLower()) {
            "cursor"   { $choice = 1 }
            "claude"   { $choice = 2 }
            "vscode"   { $choice = 3 }
            "insiders" { $choice = 4 }
            "windsurf" { $choice = 5 }
            default    { Write-Host "Unknown app: $App"; exit 1 }
        }
    } else {
        Write-Host ""
        Write-Host "========== Terminator MCP Setup =========="
        Write-Host "Which app do you want to configure Terminator MCP for?"
        Write-Host ""
        Write-Host "  1. Cursor"
        Write-Host "  2. Claude"
        Write-Host "  3. VS Code"
        Write-Host "  4. VS Code Insiders"
        Write-Host "  5. Windsurf"
        Write-Host ""
        $choice = Read-Host "Enter your choice (1-5)"
    }

    switch ($choice) {
        1 {
            # Cursor config
            $cursorConfigFile = Join-Path "$env:USERPROFILE\.cursor" "mcp.json"
            Add-McpAgent-To-Config $cursorConfigFile $true
            Write-Host "Cursor configuration saved to $cursorConfigFile"
        }
        2 {
            # Claude config
            $claudeConfigFile = "$env:APPDATA\Claude\claude_desktop_config.json"
            if (!(Test-Path (Split-Path $claudeConfigFile))) {
                throw "You've likely not installed the Claude desktop app, please install it!!"
            }
            Add-McpAgent-To-Config $claudeConfigFile $false
            Write-Host "Claude configuration saved to $claudeConfigFile"
        }
        3 {
            # VS Code CLI-based setup
            Write-Host "Adding Terminator MCP to VS Code via code CLI..."
            $json = '{"name":"terminator-mcp-agent","command":"terminator-mcp-agent.exe","args":[]}'
            $vscodePaths = @(
                "$env:LOCALAPPDATA\Programs\Microsoft VS Code\bin\code.cmd",
                "${env:ProgramFiles}\Microsoft VS Code\bin\code.cmd",
                "${env:ProgramFiles(x86)}\Microsoft VS Code\bin\code.cmd"
            )
            $vscodeCmd = $null
            foreach ($path in $vscodePaths) {
                if (Test-Path $path) {
                    $vscodeCmd = $path
                    break
                }
            }
            if (-not $vscodeCmd) {
                throw "VS Code CLI not found in standard installation paths. Please make sure VS Code is installed properly."
            }
            & $vscodeCmd --add-mcp $json
            Write-Host "Successfully added Terminator MCP to VS Code."
        }
        4 {
            # VS Code Insiders CLI-based setup
            Write-Host "Adding Terminator MCP to VS Code Insiders via code-insiders CLI..."
            $json = '{"name":"terminator-mcp-agent","command":"terminator-mcp-agent.exe","args":[]}'
            $codePath = Get-Command "code-insiders" -ErrorAction SilentlyContinue
            if (-not $codePath) {
                throw "'code-insiders' command not found in PATH. Make sure VS Code Insiders CLI is installed and available."
            }
            & code-insiders --add-mcp $json
            Write-Host "Successfully added Terminator MCP to VS Code Insiders."
        }
        5 {
            # Windsurf config file-based setup (Cursor style)
            $windsurfConfigFile = Join-Path "$env:USERPROFILE\.codeium\windsurf" "mcp_config.json"
            Add-McpAgent-To-Config $windsurfConfigFile $true
            Write-Host "Windsurf configuration saved to $windsurfConfigFile"
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
