use boa_engine::{Context, JsValue, NativeFunction, Source};
use rmcp::Error as McpError;
use serde_json::json;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::{debug, error, info};

// Simple thread-safe queue for tool calls
type ToolCall = (String, serde_json::Value, String); // (tool_name, args, response_id)
type ToolQueue = Arc<Mutex<VecDeque<ToolCall>>>;
type ResponseMap = Arc<Mutex<std::collections::HashMap<String, String>>>;

static TOOL_QUEUE: OnceLock<ToolQueue> = OnceLock::new();
static RESPONSE_MAP: OnceLock<ResponseMap> = OnceLock::new();

/// Enhanced conversion function that can use JavaScript's JSON.stringify
fn boa_js_to_json_with_context(
    value: boa_engine::JsValue,
    context: &mut boa_engine::Context,
) -> serde_json::Value {
    use boa_engine::Source;

    // For primitive types, use direct conversion
    match value {
        boa_engine::JsValue::Null | boa_engine::JsValue::Undefined => serde_json::Value::Null,
        boa_engine::JsValue::Boolean(b) => json!(b),
        boa_engine::JsValue::Integer(i) => json!(i),
        boa_engine::JsValue::Rational(r) => json!(r),
        boa_engine::JsValue::String(s) => json!(s.to_std_string_escaped()),
        boa_engine::JsValue::Symbol(_) => json!("[Symbol]"),
        boa_engine::JsValue::BigInt(bi) => json!(bi.to_string()),
        boa_engine::JsValue::Object(_) => {
            // For objects, try using JSON.stringify
            // First, set the value to a global variable
            if context
                .global_object()
                .set(
                    boa_engine::JsString::from("__temp_value"),
                    value,
                    false,
                    context,
                )
                .is_ok()
            {
                // Then call JSON.stringify on it
                match context.eval(Source::from_bytes("JSON.stringify(__temp_value)")) {
                    Ok(stringified) => {
                        if let Some(json_str) = stringified.as_string() {
                            // Parse the JSON string back to a serde_json::Value
                            serde_json::from_str(&json_str.to_std_string_escaped()).unwrap_or_else(
                                |_| json!({"error": "Failed to parse JSON.stringify result"}),
                            )
                        } else {
                            json!({"error": "JSON.stringify did not return a string"})
                        }
                    }
                    Err(e) => {
                        json!({"error": format!("JSON.stringify failed: {}", e)})
                    }
                }
            } else {
                json!({"error": "Failed to set temporary value in context"})
            }
        }
    }
}

