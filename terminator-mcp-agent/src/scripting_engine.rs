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

/// Find executable with cross-platform path resolution
pub fn find_executable(name: &str) -> Option<String> {
    use std::env;
    use std::path::Path;

    // On Windows, try both with and without .exe extension
    let candidates = if cfg!(windows) {
        vec![name.to_string(), format!("{}.exe", name)]
    } else {
        vec![name.to_string()]
    };

    // Check each candidate in PATH
    if let Ok(path_var) = env::var("PATH") {
        let separator = if cfg!(windows) { ";" } else { ":" };

        for path_dir in path_var.split(separator) {
            let path_dir = Path::new(path_dir);

            for candidate in &candidates {
                let full_path = path_dir.join(candidate);
                if full_path.exists() && full_path.is_file() {
                    info!("Found executable: {}", full_path.display());
                    return Some(full_path.to_string_lossy().to_string());
                }
            }
        }
    }

    // Fallback: try the name as-is (might work on some systems)
    info!("Executable '{}' not found in PATH, using name as-is", name);
    Some(name.to_string())
}

/// Ensure terminator.js is installed in a persistent directory and return the script directory
async fn ensure_terminator_js_installed(runtime: &str) -> Result<std::path::PathBuf, McpError> {
    use tokio::process::Command;

    info!("[{}] Checking if terminator.js is installed...", runtime);

    // Use a persistent directory instead of a new temp directory each time
    let script_dir = std::env::temp_dir().join("terminator_mcp_persistent");

    // Check if terminator.js is already installed in the persistent directory
    let node_modules_path = script_dir.join("node_modules").join("terminator.js");
    let package_json_path = script_dir.join("package.json");

    if node_modules_path.exists() && package_json_path.exists() {
        info!(
            "[{}] terminator.js already installed in persistent directory: {}",
            runtime,
            script_dir.display()
        );
        return Ok(script_dir);
    }

    info!(
        "[{}] terminator.js not found in persistent directory, installing...",
        runtime
    );

    // Find the runtime executable
    let runtime_exe = find_executable(runtime).ok_or_else(|| {
        McpError::internal_error(format!("Could not find {runtime} executable"), None)
    })?;

    info!("[{}] Using executable: {}", runtime, runtime_exe);

    // Find npm/bun for installation
    let installer_exe = match runtime {
        "bun" => find_executable("bun").ok_or_else(|| {
            McpError::internal_error("Could not find bun executable for installation", None)
        })?,
        _ => find_executable("npm").ok_or_else(|| {
            McpError::internal_error("Could not find npm executable for installation", None)
        })?,
    };

    info!("[{}] Using installer: {}", runtime, installer_exe);

    // Create the persistent directory if it doesn't exist
    tokio::fs::create_dir_all(&script_dir).await.map_err(|e| {
        McpError::internal_error(
            "Failed to create persistent script directory",
            Some(json!({"error": e.to_string()})),
        )
    })?;

    info!(
        "[{}] Created/using persistent script directory: {}",
        runtime,
        script_dir.display()
    );

    // Install terminator.js in the persistent directory
    let install_result = match runtime {
        "bun" => {
            // Bun can be executed directly
            Command::new(&installer_exe)
                .current_dir(&script_dir)
                .args(["install", "terminator.js"])
                .output()
                .await
        }
        _ => {
            // On Windows, npm is a batch file, so we need to run it through cmd.exe
            if cfg!(windows) {
                Command::new("cmd")
                    .current_dir(&script_dir)
                    .args(["/c", "npm", "install", "terminator.js"])
                    .output()
                    .await
            } else {
                Command::new(&installer_exe)
                    .current_dir(&script_dir)
                    .args(["install", "terminator.js"])
                    .output()
                    .await
            }
        }
    };

    match install_result {
        Ok(output) if output.status.success() => {
            info!(
                "[{}] terminator.js installed successfully in persistent directory",
                runtime
            );
            Ok(script_dir)
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("[{}] Failed to install terminator.js: {}", runtime, stderr);
            Err(McpError::internal_error(
                "Failed to install terminator.js",
                Some(json!({"error": stderr.to_string()})),
            ))
        }
        Err(e) => {
            error!("[{}] Failed to run install command: {}", runtime, e);
            Err(McpError::internal_error(
                "Failed to run package manager",
                Some(json!({"error": e.to_string()})),
            ))
        }
    }
}

