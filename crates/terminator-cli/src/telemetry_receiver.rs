// Simple OTLP receiver for capturing workflow telemetry
use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};
use bytes::Bytes;
use colored::Colorize;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use prost::Message;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub struct TelemetryReceiver {
    port: u16,
}

impl TelemetryReceiver {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub async fn start(self) -> Result<JoinHandle<()>> {
        let steps_state = Arc::new(Mutex::new(StepsTracker::new()));

        let app = Router::new()
            .route("/v1/traces", post(handle_traces))
            .with_state(steps_state);

        let addr = format!("127.0.0.1:{}", self.port);

        let handle = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });

        // Give it a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(handle)
    }
}

struct StepsTracker {
    total_steps: Option<usize>,
    current_step: usize,
}

impl StepsTracker {
    fn new() -> Self {
        Self {
            total_steps: None,
            current_step: 0,
        }
    }
}

async fn handle_traces(
    State(steps): State<Arc<Mutex<StepsTracker>>>,
    body: Bytes,
) -> (StatusCode, Json<serde_json::Value>) {
    // Try to parse as protobuf first (most common)
    if let Ok(request) = ExportTraceServiceRequest::decode(&body[..]) {
        process_protobuf_traces(request, steps).await;
    } else if let Ok(json_data) = serde_json::from_slice::<serde_json::Value>(&body) {
        // Fallback to JSON parsing
        process_json_traces(json_data, steps).await;
    }

    (StatusCode::OK, Json(json!({"partialSuccess": {}})))
}

async fn process_json_traces(data: serde_json::Value, steps: Arc<Mutex<StepsTracker>>) {
    if let Some(resource_spans) = data.get("resourceSpans").and_then(|v| v.as_array()) {
        for resource_span in resource_spans {
            if let Some(scope_spans) = resource_span.get("scopeSpans").and_then(|v| v.as_array()) {
                for scope_span in scope_spans {
                    if let Some(spans_array) = scope_span.get("spans").and_then(|v| v.as_array()) {
                        for span in spans_array {
                            process_span(span, &steps).await;
                        }
                    }
                }
            }
        }
    }
}

