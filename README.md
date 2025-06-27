# terminator 🤖

https://github.com/user-attachments/assets/00329105-8875-48cb-8970-a62a85a9ebd0

<p align="center">
  <a href="https://discord.gg/dU9EBuw7Uq">
    <img src="https://img.shields.io/discord/823813159592001537?color=5865F2&logo=discord&logoColor=white&style=flat-square" alt="Join us on Discord">
  </a>
  <a href="https://docs.screenpi.pe/terminator/introduction">
    <img src="https://img.shields.io/badge/read_the-docs-blue" alt="Docs">
  </a>
  <a href="https://www.youtube.com/@mediar_ai">
    <img src="https://img.shields.io/badge/YouTube-@mediar__ai-FF0000?logo=youtube&logoColor=white&style=flat-square" alt="YouTube @mediar_ai">
  </a>
  <a href="https://crates.io/crates/terminator-rs">
    <img src="https://img.shields.io/crates/v/terminator-rs.svg" alt="Crates.io - terminator-rs">
  </a>
  <a href="https://crates.io/crates/terminator-workflow-recorder">
    <img src="https://img.shields.io/crates/v/terminator-workflow-recorder.svg" alt="Crates.io - workflow recorder">
  </a>
</p>

<p align="center">
  <a href="https://insiders.vscode.dev/redirect?url=vscode%3Amcp%2Finstall%3F%257B%2522terminator-mcp-agent%2522%253A%257B%2522command%2522%253A%2522npx%2522%252C%2522args%2522%253A%255B%2522-y%2522%252C%2522terminator-mcp-agent%2522%255D%257D%257D">
    <img alt="Install in VS Code" src="https://img.shields.io/badge/VS_Code-VS_Code?style=flat-square&label=Install%20MCP&color=0098FF">
  </a>
  <a href="https://insiders.vscode.dev/redirect?url=vscode-insiders%3Amcp%2Finstall%3F%257B%2522terminator-mcp-agent%2522%253A%257B%2522command%2522%253A%2522npx%2522%252C%2522args%2522%253A%255B%2522-y%2522%252C%2522terminator-mcp-agent%2522%255D%257D%257D">
    <img alt="Install in VS Code Insiders" src="https://img.shields.io/badge/VS_Code_Insiders-VS_Code_Insiders?style=flat-square&label=Install%20MCP&color=24bfa5">
  </a>
  <a href="https://cursor.com/install-mcp?name=terminator-mcp-agent&config=eyJjb21tYW5kIjoibnB4IiwiYXJncyI6WyIteSIsInRlcm1pbmF0b3ItbWNwLWFnZW50Il19">
    <img alt="Install in Cursor" src="https://img.shields.io/badge/Cursor-Cursor?style=flat-square&label=Install%20MCP&color=22272e">
  </a>
</p>


>Computer use SDK for building agents that learn from human screen recordings. Cross-platform (Windows/macOS/Linux), deterministic, and ready for L5 desktop automation.

## ⚡ TL;DR — Hello World Example

> Skip the boilerplate. This is the fastest way to feel the magic.

### 🐍 Python

```bash
pip install terminator.py
```

```python
import terminator
desktop = terminator.Desktop()
desktop.open_application('calc')
seven = desktop.locator('name:Seven')
seven.click()
```

### 🟦 TypeScript / Node.js

```bash
bun i terminator.js # or npm, pnpm, yarn
```

```ts
const { Desktop } = require('terminator.js');
const desktop = new Desktop();
await desktop.openApplication('notepad')
await desktop.locator('name:Edit').typeText('hello world')
```

### 🧠 What is Terminator?

Terminator is an AI-first Playwright-style SDK for automating operating systems.

- 🪟 Built for Windows, with partial support on Linux and macOS
- 🤖 Learns deterministically from screen recordings of real workflows
- 🧠 Designed for AI agents—not humans
- ⚡ Uses OS-level accessibility APIs, with OCR/Vision as fallback
- 🧩 Supports TypeScript, Python, MCP, and Rust
- 📈 Scans the UI in ~80ms—up to 10,000x faster and cheaper than a human

Terminator runs “headless” by default. It doesn’t require a visible screen, relying instead on accessibility layers (like UI Automation on Windows) to interact with apps.

## Feature Support

While Terminator aims for full cross-platform support, current capabilities vary by OS. Windows is the primary development target and has the most complete feature set.

