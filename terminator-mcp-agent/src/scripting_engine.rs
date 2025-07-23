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
    // Initialize queues only once to prevent spam
    TOOL_QUEUE.get_or_init(|| Arc::new(Mutex::new(VecDeque::new())));
    RESPONSE_MAP.get_or_init(|| Arc::new(Mutex::new(std::collections::HashMap::new())));

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

        debug!("[JavaScript] Tool queues initialized, ready to execute script");

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
            debug!("[JavaScript->Rust] Tool handler started, polling queue for tool calls...");

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
                    // No tool calls, sleep longer to reduce CPU usage and log spam
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
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

    // On Windows, try multiple extensions, prioritizing executable types
    let candidates = if cfg!(windows) {
        vec![
            format!("{}.exe", name),
            format!("{}.cmd", name),
            format!("{}.bat", name),
            name.to_string(),
        ]
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
    info!("[{}] Checking if terminator.js is installed...", runtime);

    // Use a persistent directory instead of a new temp directory each time
    let script_dir = std::env::temp_dir().join("terminator_mcp_persistent");

    // Check if we need to install or update terminator.js
    let node_modules_path = script_dir.join("node_modules").join("terminator.js");
    let package_json_path = script_dir.join("package.json");

    // Create the persistent directory if it doesn't exist
    tokio::fs::create_dir_all(&script_dir).await.map_err(|e| {
        McpError::internal_error(
            "Failed to create persistent script directory",
            Some(json!({"error": e.to_string()})),
        )
    })?;

    // Always create/update package.json to ensure we use latest with all platform packages
    let package_json_content = r#"{
  "name": "terminator-mcp-persistent",
  "version": "1.0.0",
  "dependencies": {
    "terminator.js": "latest"
  },
  "optionalDependencies": {
    "terminator.js-darwin-arm64": "latest",
    "terminator.js-darwin-x64": "latest", 
    "terminator.js-linux-x64-gnu": "latest",
    "terminator.js-win32-arm64-msvc": "latest",
    "terminator.js-win32-x64-msvc": "latest"
  }
}"#;

    tokio::fs::write(&package_json_path, package_json_content)
        .await
        .map_err(|e| {
            McpError::internal_error(
                "Failed to write package.json",
                Some(json!({"error": e.to_string()})),
            )
        })?;

    // Check if we need to install/reinstall
    let should_install = !node_modules_path.exists();

    // Check if platform-specific package exists for current platform
    let platform_package_name = if cfg!(target_arch = "x86_64") && cfg!(target_os = "windows") {
        "terminator.js-win32-x64-msvc"
    } else if cfg!(target_arch = "aarch64") && cfg!(target_os = "windows") {
        "terminator.js-win32-arm64-msvc"
    } else if cfg!(target_arch = "x86_64") && cfg!(target_os = "macos") {
        "terminator.js-darwin-x64"
    } else if cfg!(target_arch = "aarch64") && cfg!(target_os = "macos") {
        "terminator.js-darwin-arm64"
    } else if cfg!(target_arch = "x86_64") && cfg!(target_os = "linux") {
        "terminator.js-linux-x64-gnu"
    } else {
        ""
    };

    let platform_package_exists = if !platform_package_name.is_empty() {
        script_dir
            .join("node_modules")
            .join(platform_package_name)
            .exists()
    } else {
        true // Skip platform check for unsupported platforms
    };

    if should_install {
        info!(
            "[{}] terminator.js not found, installing latest version...",
            runtime
        );
    } else if !platform_package_exists {
        info!(
            "[{}] terminator.js found but platform package {} missing, reinstalling...",
            runtime, platform_package_name
        );
        // Remove existing node_modules to force clean reinstall
        let node_modules_dir = script_dir.join("node_modules");
        if node_modules_dir.exists() {
            info!(
                "[{}] Removing existing node_modules for clean reinstall...",
                runtime
            );
            if let Err(e) = tokio::fs::remove_dir_all(&node_modules_dir).await {
                info!(
                    "[{}] Failed to remove node_modules (continuing): {}",
                    runtime, e
                );
            }
        }
    } else {
        info!(
            "[{}] terminator.js and platform package found, using existing installation...",
            runtime
        );
        return Ok(script_dir);
    }

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

    info!(
        "[{}] Using persistent script directory: {}",
        runtime,
        script_dir.display()
    );

    // Install or reinstall terminator.js@latest in the persistent directory
    let command_args = if should_install || !platform_package_exists {
        // Fresh install or reinstall when platform package missing
        vec!["install"]
    } else {
        // Update existing packages to latest
        vec!["update"]
    };

    info!(
        "[{}] Preparing to run package manager with args: {:?}",
        runtime, command_args
    );

    // Add retry mechanism based on screenpipe patterns
    let max_retries = 3;
    let mut attempt = 0;

    loop {
        attempt += 1;
        info!(
            "[{}] Installation attempt {} of {}",
            runtime, attempt, max_retries
        );

        // Use spawn with real-time output to see what's happening
        let install_result = match runtime {
            "bun" => {
                info!(
                    "[{}] Running bun directly: {} {:?}",
                    runtime, installer_exe, command_args
                );
                info!("[{}] Spawning bun process...", runtime);
                let child = match tokio::process::Command::new(&installer_exe)
                    .current_dir(&script_dir)
                    .args(&command_args)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                {
                    Ok(child) => child,
                    Err(e) => {
                        error!("[{}] Failed to spawn bun process: {}", runtime, e);
                        return Err(McpError::internal_error(
                            "Failed to spawn bun process",
                            Some(json!({"error": e.to_string()})),
                        ));
                    }
                };

                info!(
                    "[{}] Bun process spawned, waiting for completion...",
                    runtime
                );
                child.wait_with_output().await
            }
            _ => {
                // On Windows, npm is a batch file, so we need to run it through cmd.exe
                if cfg!(windows) {
                    let mut cmd_args = vec!["/c", "npm"];
                    // Add comprehensive flags based on screenpipe patterns
                    cmd_args.extend(
                        [
                            "--verbose",
                            "--no-audit",
                            "--no-fund",
                            "--yes",
                            "--force",            // Force resolve conflicts
                            "--legacy-peer-deps", // Handle peer dependency issues
                            "--include=optional", // Ensure optional dependencies (platform packages) are installed
                            "--save-exact",       // Lock to exact versions to prevent mismatches
                            "--registry=https://registry.npmjs.org/", // Explicit registry
                        ]
                        .iter(),
                    );
                    cmd_args.extend(command_args.iter().copied());
                    info!(
                        "[{}] Running npm via cmd.exe: cmd {:?} in directory {}",
                        runtime,
                        cmd_args,
                        script_dir.display()
                    );

                    info!("[{}] About to spawn cmd.exe process...", runtime);

                    // First, let's test if npm is working at all with a simple version check
                    // Debug: show what's in the directory and what we're trying to install
                    info!("[{}] Debugging directory contents...", runtime);
                    if let Ok(entries) = std::fs::read_dir(&script_dir) {
                        for entry in entries.flatten() {
                            info!(
                                "[{}] Directory contains: {}",
                                runtime,
                                entry.file_name().to_string_lossy()
                            );
                        }
                    }

                    if let Ok(package_json) =
                        std::fs::read_to_string(script_dir.join("package.json"))
                    {
                        info!("[{}] package.json contents:\n{}", runtime, package_json);
                    } else {
                        error!("[{}] Could not read package.json!", runtime);
                    }

                    info!("[{}] Testing npm connectivity first...", runtime);
                    let test_result = tokio::process::Command::new("cmd")
                        .current_dir(&script_dir)
                        .args(["/c", "npm", "--version"])
                        .output()
                        .await;

                    match test_result {
                        Ok(output) if output.status.success() => {
                            let version = String::from_utf8_lossy(&output.stdout);
                            info!(
                                "[{}] npm version test successful: {}",
                                runtime,
                                version.trim()
                            );
                        }
                        Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            error!("[{}] npm version test failed: {}", runtime, stderr);
                            info!(
                                "[{}] npm stdout: {}",
                                runtime,
                                String::from_utf8_lossy(&output.stdout)
                            );
                        }
                        Err(e) => {
                            error!("[{}] Failed to run npm version test: {}", runtime, e);
                        }
                    }

                    // Now test npm registry connectivity
                    info!("[{}] Testing npm registry connectivity...", runtime);
                    let registry_test = tokio::process::Command::new("cmd")
                        .current_dir(&script_dir)
                        .args(["/c", "npm", "ping"])
                        .output()
                        .await;

                    match registry_test {
                        Ok(output) if output.status.success() => {
                            let ping_result = String::from_utf8_lossy(&output.stdout);
                            info!(
                                "[{}] npm registry ping successful: {}",
                                runtime,
                                ping_result.trim()
                            );
                        }
                        Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            error!("[{}] npm registry ping failed: {}", runtime, stderr);
                            info!(
                                "[{}] npm ping stdout: {}",
                                runtime,
                                String::from_utf8_lossy(&output.stdout)
                            );
                        }
                        Err(e) => {
                            error!("[{}] Failed to run npm ping test: {}", runtime, e);
                        }
                    }
                    let spawn_result = tokio::process::Command::new("cmd")
                        .current_dir(&script_dir)
                        .args(&cmd_args)
                        // Core npm configuration (based on screenpipe patterns)
                        .env("NPM_CONFIG_REGISTRY", "https://registry.npmjs.org/")
                        .env("NPM_CONFIG_LOGLEVEL", "info")
                        .env("NPM_CONFIG_PROGRESS", "true")
                        .env("NPM_CONFIG_YES", "true")
                        .env("NPM_CONFIG_FORCE", "true")
                        .env("NPM_CONFIG_LEGACY_PEER_DEPS", "true")
                        .env("NPM_CONFIG_INCLUDE", "optional")
                        .env("NPM_CONFIG_SAVE_EXACT", "true")
                        .env("NPM_CONFIG_NO_AUDIT", "true")
                        .env("NPM_CONFIG_NO_FUND", "true")
                        // Network and timeout configuration
                        .env("NPM_CONFIG_FETCH_TIMEOUT", "60000") // 60 second timeout
                        .env("NPM_CONFIG_FETCH_RETRY_MINTIMEOUT", "10000") // 10 sec min retry
                        .env("NPM_CONFIG_FETCH_RETRY_MAXTIMEOUT", "60000") // 60 sec max retry
                        .env("NPM_CONFIG_FETCH_RETRIES", "3")
                        // CI and non-interactive mode
                        .env("CI", "true")
                        .env("NODE_ENV", "production")
                        .env("NPM_CONFIG_INTERACTIVE", "false")
                        .env("NPM_CONFIG_STDIN", "false")
                        // Cache configuration to prevent corruption issues
                        .env("NPM_CONFIG_CACHE_VERIFY", "false") // Skip cache verification
                        .env("NPM_CONFIG_PREFER_OFFLINE", "false") // Always try network first
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn();

                    match spawn_result {
                        Ok(mut child) => {
                            info!(
                                "[{}] cmd.exe process spawned successfully, PID: {:?}",
                                runtime,
                                child.id()
                            );
                            info!("[{}] Waiting for npm install to complete...", runtime);

                            // Read stdout and stderr in real-time
                            use tokio::io::{AsyncBufReadExt, BufReader};

                            let stdout = child.stdout.take().unwrap();
                            let stderr = child.stderr.take().unwrap();

                            let mut stdout_reader = BufReader::new(stdout).lines();
                            let mut stderr_reader = BufReader::new(stderr).lines();

                            let timeout_duration = std::time::Duration::from_secs(300); // 5 minutes
                            let start_time = std::time::Instant::now();
                            let mut last_progress_time = start_time;

                            loop {
                                if start_time.elapsed() > timeout_duration {
                                    error!(
                                        "[{}] npm install timed out after {:?}, killing process",
                                        runtime, timeout_duration
                                    );
                                    let _ = child.kill().await;
                                    return Err(McpError::internal_error(
                                        "npm install timed out",
                                        Some(json!({"timeout_secs": timeout_duration.as_secs()})),
                                    ));
                                }

                                tokio::select! {
                                    // Read stdout
                                    stdout_line = stdout_reader.next_line() => {
                                        match stdout_line {
                                            Ok(Some(line)) => {
                                                debug!("[{}] npm stdout: {}", runtime, line);
                                                last_progress_time = std::time::Instant::now(); // Reset progress timer on output
                                            }
                                            Ok(None) => {
                                                debug!("[{}] npm stdout stream ended", runtime);
                                            }
                                            Err(e) => {
                                                error!("[{}] Error reading npm stdout: {}", runtime, e);
                                            }
                                        }
                                    }

                                    // Read stderr
                                    stderr_line = stderr_reader.next_line() => {
                                        match stderr_line {
                                            Ok(Some(line)) => {
                                                debug!("[{}] npm stderr: {}", runtime, line);
                                                last_progress_time = std::time::Instant::now(); // Reset progress timer on output
                                            }
                                            Ok(None) => {
                                                debug!("[{}] npm stderr stream ended", runtime);
                                            }
                                            Err(e) => {
                                                error!("[{}] Error reading npm stderr: {}", runtime, e);
                                            }
                                        }
                                    }

                                    // Check if process is done and show progress
                                    _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {
                                        match child.try_wait() {
                                            Ok(Some(status)) => {
                                                debug!("[{}] npm process completed with status: {:?}", runtime, status);

                                                // Read any remaining output
                                                while let Ok(Some(line)) = stdout_reader.next_line().await {
                                                    if !line.is_empty() {
                                                        debug!("[{}] npm stdout (final): {}", runtime, line);
                                                    }
                                                }
                                                while let Ok(Some(line)) = stderr_reader.next_line().await {
                                                    if !line.is_empty() {
                                                        debug!("[{}] npm stderr (final): {}", runtime, line);
                                                    }
                                                }

                                                break if status.success() {
                                                    Ok(std::process::Output {
                                                        status,
                                                        stdout: Vec::new(), // We already logged the output
                                                        stderr: Vec::new(),
                                                    })
                                                } else {
                                                    Ok(std::process::Output {
                                                        status,
                                                        stdout: Vec::new(),
                                                        stderr: format!("npm exited with code {:?}", status.code()).into_bytes(),
                                                    })
                                                };
                                            }
                                            Ok(None) => {
                                                // Still running - show progress occasionally
                                                if last_progress_time.elapsed().as_secs() >= 10 {
                                                    info!(
                                                        "[{}] npm still running... ({:.1}s elapsed, waiting for output...)",
                                                        runtime,
                                                        start_time.elapsed().as_secs_f32()
                                                    );
                                                    last_progress_time = std::time::Instant::now();
                                                }
                                            }
                                            Err(e) => {
                                                error!("[{}] Error checking npm process status: {}", runtime, e);
                                                break Err(e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("[{}] Failed to spawn cmd.exe process: {}", runtime, e);
                            error!("[{}] Working directory: {}", runtime, script_dir.display());
                            error!("[{}] Command attempted: cmd {:?}", runtime, cmd_args);
                            Err(e)
                        }
                    }
                } else {
                    info!(
                        "[{}] Running npm directly: {} {:?}",
                        runtime, installer_exe, command_args
                    );
                    info!("[{}] Spawning npm process...", runtime);
                    let child = match tokio::process::Command::new(&installer_exe)
                        .current_dir(&script_dir)
                        .args(&command_args)
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                    {
                        Ok(child) => child,
                        Err(e) => {
                            error!("[{}] Failed to spawn npm process: {}", runtime, e);
                            return Err(McpError::internal_error(
                                "Failed to spawn npm process",
                                Some(json!({"error": e.to_string()})),
                            ));
                        }
                    };

                    info!(
                        "[{}] npm process spawned, waiting for completion...",
                        runtime
                    );
                    child.wait_with_output().await
                }
            }
        };

        info!("[{}] Package manager command execution initiated", runtime);

        match install_result {
            Ok(output) => {
                info!(
                    "[{}] Package manager command completed with exit status: {:?}",
                    runtime, output.status
                );

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !stdout.is_empty() {
                    info!("[{}] Package manager stdout:\n{}", runtime, stdout);
                }
                if !stderr.is_empty() {
                    info!("[{}] Package manager stderr:\n{}", runtime, stderr);
                }

                if output.status.success() {
                    let action = if should_install {
                        "installed"
                    } else {
                        "updated"
                    };
                    info!(
                        "[{}] terminator.js {} successfully to latest version in persistent directory",
                        runtime, action
                    );

                    // Check for version mismatches and fix them
                    if !platform_package_name.is_empty() {
                        let main_pkg_json = script_dir
                            .join("node_modules")
                            .join("terminator.js")
                            .join("package.json");
                        let platform_pkg_json = script_dir
                            .join("node_modules")
                            .join(platform_package_name)
                            .join("package.json");

                        if let (Ok(main_content), Ok(platform_content)) = (
                            tokio::fs::read_to_string(&main_pkg_json).await,
                            tokio::fs::read_to_string(&platform_pkg_json).await,
                        ) {
                            if let (Ok(main_pkg), Ok(platform_pkg)) = (
                                serde_json::from_str::<serde_json::Value>(&main_content),
                                serde_json::from_str::<serde_json::Value>(&platform_content),
                            ) {
                                let main_version = main_pkg.get("version").and_then(|v| v.as_str());
                                let platform_version =
                                    platform_pkg.get("version").and_then(|v| v.as_str());

                                if let (Some(mv), Some(pv)) = (main_version, platform_version) {
                                    if mv != pv {
                                        info!("[{}] Version mismatch detected: terminator.js@{} vs {}@{}", runtime, mv, platform_package_name, pv);
                                        info!("[{}] Upgrading main package to match platform package version...", runtime);

                                        // Install specific version of main package to match platform package
                                        let package_spec = format!("terminator.js@{pv}");
                                        let upgrade_result = if runtime == "bun" {
                                            tokio::process::Command::new(&installer_exe)
                                                .current_dir(&script_dir)
                                                .args(["add", &package_spec])
                                                .output()
                                                .await
                                        } else {
                                            tokio::process::Command::new("cmd")
                                                .current_dir(&script_dir)
                                                .args(["/c", "npm", "install", &package_spec])
                                                .output()
                                                .await
                                        };

                                        match upgrade_result {
                                            Ok(out) if out.status.success() => {
                                                info!("[{}] Successfully upgraded terminator.js to match platform package version {}", runtime, pv);
                                            }
                                            Ok(out) => {
                                                let stderr = String::from_utf8_lossy(&out.stderr);
                                                info!("[{}] Failed to upgrade main package (continuing): {}", runtime, stderr);
                                            }
                                            Err(e) => {
                                                info!("[{}] Error upgrading main package (continuing): {}", runtime, e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    return Ok(script_dir); // Success - exit retry loop
                } else {
                    let action = if should_install { "install" } else { "update" };
                    let error_msg = format!(
                        "Failed to {} terminator.js with exit code {:?}: {}",
                        action,
                        output.status.code(),
                        stderr
                    );
                    error!("[{}] {}", runtime, error_msg);

                    let error = McpError::internal_error(
                        format!("Failed to {action} terminator.js"),
                        Some(json!({
                            "error": stderr.to_string(),
                            "stdout": stdout.to_string(),
                            "exit_code": output.status.code(),
                            "attempt": attempt,
                            "max_retries": max_retries
                        })),
                    );

                    // Check if we should retry
                    if attempt >= max_retries {
                        error!(
                            "[{}] All {} installation attempts failed",
                            runtime, max_retries
                        );
                        return Err(error);
                    }
                }
            }
            Err(e) => {
                let action = if should_install { "install" } else { "update" };
                let error_msg = format!("Failed to run {action} command: {e}");
                error!("[{}] {}", runtime, error_msg);

                let error = McpError::internal_error(
                    "Failed to run package manager",
                    Some(json!({
                        "error": e.to_string(),
                        "attempt": attempt,
                        "max_retries": max_retries
                    })),
                );

                // Check if we should retry
                if attempt >= max_retries {
                    error!(
                        "[{}] All {} installation attempts failed",
                        runtime, max_retries
                    );
                    return Err(error);
                }
            }
        }

        // Wait before retrying (exponential backoff like screenpipe)
        let delay_secs = 2u64.pow(attempt as u32);
        error!(
            "[{}] Installation attempt {} failed, retrying in {} seconds...",
            runtime, attempt, delay_secs
        );
        tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
    }
}

/// Execute JavaScript using Node.js/Bun runtime with terminator.js bindings available
pub async fn execute_javascript_with_nodejs(script: String) -> Result<serde_json::Value, McpError> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    info!("[Node.js] Starting JavaScript execution with terminator.js bindings");
    debug!(
        "[Node.js] Script to execute ({} bytes):\n{}",
        script.len(),
        script
    );

    // Check if bun is available, fallback to node
    let runtime = if let Some(bun_exe) = find_executable("bun") {
        match Command::new(&bun_exe).arg("--version").output().await {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                info!(
                    "[Node.js] Found bun at: {} (version: {})",
                    bun_exe,
                    version.trim()
                );
                "bun"
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                info!(
                    "[Node.js] Bun found but version check failed: {}, falling back to node",
                    stderr
                );
                "node"
            }
            Err(e) => {
                info!(
                    "[Node.js] Bun found but not working ({}), falling back to node",
                    e
                );
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
    info!("[Node.js] Script directory: {}", script_dir.display());

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

console.log('[Node.js Wrapper] Starting user script execution...');

// Execute user script
(async () => {{
    try {{
        console.log('[Node.js Wrapper] Executing user script...');
        const result = await (async function() {{
            {script}
        }})();
        
        console.log('[Node.js Wrapper] User script completed, result:', typeof result);
        
        // Send result back
        process.stdout.write('__RESULT__' + JSON.stringify(result) + '__END__\n');
        console.log('[Node.js Wrapper] Result sent back to parent process');
    }} catch (error) {{
        console.error('[Node.js Wrapper] User script error:', error.message);
        console.error('[Node.js Wrapper] Stack trace:', error.stack);
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

    info!(
        "[Node.js] Writing wrapper script to: {}",
        script_path.display()
    );
    debug!("[Node.js] Wrapper script content:\n{}", wrapper_script);

    tokio::fs::write(&script_path, wrapper_script)
        .await
        .map_err(|e| {
            error!("[Node.js] Failed to write script file: {}", e);
            McpError::internal_error(
                "Failed to write script file",
                Some(json!({"error": e.to_string(), "path": script_path.to_string_lossy()})),
            )
        })?;

    info!("[Node.js] Script written successfully");

    // Find the runtime executable for spawning
    let runtime_exe = find_executable(runtime).ok_or_else(|| {
        error!(
            "[Node.js] Could not find {} executable for spawning",
            runtime
        );
        McpError::internal_error(
            format!("Could not find {runtime} executable for spawning"),
            None,
        )
    })?;

    info!("[Node.js] Spawning {} at: {}", runtime, runtime_exe);

    // Check if executable is a batch file on Windows
    let is_batch_file =
        cfg!(windows) && (runtime_exe.ends_with(".cmd") || runtime_exe.ends_with(".bat"));

    // Spawn Node.js/Bun process from the script directory
    let mut child = if runtime == "bun" && !is_batch_file {
        info!("[Node.js] Using direct bun execution");
        // Bun can be executed directly if it's not a batch file
        Command::new(&runtime_exe)
            .current_dir(&script_dir)
            .arg("main.js")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else if cfg!(windows) && is_batch_file {
        info!("[Node.js] Using cmd.exe for batch file execution on Windows");
        // Use cmd.exe for batch files on Windows
        Command::new("cmd")
            .current_dir(&script_dir)
            .args(["/c", &runtime_exe, "main.js"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else if cfg!(windows) && runtime_exe.ends_with(".exe") {
        info!("[Node.js] Using direct .exe execution on Windows");
        // Direct execution should work for .exe files
        Command::new(&runtime_exe)
            .current_dir(&script_dir)
            .arg("main.js")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else {
        info!("[Node.js] Using direct execution");
        Command::new(&runtime_exe)
            .current_dir(&script_dir)
            .arg("main.js")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
    .map_err(|e| {
        error!("[Node.js] Failed to spawn {} process: {}", runtime, e);
        McpError::internal_error(
            format!("Failed to spawn {runtime} process"),
            Some(json!({"error": e.to_string(), "runtime_exe": runtime_exe})),
        )
    })?;

    info!("[Node.js] Process spawned successfully, reading output...");

    let mut stdout = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut stderr = BufReader::new(child.stderr.take().unwrap()).lines();
    let mut result = None;
    let mut stderr_output = Vec::new();

    // Handle communication with Node.js process
    loop {
        tokio::select! {
            stdout_line = stdout.next_line() => {
                match stdout_line {
                    Ok(Some(line)) => {
                        debug!("[Node.js stdout] {}", line);
                        if line.starts_with("__RESULT__") && line.ends_with("__END__") {
                            // Parse final result
                            let result_json = line.replace("__RESULT__", "").replace("__END__", "");
                            info!("[Node.js] Received result, parsing JSON ({} bytes)...", result_json.len());
                            debug!("[Node.js] Result JSON: {}", result_json);

                            match serde_json::from_str(&result_json) {
                                Ok(parsed_result) => {
                                    info!("[Node.js] Successfully parsed result");
                                    result = Some(parsed_result);
                                    break;
                                }
                                Err(e) => {
                                    error!("[Node.js] Failed to parse result JSON: {}", e);
                                    debug!("[Node.js] Invalid JSON was: {}", result_json);
                                }
                            }
                        } else if line.starts_with("__ERROR__") && line.ends_with("__END__") {
                            // Parse error
                            let error_json = line.replace("__ERROR__", "").replace("__END__", "");
                            error!("[Node.js] Received error from script: {}", error_json);

                            if let Ok(error_data) = serde_json::from_str::<serde_json::Value>(&error_json) {
                                return Err(McpError::internal_error(
                                    "JavaScript execution error",
                                    Some(error_data),
                                ));
                            }
                            break;
                        } else {
                            // Regular console output
                            info!("[Node.js output] {}", line);
                        }
                    }
                    Ok(None) => {
                        info!("[Node.js] stdout stream ended");
                        break;
                    }
                    Err(e) => {
                        error!("[Node.js] Error reading stdout: {}", e);
                        break;
                    }
                }
            }
            stderr_line = stderr.next_line() => {
                match stderr_line {
                    Ok(Some(line)) => {
                        error!("[Node.js stderr] {}", line);
                        stderr_output.push(line);
                    }
                    Ok(None) => {
                        debug!("[Node.js] stderr stream ended");
                    }
                    Err(e) => {
                        error!("[Node.js] Error reading stderr: {}", e);
                    }
                }
            }
        }
    }

    // Wait for process to complete
    info!("[Node.js] Waiting for process to complete...");
    let status = child.wait().await.map_err(|e| {
        error!("[Node.js] Process wait failed: {}", e);
        McpError::internal_error(
            "Node.js process failed",
            Some(json!({"error": e.to_string()})),
        )
    })?;

    info!("[Node.js] Process completed with status: {:?}", status);

    // Don't clean up script directory - keep it persistent for reuse
    info!(
        "[Node.js] Keeping persistent script directory for reuse: {}",
        script_dir.display()
    );

    if !status.success() {
        let exit_code = status.code();
        let stderr_combined = stderr_output.join("\n");

        error!("[Node.js] Process exited with error code: {:?}", exit_code);
        error!("[Node.js] Combined stderr output:\n{}", stderr_combined);

        return Err(McpError::internal_error(
            "Node.js process exited with error",
            Some(json!({
                "exit_code": exit_code,
                "stderr": stderr_combined,
                "script_path": script_path.to_string_lossy(),
                "working_directory": script_dir.to_string_lossy(),
                "runtime_exe": runtime_exe
            })),
        ));
    }

    match result {
        Some(r) => {
            info!("[Node.js] Execution completed successfully");
            Ok(r)
        }
        None => {
            error!("[Node.js] No result received from process");
            let stderr_combined = stderr_output.join("\n");
            Err(McpError::internal_error(
                "No result received from Node.js process",
                Some(json!({
                    "stderr": stderr_combined,
                    "script_path": script_path.to_string_lossy(),
                    "working_directory": script_dir.to_string_lossy()
                })),
            ))
        }
    }
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

    // Check if executable is a batch file on Windows
    let is_batch_file =
        cfg!(windows) && (runtime_exe.ends_with(".cmd") || runtime_exe.ends_with(".bat"));

    // Spawn Node.js/Bun process from the script directory
    let mut child = if runtime == "bun" && !is_batch_file {
        // Bun can be executed directly if it's not a batch file
        Command::new(&runtime_exe)
            .current_dir(&script_dir)
            .arg("main.js")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else if cfg!(windows) && is_batch_file {
        // Use cmd.exe for batch files on Windows
        Command::new("cmd")
            .current_dir(&script_dir)
            .args(["/c", &runtime_exe, "main.js"])
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