async fn process_span(span: &serde_json::Value, tracker: &Arc<Mutex<StepsTracker>>) {
    let name = span.get("name").and_then(|v| v.as_str()).unwrap_or("");

    // Parse attributes
    let mut attributes = std::collections::HashMap::new();
    if let Some(attrs) = span.get("attributes").and_then(|v| v.as_array()) {
        for attr in attrs {
            if let (Some(key), Some(value)) =
                (attr.get("key").and_then(|v| v.as_str()), attr.get("value"))
            {
                let val_str = extract_attribute_value(value);
                attributes.insert(key.to_string(), val_str);
            }
        }
    }

    // Parse events (step starts/completes)
    if let Some(events_array) = span.get("events").and_then(|v| v.as_array()) {
        for event in events_array {
            if let Some(event_name) = event.get("name").and_then(|v| v.as_str()) {
                let mut event_attrs = std::collections::HashMap::new();
                if let Some(attrs) = event.get("attributes").and_then(|v| v.as_array()) {
                    for attr in attrs {
                        if let (Some(key), Some(value)) =
                            (attr.get("key").and_then(|v| v.as_str()), attr.get("value"))
                        {
                            let val_str = extract_attribute_value(value);
                            event_attrs.insert(key.to_string(), val_str);
                        }
                    }
                }

                // Display step progress
                match event_name {
                    "workflow.started" => {
                        if let Some(total) = event_attrs.get("workflow.total_steps") {
                            let mut tracker = tracker.lock().await;
                            tracker.total_steps = total.parse().ok();

                            println!(
                                "\n{} {} {}",
                                "🎯".cyan(),
                                "WORKFLOW STARTED:".bold().cyan(),
                                format!("{total} steps").dimmed()
                            );
                        }
                    }
                    "step.started" => {
                        if let Some(tool) = event_attrs.get("step.tool") {
                            let step_index = event_attrs
                                .get("step.index")
                                .and_then(|s| s.parse::<usize>().ok())
                                .unwrap_or(0);

                            let mut tracker = tracker.lock().await;
                            tracker.current_step = step_index + 1;
                            let total = tracker.total_steps.unwrap_or(0);

                            println!(
                                "  {} Step {}/{}: {} {}",
                                "▶".blue(),
                                tracker.current_step,
                                total,
                                tool.yellow(),
                                "[running...]".dimmed()
                            );
                        }
                    }
                    "step.completed" => {
                        if let Some(status) = event_attrs.get("step.status") {
                            let icon = if status == "success" {
                                "✓".green()
                            } else if status == "skipped" {
                                "⏭".yellow()
                            } else {
                                "✗".red()
                            };
                            println!("    {icon} Status: {status}");
                        }
                    }
                    "workflow.completed" => {
                        let had_errors = event_attrs
                            .get("workflow.had_errors")
                            .and_then(|s| s.parse::<bool>().ok())
                            .unwrap_or(false);

                        if had_errors {
                            println!("\n{} Workflow completed with errors", "⚠".yellow());
                        } else {
                            println!("\n{} Workflow completed successfully", "✅".green());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Handle span-level info
    if name.starts_with("workflow.") {
        if let Some(total) = attributes.get("workflow.total_steps") {
            let mut tracker = tracker.lock().await;
            tracker.total_steps = total.parse().ok();
        }
    } else if name.starts_with("step.") {
        // Step span started
        if let Some(tool) = attributes.get("tool.name") {
            let step_num = attributes
                .get("step.number")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            let step_total = attributes
                .get("step.total")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            println!(
                "  {} Step {}/{}: {} {}",
                "📍".green(),
                step_num,
                step_total,
                tool.yellow(),
                "[executing...]".dimmed()
            );
        }
    }
}

fn extract_attribute_value(value: &serde_json::Value) -> String {
    if let Some(s) = value.get("stringValue").and_then(|v| v.as_str()) {
        s.to_string()
    } else if let Some(i) = value.get("intValue").and_then(|v| v.as_i64()) {
        i.to_string()
    } else if let Some(f) = value.get("doubleValue").and_then(|v| v.as_f64()) {
        f.to_string()
    } else if let Some(b) = value.get("boolValue").and_then(|v| v.as_bool()) {
        b.to_string()
    } else {
        value.to_string()
    }
}

// Process protobuf traces
async fn process_protobuf_traces(
    request: ExportTraceServiceRequest,
    tracker: Arc<Mutex<StepsTracker>>,
) {
    for resource_span in request.resource_spans {
        for scope_span in resource_span.scope_spans {
            for span in scope_span.spans {
                let span_name = span.name.clone();

                // Process events in the span
                for event in &span.events {
                    let event_name = event.name.clone();
                    let mut event_attrs = std::collections::HashMap::new();

                    // Extract event attributes
                    for attr in &event.attributes {
                        let key = attr.key.clone();
                        let value = extract_proto_attr_value(&attr.value);
                        event_attrs.insert(key, value);
                    }

                    // Display step progress based on events
                    match event_name.as_str() {
                        "workflow.started" => {
                            if let Some(total) = event_attrs.get("workflow.total_steps") {
                                let mut t = tracker.lock().await;
                                t.total_steps = total.parse().ok();

                                println!(
                                    "\n{} {} {}",
                                    "🎯".cyan(),
                                    "WORKFLOW STARTED:".bold().cyan(),
                                    format!("{total} steps").dimmed()
                                );
                            }
                        }
                        "step.started" => {
                            if let Some(tool) = event_attrs.get("step.tool") {
                                let step_index = event_attrs
                                    .get("step.index")
                                    .and_then(|s| s.parse::<usize>().ok())
                                    .unwrap_or(0);

                                let mut t = tracker.lock().await;
                                t.current_step = step_index + 1;
                                let total = t.total_steps.unwrap_or(0);

                                println!(
                                    "  {} Step {}/{}: {} {}",
                                    "▶".blue(),
                                    t.current_step,
                                    total,
                                    tool.yellow(),
                                    "[running...]".dimmed()
                                );
                            }
                        }
                        "step.completed" => {
                            if let Some(status) = event_attrs.get("step.status") {
                                let icon = if status == "success" {
                                    "✓".green()
                                } else if status == "skipped" {
                                    "⏭".yellow()
                                } else {
                                    "✗".red()
                                };
                                println!("    {icon} Status: {status}");
                            }
                        }
                        "workflow.completed" => {
                            let had_errors = event_attrs
                                .get("workflow.had_errors")
                                .and_then(|s| s.parse::<bool>().ok())
                                .unwrap_or(false);

                            if had_errors {
                                println!("\n{} Workflow completed with errors", "⚠".yellow());
                            } else {
                                println!("\n{} Workflow completed successfully", "✅".green());
                            }
                        }
                        _ => {}
                    }
                }

                // Also check span-level attributes for step info
                if span_name.starts_with("step.") {
                    let mut span_attrs = std::collections::HashMap::new();
                    for attr in &span.attributes {
                        let key = attr.key.clone();
                        let value = extract_proto_attr_value(&attr.value);
                        span_attrs.insert(key, value);
                    }

                    if let Some(tool) = span_attrs.get("tool.name") {
                        let step_num = span_attrs
                            .get("step.number")
                            .and_then(|s| s.parse::<usize>().ok())
                            .unwrap_or(0);
                        let step_total = span_attrs
                            .get("step.total")
                            .and_then(|s| s.parse::<usize>().ok())
                            .unwrap_or(0);

                        println!(
                            "  {} Step {}/{}: {} {}",
                            "📍".green(),
                            step_num,
                            step_total,
                            tool.yellow(),
                            "[executing...]".dimmed()
                        );
                    }
                }
            }
        }
    }
}

// Extract value from protobuf attribute
fn extract_proto_attr_value(
    value: &Option<opentelemetry_proto::tonic::common::v1::AnyValue>,
) -> String {
    if let Some(val) = value {
        if let Some(v) = &val.value {
            match v {
                opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(s) => {
                    s.clone()
                }
                opentelemetry_proto::tonic::common::v1::any_value::Value::IntValue(i) => {
                    i.to_string()
                }
                opentelemetry_proto::tonic::common::v1::any_value::Value::DoubleValue(f) => {
                    f.to_string()
                }
                opentelemetry_proto::tonic::common::v1::any_value::Value::BoolValue(b) => {
                    b.to_string()
                }
                _ => String::new(),
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

// Start the telemetry receiver
pub async fn start_telemetry_receiver() -> Result<JoinHandle<()>> {
    let receiver = TelemetryReceiver::new(4318);
    receiver.start().await
}