/// Execute JavaScript code with Boa engine and tool call support
pub async fn execute_javascript<F, Fut>(
    script: String,
    tool_dispatcher: F,
) -> Result<serde_json::Value, McpError>
where
    F: Fn(String, serde_json::Value) -> Fut + Send + 'static + Clone,
    Fut: std::future::Future<Output = Result<String, McpError>> + Send,
{
    let script_src = script.clone();

    // Spawn the JavaScript execution task
    let js_handle = tokio::task::spawn_blocking(move || -> Result<serde_json::Value, McpError> {
        // Create JS context
        let mut ctx = Context::default();

        // Create console.log function (simplified)
        let log_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            if let Some(arg) = args.first() {
                let msg = arg.to_string(context).unwrap_or_default();
                info!("[JS] {}", msg.to_std_string_escaped());
            }
            Ok(JsValue::undefined())
        });

        // Register log function globally for simplicity
        ctx.register_global_callable("log".into(), 1, log_fn)
            .map_err(|e| {
                McpError::internal_error(
                    "Failed to register log function",
                    Some(json!({"error": e.to_string()})),
                )
            })?;

        // Create sleep function for blocking sleep in milliseconds
        let sleep_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            if let Some(arg) = args.first() {
                let ms = arg.to_number(context).unwrap_or(0.0);
                if ms > 0.0 && ms.is_finite() {
                    let duration = std::time::Duration::from_millis(ms as u64);
                    std::thread::sleep(duration);
                    info!("[JS] Sleep for {}ms completed", ms);
                }
            }
            Ok(JsValue::undefined())
        });

        // Register sleep function globally
        ctx.register_global_callable("sleep".into(), 1, sleep_fn)
            .map_err(|e| {
                McpError::internal_error(
                    "Failed to register sleep function",
                    Some(json!({"error": e.to_string()})),
                )
            })?;

        info!("[JavaScript] Tool queues initialized, ready to execute script");

        let call_tool_fn = NativeFunction::from_fn_ptr(|_, args, _| {
            let tool_name = args
                .first()
                .and_then(|v| v.as_string())
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_default();

            let args_json_str = args
                .get(1)
                .and_then(|v| v.as_string())
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_else(|| "{}".to_string());

            let args_val: serde_json::Value =
                serde_json::from_str(&args_json_str).unwrap_or(json!({}));

            let tool_name_for_logging = tool_name.clone();

            // Generate unique response ID
            use std::time::{SystemTime, UNIX_EPOCH};
            let response_id = format!(
                "{}_{}",
                tool_name_for_logging,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );

            // Add tool call to queue
            if let Some(queue) = TOOL_QUEUE.get() {
                match queue.lock() {
                    Ok(mut q) => {
                        q.push_back((tool_name, args_val, response_id.clone()));
                        info!(
                            "[JavaScript] Queued tool call '{}' with ID: {}",
                            tool_name_for_logging, response_id
                        );
                    }
                    Err(e) => {
                        error!(
                            "[JavaScript] Failed to lock tool queue for {}: {:?}",
                            tool_name_for_logging, e
                        );
                        return Ok(boa_engine::JsString::from(
                            json!({"error": "Tool queue lock failed"}).to_string(),
                        )
                        .into());
                    }
                }
            } else {
                error!(
                    "[JavaScript] Tool queue not initialized for tool: {}",
                    tool_name_for_logging
                );
                return Ok(boa_engine::JsString::from(
                    json!({"error": "Tool queue not available"}).to_string(),
                )
                .into());
            }

            // Poll for response
            let start_time = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(30);

            while start_time.elapsed() < timeout {
                if let Some(responses) = RESPONSE_MAP.get() {
                    if let Ok(resp_map) = responses.lock() {
                        if let Some(result) = resp_map.get(&response_id) {
                            info!(
                                "[JavaScript] Tool '{}' completed successfully",
                                tool_name_for_logging
                            );
                            return Ok(boa_engine::JsString::from(result.clone()).into());
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            error!(
                "[JavaScript] Tool '{}' timed out after 30s",
                tool_name_for_logging
            );
            Ok(
                boa_engine::JsString::from(json!({"error": "Tool call timed out"}).to_string())
                    .into(),
            )
        });

        ctx.register_global_callable("callTool".into(), 2, call_tool_fn)
            .map_err(|e| {
                McpError::internal_error(
                    "Failed to register callTool function",
                    Some(json!({"error": e.to_string()})),
                )
            })?;
        // Execute user script
        info!(
            "[JavaScript] Starting execution of script ({} bytes)",
            script_src.len()
        );
        let js_value = ctx.eval(Source::from_bytes(&script_src)).map_err(|e| {
            // Log detailed error information for debugging
            let error_details = format!("{e:?}");
            let error_display = format!("{e}");

            error!("[JavaScript] Script execution failed!");
            error!("[JavaScript] Error details: {}", error_details);
            error!("[JavaScript] Error display: {}", error_display);

            // Log the script content for debugging (first 500 chars)
            let script_preview = if script_src.len() > 500 {
                format!("{}...", &script_src[..500])
            } else {
                script_src.clone()
            };
            error!("[JavaScript] Failed script content: {}", script_preview);

            McpError::internal_error(
                "JavaScript evaluation error",
                Some(json!({
                    "error": error_display,
                    "error_details": error_details,
                    "script_preview": script_preview
                })),
            )
        })?;

        info!("[JavaScript] Script execution completed successfully");

        // Convert to JSON while we still have the context
        let json_result = boa_js_to_json_with_context(js_value, &mut ctx);

        Ok(json_result)
    });

    // Handle tool calls from JavaScript using queue polling
    let tool_handler = {
        let tool_dispatcher = tool_dispatcher.clone();
        tokio::spawn(async move {
            info!("[JavaScript->Rust] Tool handler started, polling queue for tool calls...");

            loop {
                // Check for tool calls in queue
                let tool_call = if let Some(queue) = TOOL_QUEUE.get() {
                    queue.lock().ok().and_then(|mut q| q.pop_front())
                } else {
                    None
                };

                if let Some((tool_name, args_val, response_id)) = tool_call {
                    debug!(
                        "[JavaScript->Rust] Processing tool call: '{}' with ID: {}",
                        tool_name, response_id
                    );

                    let result_json = match tool_dispatcher(tool_name.clone(), args_val).await {
                        Ok(result_json) => {
                            debug!(
                                "[JavaScript->Rust] Tool '{}' executed successfully, result length: {}",
                                tool_name,
                                result_json.len()
                            );
                            result_json
                        }
                        Err(e) => {
                            error!(
                                "[JavaScript->Rust] Tool '{}' failed with error: {}",
                                tool_name, e
                            );
                            json!({"error": e.to_string()}).to_string()
                        }
                    };

                    // Store response in map
                    if let Some(responses) = RESPONSE_MAP.get() {
                        if let Ok(mut resp_map) = responses.lock() {
                            resp_map.insert(response_id.clone(), result_json);
                            debug!(
                                "[JavaScript->Rust] Response stored for tool '{}' with ID: {}",
                                tool_name, response_id
                            );
                        } else {
                            error!(
                                "[JavaScript->Rust] Failed to lock response map for tool '{}'",
                                tool_name
                            );
                        }
                    }
                } else {
                    // No tool calls, sleep briefly
                    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                }

                // Note: This loop runs until the task is aborted from the main thread
            }
        })
    };

    // Wait for JavaScript execution to complete
    let execution_result = js_handle.await.map_err(|e| {
        error!("[JavaScript] Execution task panicked: {}", e);
        McpError::internal_error(
            "JavaScript execution task failed",
            Some(json!({"error": e.to_string()})),
        )
    })??;

    info!("[JavaScript] Execution completed, shutting down tool handler");

    // Abort the tool handler since we don't need it anymore
    tool_handler.abort();

    // Clean up the queues
    if let Some(queue) = TOOL_QUEUE.get() {
        if let Ok(mut q) = queue.lock() {
            q.clear();
            info!("[JavaScript] Tool queue cleaned up successfully");
        }
    }

    if let Some(responses) = RESPONSE_MAP.get() {
        if let Ok(mut resp_map) = responses.lock() {
            resp_map.clear();
            info!("[JavaScript] Response map cleaned up successfully");
        }
    }

    info!("[JavaScript] Tool handler aborted and cleanup complete");

    Ok(execution_result)
}
