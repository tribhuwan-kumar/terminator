#!/usr/bin/env python3
"""
Example demonstrating the GitHub Actions-style run command syntax.
"""

import asyncio
from terminator import Desktop

async def main():
    # Initialize the desktop
    desktop = Desktop()
    
    print("GitHub Actions-style Command Examples")
    print("=" * 40)
    
    # Example 1: Simple command
    print("\n1. Simple echo command:")
    result = await desktop.run("echo 'Hello from GitHub Actions-style syntax!'")
    print(f"   Output: {result.stdout}")
    print(f"   Exit code: {result.exit_status}")
    
    # Example 2: Multi-line script
    print("\n2. Multi-line script:")
    script = """
echo 'Starting process...'
echo 'Current directory:'
pwd
echo 'Process complete!'
"""
    result = await desktop.run(script)
    print(f"   Output: {result.stdout}")
    
    # Example 3: Using specific shell
    print("\n3. Using PowerShell (on Windows):")
    if desktop.is_windows():
        result = await desktop.run(
            "Get-Date -Format 'yyyy-MM-dd HH:mm:ss'",
            shell="powershell"
        )
        print(f"   Output: {result.stdout}")
    
    # Example 4: With working directory
    print("\n4. Command with working directory:")
    result = await desktop.run(
        "ls -la",
        working_directory="/tmp"
    )
    print(f"   Files in /tmp: {result.stdout}")
    
    # Example 5: Python script execution
    print("\n5. Python code execution:")
    python_code = """
import sys
import platform
print(f'Python {sys.version}')
print(f'Platform: {platform.system()}')
"""
    result = await desktop.run(python_code, shell="python")
    print(f"   Output: {result.stdout}")
    
    # Example 6: Cross-platform compatible
    print("\n6. Cross-platform command:")
    result = await desktop.run("echo 'This works on any platform!'")
    print(f"   Output: {result.stdout}")
    
    # Backward compatibility - old syntax still works
    print("\n7. Backward compatibility (old syntax):")
    if desktop.is_windows():
        result = await desktop.run_command(
            windows_command="dir",
            unix_command="ls"
        )
    else:
        result = await desktop.run_command(
            windows_command="dir",
            unix_command="ls"
        )
    print(f"   Output: {result.stdout[:100]}...")  # First 100 chars

if __name__ == "__main__":
    asyncio.run(main())
