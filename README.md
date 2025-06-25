# terminator 🤖

https://github.com/user-attachments/assets/00329105-8875-48cb-8970-a62a85a9ebd0

<p style="text-align: center;">
    <a href="https://discord.gg/dU9EBuw7Uq">
        <img src="https://img.shields.io/discord/823813159592001537?color=5865F2&logo=discord&logoColor=white&style=flat-square" alt="Join us on Discord">
    </a>
    <a href="https://docs.screenpi.pe/terminator/introduction">
        <img src="https://img.shields.io/badge/read_the-docs-blue" alt="docs">
    </a>
    <a href="https://www.youtube.com/@mediar_ai">
        <img src="https://img.shields.io/badge/YouTube-@mediar__ai-FF0000?logo=youtube&logoColor=white&style=flat-square" alt="YouTube @mediar_ai">
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
Terminator is the Playwright-style SDK for automating Windows GUI apps.

- 🪟 Built for Windows, works on Linux & macOS (partial)
- 🤖 Uses RLHF'd human screen recording as context
- 🧠 Designed for AI agents, not humans
- ⚡ Uses OS-level accessibility (not vision)
- 🧩 TS, Python, and Rust support
- 📈 80ms UI scans, 10000x faster and cheaper than humans

## Documentation

For detailed information on features, installation, usage, and the API, please visit the **[Official Documentation](https://docs.screenpi.pe/terminator/introduction)**.

Here's a section you can add under your `README.md` to document tools for inspecting accessibility elements across Windows, macOS, and Linux — tailored to Terminator users trying to find correct selectors:

---

## 🕵️ How to Inspect Accessibility Elements (like `name:Seven`)

To create reliable selectors (e.g. `name:Seven`, `role:Button`, `window:Calculator`), you need to inspect the Accessibility Tree of your OS. Here’s how to explore UI elements on each platform:

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
