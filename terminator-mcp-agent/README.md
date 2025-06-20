# Terminator MCP Agent

This directory contains the Model Context Protocol (MCP) agent that allows AI assistants, like Cursor, Claude desktop, VS Code, VS Code Insiders, and Windsurf to interact with your desktop using the Terminator UI automation library.

> **NPM Package Usage**
>
> This directory is also an npm package! You can install and run the MCP agent via Node.js:
>
> ```sh
> npx terminator-mcp-agent --start
> ```
>
> The correct binary for your platform will be installed automatically. Supported: Windows (x64), Linux (x64), macOS (x64, arm64).

<img width="1512" alt="Screenshot 2025-04-16 at 9 29 42 AM" src="https://github.com/user-attachments/assets/457ebaf2-640c-4f21-a236-fcb2b92748ab" />

MCP is useful to test out the `terminator` lib and see what you can do. You can use any model.

<br>

## Quick Setup for Desktop Apps

To configure Terminator MCP for use with supported desktop apps (Cursor, Claude, VS Code, VS Code Insiders, Windsurf), use the following CLI commands:

### 1. Configure for an App

Run this command in your terminal:

```sh
npx terminator-mcp-agent --add-to-app [app]
```

Replace `[app]` with one of:
- `cursor`
- `claude`
- `vscode`
- `insiders`
- `windsurf`

If you omit the app name, you will be prompted interactively to select one.

**Examples:**
```sh
npx terminator-mcp-agent --add-to-app cursor
npx terminator-mcp-agent --add-to-app
```

### 2. Start the MCP Agent

To start the agent (normally handled by your app):

```sh
npx terminator-mcp-agent --start
```

Or simply:

```sh
npx terminator-mcp-agent
```

---

## How it Works

- The CLI will automatically detect your platform and install the correct binary.
- The `--add-to-app` command will update the appropriate configuration file for your selected app, so it knows how to launch the MCP agent using `npx terminator-mcp-agent`.
- No PowerShell or manual JSON editing is required.

---

## Supported Apps & Config Locations

- **Cursor:** `%USERPROFILE%\.cursor\mcp.json` (Windows) or `~/.cursor/mcp.json` (macOS/Linux)
- **Claude:** `%APPDATA%\Claude\claude_desktop_config.json`
- **VS Code:** Registered via the `code` CLI
- **VS Code Insiders:** Registered via the `code-insiders` CLI
- **Windsurf:** `%USERPROFILE%\.codeium\windsurf\mcp_config.json`

---

## Development

If you want to build and test the agent locally, clone the repo and run:

```sh
git clone https://github.com/mediar-ai/terminator
cd terminator/terminator-mcp-agent
npm install
npm run build
npm install --global .
```

You can then use the CLI as above.

---

## Troubleshooting

- Make sure you have Node.js installed (v16+ recommended).
- For VS Code/Insiders, ensure the CLI (`code` or `code-insiders`) is available in your PATH.
- If you encounter issues, try running with elevated permissions or check the config file paths above.

---

## License

MIT


