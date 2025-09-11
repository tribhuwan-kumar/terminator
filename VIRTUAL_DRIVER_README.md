# Virtual Driver Support for Terminator

This branch implements virtual display driver support for running Terminator MCP server agents on VMs without requiring RDP connections.

## Overview

The virtual driver implementation allows Terminator to:
- Run on headless Windows VMs without an active RDP session
- Create virtual display contexts that UI Automation APIs can interact with
- Support multiple concurrent automation instances on different virtual displays
- Maintain full UI automation capabilities in server/cloud environments

## Architecture

### Key Components

1. **VirtualDisplayManager** (`src/platforms/windows/virtual_display.rs`)
   - Manages virtual display lifecycle
   - Creates display contexts for UI automation
   - Handles fallback to memory-based displays

2. **WindowsEngine Integration**
   - Automatic detection of headless environments
   - Seamless initialization of virtual displays
   - Backward compatibility with normal display mode

3. **Configuration Options**
   - Customizable display resolution and color depth
   - Optional virtual display driver installation
   - Fallback mechanisms for compatibility

## Usage

### Basic Usage (Auto-detection)

The system automatically detects headless environments:

```rust
use terminator::Desktop;

// Automatically uses virtual display if in headless environment
let desktop = Desktop::new()?;
```

### Explicit Configuration

```rust
use terminator::platforms::windows::{WindowsEngine, HeadlessConfig, VirtualDisplayConfig};

let headless_config = HeadlessConfig {
    use_virtual_display: true,
    virtual_display_config: VirtualDisplayConfig {
        width: 1920,
        height: 1080,
        color_depth: 32,
        refresh_rate: 60,
        driver_path: Some("path/to/driver.inf".to_string()),
    },
    fallback_to_memory: true,
};

let engine = WindowsEngine::new_with_headless(false, false, headless_config)?;
```

### Environment Variables

- `TERMINATOR_HEADLESS=true` - Force headless mode with virtual display
- `VIRTUAL_DISPLAY_DRIVER=/path/to/driver` - Path to virtual display driver

## Testing

### Local Testing

Use the provided PowerShell script:

```powershell
# Normal mode test
.\test_virtual_display.ps1

# Headless mode test
.\test_virtual_display.ps1 -Headless

# With driver installation (requires admin)
.\test_virtual_display.ps1 -Headless -DriverPath "C:\drivers\virtual_display.inf"
```

### Manual Testing

```bash
# Build the project
cd terminator3
cargo build --release

# Run in headless mode
$env:TERMINATOR_HEADLESS="true"
cargo run --example virtual_display_test

# Run in normal mode
$env:TERMINATOR_HEADLESS="false"
cargo run --example virtual_display_test
```

## VM Deployment

### Requirements

1. Windows Server 2016+ or Windows 10/11
2. .NET Framework 4.7.2+
3. Visual C++ Redistributables
4. (Optional) Virtual display driver for better performance

### Setup Steps

1. **Install Dependencies**
   ```powershell
   # Install Visual C++ Redistributables
   winget install Microsoft.VCRedist.2015+.x64
   ```

2. **Deploy Terminator**
   ```powershell
   git clone https://github.com/mediar-ai/terminator.git
   cd terminator
   git checkout virtual-driver-support
   cargo build --release
   ```

3. **Configure for Headless Operation**
   ```powershell
   # Set system environment variable
   [System.Environment]::SetEnvironmentVariable("TERMINATOR_HEADLESS", "true", "Machine")
   ```

4. **Run as Service (Optional)**
   ```powershell
   # Create Windows service for MCP agent
   sc create TerminatorMCP binPath= "path\to\terminator-mcp-agent.exe"
   sc config TerminatorMCP start= auto
   sc start TerminatorMCP
   ```

## Virtual Display Drivers

### Option 1: Windows Virtual Display (Built-in)
- Uses Windows' built-in virtual display capabilities
- No additional driver required
- Limited to basic display operations

### Option 2: IddSampleDriver
- Microsoft's Indirect Display Driver sample
- Provides full display functionality
- Requires driver signing for production

### Option 3: Commercial Solutions
- Various commercial virtual display drivers available
- Better performance and stability
- Licensing costs

## Troubleshooting

### Common Issues

1. **"No desktop window available" error**
   - Ensure Windows Desktop Experience feature is installed
   - Check that the user account has desktop access permissions

2. **UI Automation fails in headless mode**
   - Verify virtual display is initialized: check logs for "Virtual display initialized successfully"
   - Try setting `fallback_to_memory: true` in configuration

3. **Performance issues**
   - Consider using a proper virtual display driver instead of memory fallback
   - Adjust display resolution to reduce resource usage

### Debug Logging

Enable detailed logging:
```bash
$env:RUST_LOG="terminator=debug"
cargo run --example virtual_display_test
```

## Implementation Details

### How It Works

1. **Detection**: System checks for headless environment indicators
   - No active console session (`WTSGetActiveConsoleSessionId`)
   - No desktop window available
   - `TERMINATOR_HEADLESS` environment variable

2. **Initialization**: Creates virtual display context
   - Attempts to create display device context (DC)
   - Falls back to memory DC if needed
   - Maintains session for UI automation

3. **UI Automation**: Standard operations work normally
   - Applications render to virtual display
   - UI Automation APIs enumerate elements
   - Screenshots and interactions function as expected

### Limitations

- Some applications may detect virtual displays and behave differently
- Hardware acceleration may not be available
- Display capture performance depends on driver implementation

## Future Improvements

- [ ] Support for multiple virtual displays
- [ ] Dynamic resolution changes
- [ ] Integration with container platforms
- [ ] Linux/macOS virtual display support
- [ ] Performance optimizations for large-scale deployments

## Contributing

Please test thoroughly in your target VM environment and report any issues. Contributions for additional virtual display driver support are welcome.