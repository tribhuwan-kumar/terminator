#!/usr/bin/env node

const config = JSON.stringify({
  name: "terminator-mcp-agent",
  command: "npx",
  args: ["-y", "terminator-mcp-agent"],
});

// VS Code
const urlForVSCode = `vscode:mcp/install?${encodeURIComponent(config)}`;
const urlForVSCodeGithub = `https://insiders.vscode.dev/redirect?url=${encodeURIComponent(urlForVSCode)}`;

// VS Code Insiders
const urlForVSCodeInsiders = `vscode-insiders:mcp/install?${encodeURIComponent(config)}`;
const urlForVSCodeInsidersGithub = `https://insiders.vscode.dev/redirect?url=${encodeURIComponent(urlForVSCodeInsiders)}`;

console.log("VS Code Install Link:");
console.log(urlForVSCodeGithub);
console.log("\nVS Code Insiders Install Link:");
console.log(urlForVSCodeInsidersGithub);
