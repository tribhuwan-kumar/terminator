# Stop Highlighting Tool Documentation

## Overview

`stop_highlighting` immediately stops active visual highlight overlays that were started by tools like `highlight_element` or by recorder-driven highlighting. It returns the number of highlights that were stopped.

> Note: This tool is designed as a lightweight, fast interrupt. Use it to end long-running or persistent highlights early.

## Tool: `stop_highlighting`

### Parameters

| Parameter      | Type   | Required | Default | Description                                                            |
| -------------- | ------ | -------- | ------- | ---------------------------------------------------------------------- |
| `highlight_id` | string | No       | `null`  | Optional specific highlight identifier to stop. If omitted, stops all. |

- Current behavior: If `highlight_id` is omitted, the tool stops all tracked highlights. The `highlight_id` parameter is reserved for future versions that support individually addressable highlights.

### Return

On success, the tool returns a JSON object with the count of overlays that were stopped.

```json
{
  "action": "stop_highlighting",
  "status": "success",
  "highlights_stopped": 1,
  "timestamp": "2025-08-19T00:06:10.782011600+00:00"
}
```

### Usage Examples

#### Basic: Stop all active highlights

```javascript
{
  "tool_name": "stop_highlighting",
  "args": {}
}
```

#### Future-compatible: Stop a specific highlight (when IDs are supported)

```javascript
{
  "tool_name": "stop_highlighting",
  "args": { "highlight_id": "abc123" }
}
```

#### Script snippet (Node SDK)

```javascript
// Start a 30s highlight
await client.callTool("highlight_element", {
  selector: "Window|Untitled - Google Chrome",
  color: 0x00ff00,
  duration_ms: 30000,
  include_tree: false,
});

// Wait 3 seconds, then stop early
await new Promise((r) => setTimeout(r, 3000));
const result = await client.callTool("stop_highlighting", {});
console.log(
  "highlights_stopped =",
  JSON.parse(result.content[0].text).highlights_stopped
);
```

## Best Practices

- Prefer `include_tree: false` on `highlight_element` for low-latency responses when you plan to stop early.
- Use short element find timeouts for `highlight_element` (e.g., `timeout_ms ~ 1000`) to avoid long pre-highlight delays.
- If the overlay has already expired (duration elapsed), `highlights_stopped` may be `0`.

## Troubleshooting

- If nothing stops:
  - The overlay may have already finished. Try increasing `duration_ms` during testing.
  - Ensure the MCP server is tracking highlights (you should see `OVERLAY_THREAD_START`/`OVERLAY_THREAD_DONE` logs).
- Logs to watch:
  - `terminator::platforms::windows::highlighting: OVERLAY_THREAD_START`
  - `terminator::platforms::windows::highlighting: OVERLAY_THREAD_DONE`

## Related Tools

- `highlight_element`: Draws a highlight around a target element with optional text overlay. For faster responses, set `include_tree: false`. You can also enable a detailed element payload by setting `include_element_info: true`.
- `record_workflow`: When visual highlighting is enabled during recording, this tool can stop recorder-driven highlights as well.

## Notes

- Platform support: Fully supported on Windows. Other platforms may have limited overlay capabilities.
- Performance target: Stop requests are processed immediately; overlays should disappear within a few milliseconds once the stop request is received.
