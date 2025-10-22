# terminator-recorder-server

HTTP/WebSocket server for workflow recording with visual highlighting support.

## Overview

This server provides a simple REST API and WebSocket streaming for recording user interactions with optional real-time visual highlighting. It replaces the MCP-based `record_workflow` tool with a more direct server-to-server communication approach.

## Features

- **HTTP REST API** for starting/stopping recordings
- **WebSocket streaming** for real-time event delivery
- **Visual highlighting** with configurable colors, labels, and durations
- **Flexible configuration** via request parameters or defaults
- **CORS support** for web-based clients
- **Standalone server** - no MCP protocol overhead

## Building

```bash
cargo build --release
```

The binary will be at `target/release/terminator-recorder-server` (or `.exe` on Windows).

## Running

```bash
# Default (port 8082)
./target/release/terminator-recorder-server

# Custom port
./target/release/terminator-recorder-server --port 8090

# With CORS enabled
./target/release/terminator-recorder-server --cors
```

## API Reference

### Health Check

```http
GET /api/health
```

**Response:**
```json
{
  "status": "ok",
  "version": "0.10.6"
}
```

### Start Recording

```http
POST /api/recording/start
Content-Type: application/json

{
  "workflow_name": "My Workflow",
  "config": {
    "performance_mode": "Normal",
    "record_mouse": true,
    "record_keyboard": true
  },
  "highlighting": {
    "enabled": true,
    "color": 255,
    "duration_ms": 2000,
    "show_labels": true,
    "label_position": "Top",
    "label_style": {
      "size": 14,
      "bold": true,
      "color": 16777215
    }
  }
}
```

**Response:**
```json
{
  "status": "started",
  "session_id": "uuid-here",
  "websocket_url": "ws://127.0.0.1:8082/api/recording/events?session=uuid-here",
  "highlighting": {
    "enabled": true,
    "task_started": true
  }
}
```

### Stop Recording

```http
POST /api/recording/stop
Content-Type: application/json

{
  "session_id": "uuid-here"
}
```

**Response:**
```json
{
  "status": "stopped",
  "session_id": "uuid-here",
  "event_count": 42
}
```

### Get Status

```http
GET /api/recording/status
```

**Response (when recording):**
```json
{
  "status": "recording",
  "session_id": "uuid-here",
  "workflow_name": "My Workflow"
}
```

**Response (when idle):**
```json
{
  "status": "idle"
}
```

### WebSocket Event Stream

```
ws://127.0.0.1:8082/api/recording/events?session=uuid-here
```

**Event Message:**
```json
{
  "type": "event",
  "session_id": "uuid-here",
  "event": { /* WorkflowEvent JSON */ },
  "timestamp": 1234567890
}
```

**Status Message:**
```json
{
  "type": "status",
  "session_id": "uuid-here",
  "status": "recording",
  "event_count": 10
}
```

## Configuration Options

### Recorder Config

- `performance_mode`: "Normal" | "Balanced" | "LowEnergy"
- `record_mouse`: boolean (default: true)
- `record_keyboard`: boolean (default: true)
- `record_text_input_completion`: boolean (default: true)
- `record_clipboard`: boolean (default: true)
- `record_hotkeys`: boolean (default: true)

### Highlighting Config

- `enabled`: boolean (required)
- `color`: u32 in BGR format (default: 0x0000FF = red)
- `duration_ms`: u64 (default: 2000)
- `show_labels`: boolean (default: true)
- `label_position`: "Top" | "TopRight" | "Right" | "BottomRight" | "Bottom" | "BottomLeft" | "Left" | "TopLeft" | "Inside"
- `label_style`:
  - `size`: u32 (default: 14)
  - `bold`: boolean (default: true)
  - `color`: u32 in BGR format (default: 0xFFFFFF = white)

## Event Labels

When `show_labels` is true, events are highlighted with descriptive labels:

- Click → "CLICK"
- Text Input → "TYPE"
- Keyboard → "KEY: {keycode}"
- Drag/Drop → "DRAG"
- App Switch → "SWITCH"
- Tab Navigation → "TAB"
- Right Click → "RCLICK"
- Middle Click → "MCLICK"

## Architecture

```
Client (mediar-app)
    ↓ HTTP POST /api/recording/start
terminator-recorder-server
    ├─→ RecorderManager
    │   ├─→ WorkflowRecorder (captures events)
    │   └─→ EventHighlighter (highlights UI elements)
    │
    ├─→ WebSocket (streams events)
    └─→ HTTP API (control endpoints)
```

## License

Same as parent terminator project.
