use once_cell::sync::OnceCell;
use tokio::sync::broadcast;

/// Lightweight global event bus for streaming workflow execution events over HTTP
///
/// Events are arbitrary JSON values. Producers call `publish` and subscribers
/// (e.g., the `/events` SSE endpoint) call `subscribe` to receive a stream.
static EVENT_SENDER: OnceCell<broadcast::Sender<serde_json::Value>> = OnceCell::new();

fn sender() -> broadcast::Sender<serde_json::Value> {
    EVENT_SENDER
        .get_or_init(|| {
            let (tx, _rx) = broadcast::channel(256);
            tx
        })
        .clone()
}

/// Publish an event to all subscribers. Errors (e.g., no subscribers) are ignored.
pub fn publish(event: serde_json::Value) {
    let _ = sender().send(event);
}

/// Subscribe to the global event stream.
pub fn subscribe() -> broadcast::Receiver<serde_json::Value> {
    sender().subscribe()
}

