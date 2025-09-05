# Run Command Tool Documentation

## Overview

The `run_command` tool allows you to execute shell commands using GitHub Actions-style syntax. This provides a simpler, more intuitive interface compared to platform-specific commands.

## GitHub Actions-Style Syntax

The new syntax follows the [GitHub Actions convention](https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/add-scripts) for running scripts and commands.

### Tool Arguments

- **`run`** (required): The command to execute. Can be a single command or multi-line script.
- **`shell`** (optional): The shell to use for execution. Defaults to:
  - Windows: `powershell`
  - Unix/Linux/macOS: `bash`
- **`working_directory`** (optional): The directory where the command should be executed. Defaults to the current directory.

### Supported Shells

#### Windows
- `powershell` (default) - PowerShell Core or Windows PowerShell
- `pwsh` - PowerShell Core explicitly
- `cmd` - Windows Command Prompt
- `bash` - Git Bash or WSL Bash
- `sh` - Shell (typically Git Bash on Windows)

#### Unix/Linux/macOS
- `bash` (default) - Bash shell
- `sh` - POSIX shell
- `zsh` - Z shell
- `python` - Execute as Python code
- `node` - Execute as JavaScript code

## Examples

### Basic Command Execution

```json
{
  "run": "echo 'Hello, World!'"
}
```

### Multi-line Script

```json
{
  "run": "echo 'Starting process...'\nls -la\necho 'Process complete!'"
}
```

### Using a Specific Shell

```json
{
  "run": "Get-Process | Select-Object -First 5",
  "shell": "powershell"
}
```

### With Working Directory

```json
{
  "run": "npm install",
  "working_directory": "./my-project"
}
```

### Cross-Platform Script

```json
{
  "run": "python --version",
  "shell": "python"
}
```

## Platform-Specific Behavior

The tool automatically handles platform differences:

### Windows
- Default shell is PowerShell
- Working directory changes use PowerShell's `cd` command
- Commands are executed with appropriate escaping

### Unix/Linux/macOS
- Default shell is Bash
- Working directory changes use `cd` command
- Standard Unix command execution

## Output Format

The tool returns a JSON object with the following fields:

```json
{
  "exit_status": 0,
  "stdout": "Command output here",
  "stderr": "Any error output",
  "command": "echo 'Hello'",
  "shell": "bash",
  "working_directory": "/home/user"
}
```

## Migration from Old Syntax

### Old Syntax
```json
{
  "windows_command": "dir",
  "unix_command": "ls"
}
```

### New Syntax
```json
{
  "run": "ls"
}
```
The new syntax automatically detects the platform and uses the appropriate command execution method.

## Best Practices

1. **Use the default shell** when possible for better cross-platform compatibility
2. **Prefer simple commands** over complex scripts for reliability
3. **Test commands locally** before using in automation
4. **Use working_directory** instead of `cd` commands when possible
5. **Handle errors** by checking the `exit_status` field

## Error Handling

Commands that fail will return:
- Non-zero `exit_status`
- Error details in `stderr`
- Original command information for debugging

## Security Considerations

- Commands are executed with the same privileges as the MCP server
- Avoid executing untrusted input
- Use proper escaping for user-provided values
- Consider using working_directory to limit file system access
