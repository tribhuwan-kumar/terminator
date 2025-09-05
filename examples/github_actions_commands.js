#!/usr/bin/env node
/**
 * Example demonstrating the GitHub Actions-style run command syntax.
 */

const { Desktop } = require("@mediar/terminator");

async function main() {
  // Initialize the desktop
  const desktop = new Desktop();

  console.log("GitHub Actions-style Command Examples");
  console.log("=".repeat(40));

  // Example 1: Simple command
  console.log("\n1. Simple echo command:");
  let result = await desktop.run(
    "echo 'Hello from GitHub Actions-style syntax!'"
  );
  console.log(`   Output: ${result.stdout}`);
  console.log(`   Exit code: ${result.exit_status}`);

  // Example 2: Multi-line script
  console.log("\n2. Multi-line script:");
  const script = `
echo 'Starting process...'
echo 'Current directory:'
pwd
echo 'Process complete!'
`;
  result = await desktop.run(script);
  console.log(`   Output: ${result.stdout}`);

  // Example 3: Using specific shell
  console.log("\n3. Using specific shell:");
  if (process.platform === "win32") {
    result = await desktop.run(
      "Get-Date -Format 'yyyy-MM-dd HH:mm:ss'",
      "powershell"
    );
    console.log(`   PowerShell output: ${result.stdout}`);
  } else {
    result = await desktop.run("date '+%Y-%m-%d %H:%M:%S'", "bash");
    console.log(`   Bash output: ${result.stdout}`);
  }

  // Example 4: With working directory
  console.log("\n4. Command with working directory:");
  result = await desktop.run(
    "ls -la",
    null, // default shell
    "/tmp" // working directory
  );
  console.log(`   Files in /tmp: ${result.stdout}`);

  // Example 5: Node.js script execution
  console.log("\n5. Node.js code execution:");
  const nodeCode = `
console.log('Node.js version:', process.version);
console.log('Platform:', process.platform);
console.log('Architecture:', process.arch);
`;
  result = await desktop.run(nodeCode, "node");
  console.log(`   Output: ${result.stdout}`);

  // Example 6: Cross-platform compatible
  console.log("\n6. Cross-platform command:");
  result = await desktop.run("echo 'This works on any platform!'");
  console.log(`   Output: ${result.stdout}`);

  // Example 7: Error handling
  console.log("\n7. Error handling:");
  try {
    result = await desktop.run("exit 1");
    console.log(`   Exit status: ${result.exit_status}`);
    if (result.exit_status !== 0) {
      console.log(`   Command failed with exit code: ${result.exit_status}`);
    }
  } catch (error) {
    console.error(`   Error: ${error.message}`);
  }

  // Backward compatibility - old syntax still works
  console.log("\n8. Backward compatibility (old syntax):");
  result = await desktop.runCommand(
    process.platform === "win32" ? "dir" : null, // windowsCommand
    process.platform !== "win32" ? "ls" : null // unixCommand
  );
  console.log(`   Output: ${result.stdout.substring(0, 100)}...`); // First 100 chars
}

// Run the examples
main().catch(console.error);