/// Execute JavaScript using Node.js/Bun runtime with terminator.js bindings available
pub async fn execute_javascript_with_nodejs(script: String) -> Result<serde_json::Value, McpError> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    info!("[Node.js] Starting JavaScript execution with terminator.js bindings");

    // Check if bun is available, fallback to node
    let runtime = if let Some(bun_exe) = find_executable("bun") {
        match Command::new(&bun_exe).arg("--version").output().await {
            Ok(output) if output.status.success() => {
                info!("[Node.js] Found bun at: {}", bun_exe);
                "bun"
            }
            _ => {
                info!("[Node.js] Bun found but not working, falling back to node");
                "node"
            }
        }
    } else {
        info!("[Node.js] Bun not found, using node");
        "node"
    };

    info!("[Node.js] Using runtime: {}", runtime);

    // Ensure terminator.js is installed and get the script directory
    let script_dir = ensure_terminator_js_installed(runtime).await?;

    // Create a wrapper script that:
    // 1. Imports terminator.js
    // 2. Executes user script
    // 3. Returns result
    let wrapper_script = format!(
        r#"
const {{ Desktop }} = require('terminator.js');

// Create global objects
global.desktop = new Desktop();
global.log = console.log;
global.sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

// Execute user script
(async () => {{
    try {{
        const result = await (async function() {{
            {script}
        }})();
        
        // Send result back
        process.stdout.write('__RESULT__' + JSON.stringify(result) + '__END__\n');
    }} catch (error) {{
        process.stdout.write('__ERROR__' + JSON.stringify({{
            message: error.message,
            stack: error.stack
        }}) + '__END__\n');
    }}
}})();
"#
    );

    // Write script to the same directory where terminator.js is installed
    let script_path = script_dir.join("main.js");

    tokio::fs::write(&script_path, wrapper_script)
        .await
        .map_err(|e| {
            McpError::internal_error(
                "Failed to write script file",
                Some(json!({"error": e.to_string()})),
            )
        })?;

    info!("[Node.js] Script written to: {}", script_path.display());

    // Find the runtime executable for spawning
    let runtime_exe = find_executable(runtime).ok_or_else(|| {
        McpError::internal_error(
            format!("Could not find {runtime} executable for spawning"),
            None,
        )
    })?;

    info!("[Node.js] Spawning {} at: {}", runtime, runtime_exe);

    // Spawn Node.js/Bun process from the script directory
    let mut child = if runtime == "bun" {
        // Bun can be executed directly
        Command::new(&runtime_exe)
            .current_dir(&script_dir)
            .arg("main.js")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else if cfg!(windows) && runtime_exe.ends_with(".exe") {
        // Direct execution should work for .exe files
        Command::new(&runtime_exe)
            .current_dir(&script_dir)
            .arg("main.js")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else if cfg!(windows) {
        // Fallback to cmd.exe for batch files
        Command::new("cmd")
            .current_dir(&script_dir)
            .args(["/c", "node", "main.js"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else {
        Command::new(&runtime_exe)
            .current_dir(&script_dir)
            .arg("main.js")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
    .map_err(|e| {
        McpError::internal_error(
            format!("Failed to spawn {runtime} process"),
            Some(json!({"error": e.to_string()})),
        )
    })?;

    let mut stdout = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut result = None;

    // Handle communication with Node.js process
    while let Ok(Some(line)) = stdout.next_line().await {
        if line.starts_with("__RESULT__") && line.ends_with("__END__") {
            // Parse final result
            let result_json = line.replace("__RESULT__", "").replace("__END__", "");
            result = serde_json::from_str(&result_json).ok();
            break;
        } else if line.starts_with("__ERROR__") && line.ends_with("__END__") {
            // Parse error
            let error_json = line.replace("__ERROR__", "").replace("__END__", "");
            if let Ok(error_data) = serde_json::from_str::<serde_json::Value>(&error_json) {
                return Err(McpError::internal_error(
                    "JavaScript execution error",
                    Some(error_data),
                ));
            }
            break;
        } else {
            // Regular console output
            info!("[Node.js] {}", line);
        }
    }

    // Wait for process to complete
    let status = child.wait().await.map_err(|e| {
        McpError::internal_error(
            "Node.js process failed",
            Some(json!({"error": e.to_string()})),
        )
    })?;

    // Don't clean up script directory - keep it persistent for reuse
    // (Previously this cleaned up with: tokio::fs::remove_dir_all(&script_dir).await.ok();)
    info!(
        "[Node.js] Keeping persistent script directory for reuse: {}",
        script_dir.display()
    );

    if !status.success() {
        return Err(McpError::internal_error(
            "Node.js process exited with error",
            Some(json!({"exit_code": status.code()})),
        ));
    }

    result.ok_or_else(|| McpError::internal_error("No result received from Node.js process", None))
}

/// Execute JavaScript using Node.js runtime with LOCAL terminator.js bindings (for development/testing)
pub async fn execute_javascript_with_local_bindings(
    script: String,
) -> Result<serde_json::Value, McpError> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    info!("[Node.js Local] Starting JavaScript execution with local terminator.js bindings");

    // Check if bun is available, fallback to node
    let runtime = if let Some(bun_exe) = find_executable("bun") {
        match Command::new(&bun_exe).arg("--version").output().await {
            Ok(output) if output.status.success() => {
                info!("[Node.js Local] Found bun at: {}", bun_exe);
                "bun"
            }
            _ => {
                info!("[Node.js Local] Bun found but not working, falling back to node");
                "node"
            }
        }
    } else {
        info!("[Node.js Local] Bun not found, using node");
        "node"
    };

    info!("[Node.js Local] Using runtime: {}", runtime);

    // Get workspace root - assuming we're running from terminator-mcp-agent directory
    let workspace_root = std::env::current_dir()
        .map_err(|e| {
            McpError::internal_error(
                "Failed to get current directory",
                Some(json!({"error": e.to_string()})),
            )
        })?
        .parent()
        .ok_or_else(|| McpError::internal_error("Failed to find workspace root", None))?
        .to_path_buf();

    let local_bindings_path = workspace_root.join("bindings").join("nodejs");

    // Verify the local bindings directory exists
    if tokio::fs::metadata(&local_bindings_path).await.is_err() {
        return Err(McpError::internal_error(
            "Local bindings directory not found",
            Some(json!({"expected_path": local_bindings_path.to_string_lossy()})),
        ));
    }

    info!(
        "[Node.js Local] Using local bindings at: {}",
        local_bindings_path.display()
    );

    // Build the local bindings if needed
    info!("[Node.js Local] Building local terminator.js bindings...");
    let build_result = if cfg!(windows) {
        Command::new("cmd")
            .current_dir(&local_bindings_path)
            .args(["/c", "npm", "run", "build"])
            .output()
            .await
    } else {
        Command::new("npm")
            .current_dir(&local_bindings_path)
            .args(["run", "build"])
            .output()
            .await
    };

    match build_result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                info!(
                    "[Node.js Local] Build failed, continuing with existing build: {}",
                    stderr
                );
            } else {
                info!("[Node.js Local] Local bindings built successfully");
            }
        }
        Err(e) => {
            info!(
                "[Node.js Local] Failed to run build command, continuing: {}",
                e
            );
        }
    }

    // Create isolated test directory for this execution
    let script_dir = std::env::temp_dir().join(format!(
        "terminator_mcp_local_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    tokio::fs::create_dir_all(&script_dir).await.map_err(|e| {
        McpError::internal_error(
            "Failed to create script directory",
            Some(json!({"error": e.to_string()})),
        )
    })?;

    info!(
        "[Node.js Local] Created script directory: {}",
        script_dir.display()
    );

    // Create package.json that references the local bindings
    let package_json = format!(
        r#"{{
  "name": "terminator-mcp-local-execution",
  "version": "1.0.0",
  "dependencies": {{
    "terminator.js": "file:{}"
  }}
}}"#,
        local_bindings_path.to_string_lossy().replace('\\', "/")
    );

    let package_json_path = script_dir.join("package.json");
    tokio::fs::write(&package_json_path, package_json)
        .await
        .map_err(|e| {
            McpError::internal_error(
                "Failed to write package.json",
                Some(json!({"error": e.to_string()})),
            )
        })?;

    // Install the local bindings
    info!("[Node.js Local] Installing local terminator.js...");
    let install_result = if cfg!(windows) {
        Command::new("cmd")
            .current_dir(&script_dir)
            .args(["/c", "npm", "install"])
            .output()
            .await
    } else {
        Command::new("npm")
            .current_dir(&script_dir)
            .args(["install"])
            .output()
            .await
    };

    match install_result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(McpError::internal_error(
                    "Failed to install local bindings",
                    Some(json!({"error": stderr.to_string()})),
                ));
            } else {
                info!("[Node.js Local] Local bindings installed successfully");
            }
        }
        Err(e) => {
            return Err(McpError::internal_error(
                "Failed to run npm install",
                Some(json!({"error": e.to_string()})),
            ));
        }
    }

    // Create a wrapper script that:
    // 1. Imports local terminator.js
    // 2. Executes user script
    // 3. Returns result
    let wrapper_script = format!(
        r#"
const {{ Desktop }} = require('terminator.js');

// Create global objects
global.desktop = new Desktop();
global.log = console.log;
global.sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

// Execute user script
(async () => {{
    try {{
        const result = await (async function() {{
            {script}
        }})();
        
        // Send result back
        process.stdout.write('__RESULT__' + JSON.stringify(result) + '__END__\n');
    }} catch (error) {{
        process.stdout.write('__ERROR__' + JSON.stringify({{
            message: error.message,
            stack: error.stack
        }}) + '__END__\n');
    }}
}})();
"#
    );

    // Write script to the directory with local bindings
    let script_path = script_dir.join("main.js");
    tokio::fs::write(&script_path, wrapper_script)
        .await
        .map_err(|e| {
            McpError::internal_error(
                "Failed to write script file",
                Some(json!({"error": e.to_string()})),
            )
        })?;

    info!(
        "[Node.js Local] Script written to: {}",
        script_path.display()
    );

    // Find the runtime executable for spawning
    let runtime_exe = find_executable(runtime).ok_or_else(|| {
        McpError::internal_error(
            format!("Could not find {runtime} executable for spawning"),
            None,
        )
    })?;

    info!("[Node.js Local] Spawning {} at: {}", runtime, runtime_exe);

    // Spawn Node.js/Bun process from the script directory
    let mut child = if runtime == "bun" {
        // Bun can be executed directly
        Command::new(&runtime_exe)
            .current_dir(&script_dir)
            .arg("main.js")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else if cfg!(windows) && runtime_exe.ends_with(".exe") {
        // Direct execution should work for .exe files
        Command::new(&runtime_exe)
            .current_dir(&script_dir)
            .arg("main.js")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else if cfg!(windows) {
        // Fallback to cmd.exe for batch files
        Command::new("cmd")
            .current_dir(&script_dir)
            .args(["/c", "node", "main.js"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else {
        Command::new(&runtime_exe)
            .current_dir(&script_dir)
            .arg("main.js")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
    .map_err(|e| {
        McpError::internal_error(
            format!("Failed to spawn {runtime} process"),
            Some(json!({"error": e.to_string()})),
        )
    })?;

    let mut stdout = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut result = None;

    // Handle communication with Node.js process
    while let Ok(Some(line)) = stdout.next_line().await {
        if line.starts_with("__RESULT__") && line.ends_with("__END__") {
            // Parse final result
            let result_json = line.replace("__RESULT__", "").replace("__END__", "");
            result = serde_json::from_str(&result_json).ok();
            break;
        } else if line.starts_with("__ERROR__") && line.ends_with("__END__") {
            // Parse error
            let error_json = line.replace("__ERROR__", "").replace("__END__", "");
            if let Ok(error_data) = serde_json::from_str::<serde_json::Value>(&error_json) {
                return Err(McpError::internal_error(
                    "JavaScript execution error with local bindings",
                    Some(error_data),
                ));
            }
            break;
        } else {
            // Regular console output
            info!("[Node.js Local] {}", line);
        }
    }

    // Wait for process to complete
    let status = child.wait().await.map_err(|e| {
        McpError::internal_error(
            "Node.js process failed",
            Some(json!({"error": e.to_string()})),
        )
    })?;

    // Clean up script directory
    tokio::fs::remove_dir_all(&script_dir).await.ok();
    info!("[Node.js Local] Cleaned up script directory");

    if !status.success() {
        return Err(McpError::internal_error(
            "Node.js process exited with error",
            Some(json!({"exit_code": status.code()})),
        ));
    }

    result.ok_or_else(|| McpError::internal_error("No result received from Node.js process", None))
}
