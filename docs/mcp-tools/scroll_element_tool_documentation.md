# Scroll Element Tool Documentation

## Overview

The `scroll_element` tool scrolls UI elements in specified directions with optional visual highlighting for confirmation. Supports scrolling in all four directions with configurable amounts.

## Basic Usage

```javascript
{
  "tool_name": "scroll_element",
  "args": {
    "selector": "role:List|name:FileList",
    "direction": "down",
    "amount": 3
  }
}
```

## With Visual Highlighting

```javascript
{
  "tool_name": "scroll_element",
  "args": {
    "selector": "role:List|name:FileList",
    "direction": "down",
    "amount": 5,
    "highlight_before_action": {
      "enabled": true,
      "duration_ms": 1500,        // 1.5 seconds
      "color": 0xFF00FF,          // Purple (BGR format)
      "text": "SCROLL ↓",          // Custom overlay text with arrow
      "text_position": "Inside"   // Text placement
    }
  }
}
```

## Parameters

| Parameter                 | Type    | Required | Default | Description                                        |
| ------------------------- | ------- | -------- | ------- | -------------------------------------------------- |
| `selector`                | string  | Yes      | -       | Element selector (e.g., `role:List\|name:Files`)   |
| `direction`               | string  | Yes      | -       | Direction to scroll: 'up', 'down', 'left', 'right' |
| `amount`                  | number  | No       | 3       | Amount to scroll (lines or pages)                  |
| `alternative_selectors`   | string  | No       | -       | Alternative selectors to try in parallel           |
| `fallback_selectors`      | string  | No       | -       | Fallback selectors if primary fails                |
| `highlight_before_action` | object  | No       | -       | Visual highlighting configuration                  |
| `timeout_ms`              | number  | No       | 3000    | Timeout for finding element                        |
| `include_tree`            | boolean | No       | false   | Include UI tree in response                        |

## Scroll Directions

| Direction   | Description      | Common Use Cases                        |
| ----------- | ---------------- | --------------------------------------- |
| **`up`**    | Scroll upward    | Move to previous content, go to top     |
| **`down`**  | Scroll downward  | Move to next content, go to bottom      |
| **`left`**  | Scroll leftward  | Navigate wide content, previous columns |
| **`right`** | Scroll rightward | Navigate wide content, next columns     |

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

**List navigation with arrows:**

```javascript
{
  "tool_name": "scroll_element",
  "args": {
    "selector": "role:List",
    "direction": "down",
    "amount": 5,
    "highlight_before_action": {
      "enabled": true,
      "text": "↓",
      "color": 0x00FF00  // Green
    }
  }
}
```

**Page scrolling with custom highlighting:**

```javascript
{
  "tool_name": "scroll_element",
  "args": {
    "selector": "role:Document",
    "direction": "up",
    "amount": 10,
    "highlight_before_action": {
      "enabled": true,
      "duration_ms": 2000,
      "color": 0xFF0000,          // Blue border
      "text": "PAGE UP",
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

**Horizontal scrolling:**

```javascript
{
  "tool_name": "scroll_element",
  "args": {
    "selector": "role:Table",
    "direction": "right",
    "amount": 3,
    "highlight_before_action": {
      "enabled": true,
      "duration_ms": 1000,
      "color": 0x0080FF,          // Orange border
      "text": "SCROLL →"
    }
  }
}
```

**Large scroll with visual feedback:**

```javascript
{
  "tool_name": "scroll_element",
  "args": {
    "selector": "role:window|name:Browser",
    "direction": "down",
    "amount": 20,
    "highlight_before_action": {
      "enabled": true,
      "duration_ms": 2500,
      "color": 0xFFFF00,          // Cyan border
      "text": "BIG SCROLL",
      "text_position": "Bottom",
      "font_style": {
        "size": 18,
        "bold": true,
        "color": 0x000000         // Black text
      }
    }
  }
}
```

## Scroll Amount Guidelines

| Amount    | Description   | Use Cases                     |
| --------- | ------------- | ----------------------------- |
| **1-3**   | Small scroll  | Fine navigation, single items |
| **5-10**  | Medium scroll | Page sections, multiple items |
| **15-20** | Large scroll  | Full pages, major navigation  |
| **25+**   | Bulk scroll   | End of document, major jumps  |

## Color Recommendations by Direction

| Direction | Suggested Color | BGR Value  | Visual Cue        |
| --------- | --------------- | ---------- | ----------------- |
| **Up**    | Blue            | `0xFF0000` | ↑ Sky/upward      |
| **Down**  | Green           | `0x00FF00` | ↓ Ground/downward |
| **Left**  | Orange          | `0x0080FF` | ← Warning/back    |
| **Right** | Purple          | `0xFF00FF` | → Forward/next    |

## Benefits

- **Visual confirmation** of scroll target and direction
- **Debugging aid** for navigation automation workflows
- **Non-blocking** - maintains automation speed
- **Direction indicators** - clear visual cues for scroll direction
- **Amount feedback** - shows how much scrolling will occur
- **Flexible targeting** - works with any scrollable element

## Best Practices

- Use directional arrows (↑, ↓, ←, →) in text overlay for clarity
- Set duration to 1000-2000ms for manual verification during development
- Use different colors for different directions to build visual patterns
- Test scroll amounts with target applications before automation
- Use `"Inside"` text position for better visibility on scrollable areas
- Enable highlighting when developing complex navigation workflows

## Common Use Cases

- **List Navigation**: Scrolling through file lists, menus, options
- **Document Reading**: Page up/down in text documents, PDFs
- **Table Navigation**: Horizontal scrolling in wide spreadsheets
- **Web Browsing**: Page scrolling in browsers, content areas
- **Data Exploration**: Navigating large datasets, logs, reports

## Troubleshooting

- **Element not scrollable**: Ensure target element supports scrolling
- **Wrong direction**: Verify element's scroll capabilities (horizontal/vertical)
- **No visible effect**: Try larger amounts or check if element has content to scroll
- **Selector issues**: Use more specific selectors for nested scrollable areas
