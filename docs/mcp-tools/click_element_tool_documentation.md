# Click Element Tool Documentation

## Overview

The `click_element` tool clicks UI elements with optional visual highlighting for confirmation.

## Basic Usage

```javascript
{
  "tool_name": "click_element",
  "args": {
    "selector": "role:Button|name:Submit"
  }
}
```

## With Visual Highlighting

```javascript
{
  "tool_name": "click_element",
  "args": {
    "selector": "role:Button|name:Submit",
    "highlight_before_action": {
      "enabled": true,
      "duration_ms": 1500,        // 1.5 seconds
      "color": 0x00FF00,          // Bright green (BGR format)
      "text": "CLICKING",          // Custom overlay text
      "text_position": "Inside"   // Text placement
    }
  }
}
```

## Parameters

| Parameter                 | Type    | Required | Default | Description                                         |
| ------------------------- | ------- | -------- | ------- | --------------------------------------------------- |
| `selector`                | string  | Yes      | -       | Element selector (e.g., `role:Button\|name:Submit`) |
| `alternative_selectors`   | string  | No       | -       | Alternative selectors to try in parallel            |
| `fallback_selectors`      | string  | No       | -       | Fallback selectors if primary fails                 |
| `highlight_before_action` | object  | No       | -       | Visual highlighting configuration                   |
| `timeout_ms`              | number  | No       | 3000    | Timeout for finding element                         |
| `include_tree`            | boolean | No       | false   | Include UI tree in response                         |

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

**Quick visual confirmation:**

```javascript
{
  "tool_name": "click_element",
  "args": {
    "selector": "#submit-btn",
    "highlight_before_action": { "enabled": true }
  }
}
```

**Custom highlighting:**

```javascript
{
  "tool_name": "click_element",
  "args": {
    "selector": "role:Button|name:Cancel",
    "highlight_before_action": {
      "enabled": true,
      "duration_ms": 2000,
      "color": 0x0000FF,          // Red border
      "text": "CANCEL",
      "text_position": "Top",
      "font_style": {
        "size": 16,
        "bold": true,
        "color": 0xFFFFFF         // White text
      }
    }
  }
}
```

## Benefits

- **Visual confirmation** of click targets
- **Debugging aid** for automation workflows
- **Non-blocking** - maintains automation speed
- **Customizable** appearance and timing
- **Backward compatible** - existing calls work unchanged

## Best Practices

- Use bright colors (green: `0x00FF00`, yellow: `0x00FFFF`) for visibility
- Set duration to 1000-2000ms for manual verification
- Use `"Inside"` text position for better visibility on buttons
- Enable highlighting during development and testing
