# Press Key Tool Documentation

## Overview

The `press_key` tool sends key presses to UI elements with optional visual highlighting for confirmation. Supports complex key combinations and special keys.

## Basic Usage

```javascript
{
  "tool_name": "press_key",
  "args": {
    "selector": "role:Edit|name:Search",
    "key": "{Enter}"
  }
}
```

## With Visual Highlighting

```javascript
{
  "tool_name": "press_key",
  "args": {
    "selector": "role:Edit|name:Search",
    "key": "{Ctrl}a",
    "highlight_before_action": {
      "enabled": true,
      "duration_ms": 1500,        // 1.5 seconds
      "color": 0xFF0000,          // Bright blue (BGR format)
      "text": "CTRL+A",            // Custom overlay text
      "text_position": "Inside"   // Text placement
    }
  }
}
```

## Parameters

| Parameter                 | Type    | Required | Default | Description                                                         |
| ------------------------- | ------- | -------- | ------- | ------------------------------------------------------------------- |
| `selector`                | string  | Yes      | -       | Element selector (e.g., `role:Edit\|name:Search`)                   |
| `key`                     | string  | Yes      | -       | Key or key combination to press                                     |
| `alternative_selectors`   | string  | No       | -       | Alternative selectors to try in parallel                            |
| `fallback_selectors`      | string  | No       | -       | Fallback selectors if primary fails                                 |
| `highlight_before_action` | object  | No       | -       | Visual highlighting configuration                                   |
| `timeout_ms`              | number  | No       | 3000    | Timeout for finding element                                         |
| `include_tree`            | boolean | No       | true    | Include UI tree in response (currently always attached by the tool) |

## Key Format

Use curly brace notation for special keys and combinations:

| Key Type          | Format            | Examples                                    |
| ----------------- | ----------------- | ------------------------------------------- |
| **Single Keys**   | `{KeyName}`       | `{Enter}`, `{Tab}`, `{Escape}`, `{Space}`   |
| **Function Keys** | `{FN}`            | `{F1}`, `{F2}`, `{F12}`                     |
| **Navigation**    | `{KeyName}`       | `{Home}`, `{End}`, `{PageUp}`, `{PageDown}` |
| **Arrow Keys**    | `{Direction}`     | `{Up}`, `{Down}`, `{Left}`, `{Right}`       |
| **Modifiers**     | `{Modifier}key`   | `{Ctrl}c`, `{Alt}{F4}`, `{Shift}{Tab}`      |
| **Complex**       | `{Mod1}{Mod2}key` | `{Ctrl}{Shift}n`, `{Ctrl}{Alt}t`            |

## Highlighting Options

| Parameter       | Type    | Default  | Description                              |
| --------------- | ------- | -------- | ---------------------------------------- |
| `enabled`       | boolean | true     | Enable highlighting                      |
| `duration_ms`   | number  | 500      | Highlight duration in milliseconds       |
| `color`         | number  | 0x00FF00 | BGR color code (green)                   |
| `text`          | string  | -        | Overlay text (max 10 chars)              |
| `text_position` | enum    | "Top"    | Text position: Top, Inside, Bottom, etc. |
| `font_style`    | object  | -        | Font size, bold, color options           |

## Examples

**Form submission with confirmation:**

```javascript
{
  "tool_name": "press_key",
  "args": {
    "selector": "role:Edit|name:Username",
    "key": "{Enter}",
    "highlight_before_action": { "enabled": true }
  }
}
```

**Keyboard shortcuts with custom highlighting:**

```javascript
{
  "tool_name": "press_key",
  "args": {
    "selector": "role:window|name:Notepad",
    "key": "{Ctrl}s",
    "highlight_before_action": {
      "enabled": true,
      "duration_ms": 2000,
      "color": 0x00FFFF,          // Yellow border
      "text": "SAVE",
      "text_position": "Top",
      "font_style": {
        "size": 14,
        "bold": true,
        "color": 0x000000         // Black text
      }
    }
  }
}
```

**Navigation keys:**

```javascript
{
  "tool_name": "press_key",
  "args": {
    "selector": "role:List",
    "key": "{Down}",
    "highlight_before_action": {
      "enabled": true,
      "duration_ms": 1000,
      "color": 0xFF0000,          // Blue border
      "text": "↓"
    }
  }
}
```

**Complex key combinations:**

```javascript
{
  "tool_name": "press_key",
  "args": {
    "selector": "role:Document",
    "key": "{Ctrl}{Shift}{End}",
    "highlight_before_action": {
      "enabled": true,
      "duration_ms": 1500,
      "color": 0x0000FF,          // Red border
      "text": "SELECT ALL"
    }
  }
}
```

## Common Key Combinations

| Action         | Key Combination | Description                 |
| -------------- | --------------- | --------------------------- |
| **Copy**       | `{Ctrl}c`       | Copy selected content       |
| **Paste**      | `{Ctrl}v`       | Paste from clipboard        |
| **Select All** | `{Ctrl}a`       | Select all content          |
| **Save**       | `{Ctrl}s`       | Save current document       |
| **Undo**       | `{Ctrl}z`       | Undo last action            |
| **Find**       | `{Ctrl}f`       | Open find dialog            |
| **Close**      | `{Alt}{F4}`     | Close application           |
| **Switch App** | `{Alt}{Tab}`    | Switch between applications |
| **New Tab**    | `{Ctrl}t`       | Open new tab (browsers)     |
| **Refresh**    | `{F5}`          | Refresh page/content        |

## Benefits

- **Visual confirmation** of target elements before key input
- **Debugging aid** for keyboard automation workflows
- **Non-blocking** - maintains automation speed
- **Flexible key support** - handles simple keys to complex combinations
- **Robust targeting** - works with any focusable UI element

## Best Practices

- Use bright colors (blue: `0xFF0000`, yellow: `0x00FFFF`) for key action visibility
- Set duration to 1000-2000ms for manual verification during development
- Use descriptive text overlay for complex key combinations
- Test key combinations in target applications before automation
- Use `"Inside"` text position for better visibility on input elements
- Enable highlighting when developing complex keyboard workflows

## Common Use Cases

- **Form Navigation**: Tab, Enter, Escape for form completion
- **Text Editing**: Ctrl+A, Ctrl+C, Ctrl+V for text manipulation
- **Application Control**: Alt+F4, Ctrl+S, Ctrl+N for app commands
- **Navigation**: Arrow keys, Page Up/Down for content browsing
- **Shortcuts**: Function keys, custom hotkeys for specific actions