| Feature                  | Windows | macOS | Linux | Notes                                        |
| ------------------------ | :-----: | :---: | :---: | -------------------------------------------- |
| **Core Automation**      |         |       |       |                                              |
| Element Locators         |    ✅   |  🟡   |  🟡   | Find elements by `name`, `role`, `window`, etc. |
| UI Actions (`click`, `type`) |    ✅   |  🟡   |  🟡   | Core interactions with UI elements.          |
| Application Management   |    ✅   |  🟡   |  🟡   | Launch, list, and manage applications. |
| Window Management        |    ✅   |  🟡   |  🟡   | Get active window, list windows.             |
| **Advanced Features**    |         |       |       |                                              |
| Workflow Recording       |    ✅   |  ❌   |  ❌   | Record human workflows for deterministic automation.     |
| Monitor Management       |    ✅   |  🟡   |  🟡   | Multi-display support.                       |
| Screen & Element Capture |    ✅   |  ✅   |  🟡   | Take screenshots of displays or elements.     |
| **Language Bindings**    |         |       |       |                                              |
| Python (`terminator.py`) |    ✅   |  ✅   |  ✅   | `pip install terminator.py`                  |
| TypeScript (`terminator.js`) |    ✅   |  ✅   |  ✅   | `npm i terminator.js`                        |
| MCP (`terminator-mcp-agent`) |    ✅   |  ✅   |  ✅   | `npx -y terminator-mcp-agent --add-to-app [app]`                        |
| Rust (`terminator-rs`) |    ✅   |  ✅   |  ✅   | `cargo add terminator-rs`                        |

**Legend:**
- ✅: **Supported** - The feature is stable and well-tested.
- 🟡: **Partial / Experimental** - The feature is in development and may have limitations.
- ❌: **Not Supported** - The feature is not yet available on this platform.

## Documentation

For detailed information on features, installation, usage, and the API, please visit the **[Official Documentation](https://docs.screenpi.pe/terminator/introduction)**.

Here's a section you can add under your `README.md` to document tools for inspecting accessibility elements across Windows, macOS, and Linux — tailored to Terminator users trying to find correct selectors:

---

## 🕵️ How to Inspect Accessibility Elements (like `name:Seven`)

To create reliable selectors (e.g. `name:Seven`, `role:Button`, `window:Calculator`), you need to inspect the Accessibility Tree of your OS. Here's how to explore UI elements on each platform:

### 🪟 Windows

* **Tool:** [Accessibility Insights for Windows](https://accessibilityinsights.io/downloads/)
* **Alt:** [Inspect.exe](https://learn.microsoft.com/en-us/windows/win32/winauto/inspect-objects) (comes with Windows SDK)
* **Usage:** Open the app you want to inspect → launch Accessibility Insights → hover or use keyboard navigation to explore the UI tree (Name, Role, ControlType, AutomationId).

> These tools show you the `Name`, `Role`, `ControlType`, and other metadata used in Terminator selectors.

---

### 🍎 macOS

* **Tool:** [Accessibility Inspector](https://developer.apple.com/documentation/xcode/accessibility_inspector)
* **Usage:** Comes with Xcode → Open `Xcode > Open Developer Tool > Accessibility Inspector` → Use the target icon to explore UI elements on screen.

---

### 🐧 Linux

* **Tool:** [Accerciser](https://wiki.gnome.org/Apps/Accerciser)
* **Install:**

  ```bash
  sudo apt install accerciser
  ```
* **Usage:** Launch Accerciser → Select the window/app → Browse the accessible widget tree.

---

### 💡 Tip

Once you identify the structure of your UI:

```python
# Sample pattern
desktop.locator('window:Calculator')
       .locator('role:Button')
       .locator('name:Seven')
```

You can build and debug selector paths incrementally using `.locator()` chaining.

## Explore Further

- https://github.com/mediar-ai/terminator-typescript-examples
- https://github.com/mediar-ai/terminator-python-examples
- https://github.com/mediar-ai/terminator/examples

## contributing

contributions are welcome! please feel free to submit issues and pull requests. many parts are experimental, and help is appreciated. join our [discord](https://discord.gg/dU9EBuw7Uq) to discuss.

## businesses 

if you want desktop automation at scale for your business, [let's talk](https://mediar.ai)
