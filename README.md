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

## Explore Further

- https://github.com/mediar-ai/terminator-typescript-examples
- https://github.com/mediar-ai/terminator-python-examples
- https://github.com/mediar-ai/terminator/examples

## contributing

contributions are welcome! please feel free to submit issues and pull requests. many parts are experimental, and help is appreciated. join our [discord](https://discord.gg/dU9EBuw7Uq) to discuss.

## businesses 

if you want desktop automation at scale for your business, [let's talk](https://mediar.ai)
