SSE Events
==========

- Endpoint: `/events` (outside `/mcp`)
- Content-Type: `text/event-stream`
- Event types:
  - `sequence` with `phase: start|end`
  - `sequence_progress`
  - `sequence_step` with `phase: begin|end`

Each event payload is a JSON string with at least:

```
{
  "request_id": "<uuid>",
  "timestamp": "<ISO-8601>"
}
```

Client example (Node.js):

```js
import EventSource from 'eventsource';
const es = new EventSource('http://127.0.0.1:3000/events');
es.onmessage = (e) => console.log('event', e.data);
es.onerror = (e) => console.error('error', e);
```

