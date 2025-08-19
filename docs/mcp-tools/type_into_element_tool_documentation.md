# Type Into Element Tool Documentation

## Overview

The `type_into_element` tool types text into UI elements with smart clipboard optimization and optional visual highlighting for confirmation.

## Basic Usage

```javascript
{
  "tool_name": "type_into_element",
  "args": {
    "selector": "role:Edit|name:Search",
    "text_to_type": "Hello World"
  }
}
```

## With Visual Highlighting

```javascript
{
  "tool_name": "type_into_element",
  "args": {
    "selector": "role:Edit|name:Search",
    "text_to_type": "Hello World",
    "highlight_before_action": {
      "enabled": true,
      "duration_ms": 1500,        // 1.5 seconds
      "color": 0x00FF00,          // Bright green (BGR format)
      "text": "TYPING",            // Custom overlay text
      "text_position": "Inside"   // Text placement
    }
  }
}
```

## Parameters

| Parameter                 | Type    | Required | Default | Description                                       |
| ------------------------- | ------- | -------- | ------- | ------------------------------------------------- |
| `selector`                | string  | Yes      | -       | Element selector (e.g., `role:Edit\|name:Search`) |
| `text_to_type`            | string  | Yes      | -       | Text content to type into the element             |
| `alternative_selectors`   | string  | No       | -       | Alternative selectors to try in parallel          |
| `fallback_selectors`      | string  | No       | -       | Fallback selectors if primary fails               |
| `clear_before_typing`     | boolean | No       | true    | Clear element before typing new text              |
| `verify_action`           | boolean | No       | true    | Verify text was typed successfully                |
| `highlight_before_action` | object  | No       | -       | Visual highlighting configuration                 |
| `timeout_ms`              | number  | No       | 3000    | Timeout for finding element                       |
| `include_tree`            | boolean | No       | false   | Include UI tree in response                       |

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

**Form filling with confirmation:**

```javascript
{
  "tool_name": "type_into_element",
  "args": {
    "selector": "role:Edit|name:Username",
    "text_to_type": "user@example.com",
    "highlight_before_action": { "enabled": true }
  }
}
```

**Search with custom highlighting:**

```javascript
{
  "tool_name": "type_into_element",
  "args": {
    "selector": "role:Edit|name:Search",
    "text_to_type": "automation testing",
    "clear_before_typing": true,
    "highlight_before_action": {
      "enabled": true,
      "duration_ms": 2000,
      "color": 0x00FFFF,          // Yellow border
      "text": "SEARCH",
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

**Append text without clearing:**

```javascript
{
  "tool_name": "type_into_element",
  "args": {
    "selector": "#message-input",
    "text_to_type": " - additional text",
    "clear_before_typing": false,
    "highlight_before_action": {
      "enabled": true,
      "duration_ms": 1000,
      "color": 0x0000FF,          // Red border
      "text": "APPEND"
    }
  }
}
```

## Smart Features

- **Clipboard Optimization**: Uses clipboard for fast text input on long strings
- **Auto-verification**: Confirms text was typed correctly (can be disabled)
- **Element Clearing**: Automatically clears existing content before typing
- **Retry Logic**: Built-in retry mechanism for flaky UI elements

## Benefits

- **Visual confirmation** of target input fields
- **Debugging aid** for form automation workflows
- **Non-blocking** - maintains automation speed
- **Smart input method** - much faster than simulated keystrokes
- **Robust verification** - ensures text was actually entered

## Best Practices

- Use bright colors (green: `0x00FF00`, blue: `0xFF0000`) for input field visibility
- Set duration to 1000-2000ms for manual verification during development
- Use `"Inside"` text position for better visibility on input fields
- Enable highlighting when developing complex form automation
- Use `clear_before_typing: false` when appending to existing content
- Disable verification (`verify_action: false`) for performance-critical scenarios

## Common Use Cases

- **Form Automation**: Username/password fields, contact forms
- **Search Operations**: Search boxes, filter inputs
- **Content Creation**: Text areas, message composers
- **Data Entry**: Spreadsheet cells, database forms
