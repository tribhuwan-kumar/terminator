# JavaScript Workflow Automation Guide

This document explains how to author and execute JavaScript-based automation workflows through the **`run_javascript`** tool that is now part of Terminator MCP agent.

---

## 1. Overview

`run_javascript` embeds a sandboxed JavaScript engine (QuickJS) inside the MCP. A small helper library (`terminator.js`) is injected automatically giving your scripts high-level access to UI automation tools as well as quality-of-life utilities. Two authoring modes are supported:

1. **Raw script** – pass a single JavaScript snippet.
2. **Workflow** – pass a GitHub-Actions-style YAML / JSON file describing multiple sequential steps.

### Raw script example

```jsonc
{
  "tool_name": "run_javascript",
  "arguments": {
    "script": "click('button|Login');"
  }
}
```

### Workflow example

```yaml
# workflows/login.yaml
name: Sample Login WF
env:
  URL: "https://example.com/login"
steps:
  - name: Open browser and navigate
    run: |
      typeText('name:Browser|AddressBar', ENV.URL);
      pressKey('name:Browser|AddressBar', '{Enter}');
      wait(3000);
  - name: Submit credential form
    env:
      USER: "demo"
      PASS: "password"
    run: |
      typeText('textbox|Username', ENV.USER);
      typeText('textbox|Password', ENV.PASS);
      click('button|Submit');
```

`workflow_yaml` holds the YAML (or JSON) text when calling the tool:

```jsonc
{
  "tool_name": "run_javascript",
  "arguments": {
    "workflow_yaml": "$(cat workflows/login.yaml)"
  }
}
```

---

## 2. terminator.js Helpers

| Helper | Description |
|--------|-------------|
| `click(selector)` | Click UI element |
| `typeText(selector, text)` | Type text using clipboard optimisation |
| `pressKey(selector, key)` | Send key press (`{Enter}` etc.) |
| `delay(ms)` / `wait(ms)` | Pause execution |
| `waitFor(selector, condition, timeout)` | Wait for element condition (`exists`, `visible` …) |
| `clickAndWaitDisappear(selector, timeout)` | Convenience combo |
| `getEnv(name, default)` | Read env variable defined in workflow/step |
| `runCommand(cmd)` | Run OS shell command (cross-platform) |
| `call(tool, args)` | Call any MCP tool directly |

All helpers return the parsed JSON result from the underlying tool so you can inspect fields easily.

#### Example looping over items

```js
for (let i = 0; i < 5; i++) {
  click(`#row_${i}`);
  wait(300);
}
```

---

## 3. Workflow Syntax Reference

### Workflow file

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | no | Human-readable label |
| `env` | map<string,string> | no | Environment variables inherited by all steps |
| `steps` | array<Step> | **yes** | Ordered list of steps |

### Step object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | no | Step label |
| `run` | string | **yes** | Multiline JS executed by QuickJS |
| `env` | map<string,string> | no | Step-specific env overrides |

---

## 4. Advanced Usage

### Direct tool access

```js
const apps = call('get_applications', {});
console.log(`There are ${apps.applications.length} apps running`);
```

### Shell integration

```js
runCommand('echo "Hello from JS"');
```

### Error handling

```js
try {
  click('button|Save');
} catch (e) {
  console.log('Save failed: ' + e);
}
```

---

## 5. Embedding inside `execute_sequence`

`run_javascript` can be used as a step inside larger multi-tool sequences:

```jsonc
{
  "tool_name": "execute_sequence",
  "arguments": {
    "steps": [
      {
        "tool_name": "run_javascript",
        "arguments": {
          "workflow_yaml": "steps:\n  - run: click('#ok')\n"
        }
      },
      { "tool_name": "delay", "arguments": { "delay_ms": 1000 } }
    ]
  }
}
```

---

## 6. Limitations & Roadmap

* Currently powered by QuickJS – no built-in Node/Bun/Deno APIs. If you need file I/O use `runCommand`.
* Scripts execute synchronously – avoid infinite loops or long-blocking operations.
* Future improvements: per-step timeout, artefact upload, parallel matrix support, Node runtime backend.

Happy automating! 🎉