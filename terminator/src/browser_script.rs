//! Cross-platform browser script execution using terminator SDK
//!
//! Uses terminator SDK selectors for cross-platform browser automation.
//! Finds console tab and prompt using proper selectors, runs JavaScript, extracts results.

use crate::{AutomationError, Desktop};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::info;

/// Execute JavaScript in browser using local server for result communication
pub async fn execute_script(
    browser_element: &crate::UIElement,
    script: &str,
) -> Result<String, AutomationError> {
    info!("🚀 Executing JavaScript using local server approach");

    // Step 1: Start a local server to receive results
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| AutomationError::PlatformError(format!("Failed to bind server: {e}")))?;
    let port = listener
        .local_addr()
        .map_err(|e| AutomationError::PlatformError(format!("Failed to get port: {e}")))?
        .port();

    info!("📡 Local server listening on port {}", port);

    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();
    let last_heartbeat = Arc::new(Mutex::new(std::time::Instant::now()));
    let heartbeat_clone = last_heartbeat.clone();

    // Spawn server task
    let _server_handle = tokio::spawn(async move {
        loop {
            // If a result is already set, stop the server loop
            if result_clone.lock().await.is_some() {
                break;
            }

            info!("🔌 Server waiting for connection...");
            let (mut socket, addr) = match listener.accept().await {
                Ok(ok) => ok,
                Err(e) => {
                    info!("❌ Failed to accept connection: {}", e);
                    continue;
                }
            };

            info!("📡 Connection from: {}", addr);
            let mut buf = vec![0; 65536];
            match socket.read(&mut buf).await {
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]);
                    info!(
                        "📨 Received {} bytes, first 500 chars: {}",
                        n,
                        &data[..data.len().min(500)]
                    );

                    // Parse GET request with query params
                    if data.starts_with("GET ") {
                        if let Some(line_end) = data.find('\r') {
                            let request_line = &data[4..line_end];
                            info!("📦 Request line: {}", request_line);

                            // Extract query params
                            if request_line.contains("?heartbeat=") {
                                info!("♥ Received heartbeat");
                                *heartbeat_clone.lock().await = std::time::Instant::now();
                            } else if request_line.contains("?result=") {
                                if let Some(query_start) = request_line.find("?result=") {
                                    let result_encoded = &request_line[query_start + 8..];
                                    let result_end =
                                        result_encoded.find(' ').unwrap_or(result_encoded.len());
                                    let result_encoded = &result_encoded[..result_end];

                                    info!("📦 Encoded result: {}", result_encoded);

                                    // Simple URL decode (just handle %20 for spaces and basic chars)
                                    let decoded = result_encoded
                                        .replace("%20", " ")
                                        .replace("%22", "\"")
                                        .replace("%2C", ",");
                                    info!("📦 Decoded result: {}", decoded);
                                    *result_clone.lock().await = Some(decoded.to_string());
                                }
                            } else if request_line.contains("?error=") {
                                if let Some(query_start) = request_line.find("?error=") {
                                    let error_encoded = &request_line[query_start + 7..];
                                    let error_end =
                                        error_encoded.find(' ').unwrap_or(error_encoded.len());
                                    let error_encoded = &error_encoded[..error_end];

                                    let decoded = error_encoded
                                        .replace("%20", " ")
                                        .replace("%22", "\"")
                                        .replace("%2C", ",");
                                    info!("📦 Decoded error: {}", decoded);
                                    *result_clone.lock().await = Some(format!("ERROR: {decoded}"));
                                }
                            }
                        }
                    }

                    // Send HTTP response
                    let response = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 2\r\n\r\nOK";
                    let _ =
                        tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes()).await;
                }
                Err(e) => info!("❌ Failed to read from socket: {}", e),
            }

            // If we have received a result, break out of the loop to let the caller proceed
            if result_clone.lock().await.is_some() {
                break;
            }
        }
    });

    // Wait a moment for server to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Step 2: Focus browser
    browser_element.focus()?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Step 3: Wrap script to send result to our server
    let wrapped_script = format!(
        r#"
        (async function() {{
            // Keep image references to avoid GC cancelling requests
            window.__terminatorImgs = window.__terminatorImgs || [];
            const __sendPing = (q) => {{
                try {{
                    const img = new Image();
                    window.__terminatorImgs.push(img);
                    // cache-bust to avoid any caching
                    img.src = 'http://127.0.0.1:{port}/?'+ q + '&t=' + Date.now();
                }} catch (e) {{
                    // best-effort, ignore
                }}
            }};

            // Monkey-patch console.log to piggyback heartbeats during long runs
            (function() {{
                const __origLog = console.log;
                let __lastPing = 0;
                console.log = function(...args) {{
                    const now = Date.now();
                    if (now - __lastPing > 3000) {{
                        try {{ __sendPing('heartbeat=alive'); }} catch (_) {{}}
                        __lastPing = now;
                    }}
                    return __origLog.apply(this, args);
                }};
            }})()

            // Immediate heartbeat so Rust knows we started
            __sendPing('heartbeat=alive');
            // Heartbeat every 5s (shorter to survive long sync blocks)
            const heartbeatInterval = setInterval(() => {{
                __sendPing('heartbeat=alive');
                console.log('♥ Heartbeat sent');
            }}, 5000);
            
            try {{
                // For async scripts, await the result
                let scriptResult;
                const scriptCode = `{script}`;
                
                // Check if the script is async or returns a promise
                try {{
                    const evalResult = eval(scriptCode);
                    if (evalResult && typeof evalResult.then === 'function') {{
                        scriptResult = await evalResult;
                    }} else {{
                        scriptResult = evalResult;
                    }}
                }} catch (syncError) {{
                    // If eval fails, it's a sync error
                    throw syncError;
                }}
                
                const resultStr = typeof scriptResult === 'object' ? JSON.stringify(scriptResult) : String(scriptResult);
                
                // Clear heartbeat and send result
                clearInterval(heartbeatInterval);
                __sendPing('result=' + encodeURIComponent(resultStr));
                console.log('Result:', resultStr);
                
                return scriptResult;
            }} catch (e) {{
                clearInterval(heartbeatInterval);
                __sendPing('error=' + encodeURIComponent(e && (e.message || String(e))));
                console.error('Error:', e.message);
                throw e;
            }}
        }})()
        "#
    );

    // Step 3: Open dev tools if not already open (retry strategy)
    let desktop = Desktop::new(true, false)?;
    let mut console_prompt_opt: Option<crate::UIElement> = None;

    for attempt in 1..=3 {
        info!("⚙️ Opening dev tools (attempt {}): Ctrl+Shift+J", attempt);
        browser_element.press_key("{Ctrl}{Shift}J")?;
        tokio::time::sleep(Duration::from_millis(1200)).await;

        match desktop
            .locator("role:document|name:DevTools >> name:Console prompt")
            .first(None)
            .await
        {
            Ok(el) => {
                console_prompt_opt = Some(el);
                break;
            }
            Err(e) => {
                info!(
                    "🔁 Console prompt not found after Ctrl+Shift+J (attempt {}): {}",
                    attempt, e
                );
                // Try toggling DevTools with F12 and re-attempt
                info!("⚙️ Toggling DevTools with F12");
                browser_element.press_key("{F12}")?;
                tokio::time::sleep(Duration::from_millis(900)).await;
            }
        }
    }

    let console_prompt = match console_prompt_opt {
        Some(el) => el,
        None => {
            // Final attempt once more with the same trusted selector
            info!("🔍 Final attempt to locate console prompt");
            desktop
                .locator("role:document|name:DevTools >> name:Console prompt")
                .first(None)
                .await?
        }
    };

    info!("⌨️ Typing wrapped JavaScript into console prompt");
    console_prompt.type_text(&wrapped_script, true)?;
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Step 6: Execute the script (Enter)
    info!("🚀 Executing script with Enter");
    console_prompt.press_key("{ENTER}")?;

    // Step 7: Wait for result from server with heartbeat tracking
    info!("📄 Waiting for result from browser...");
    let mut elapsed_seconds = 0;
    let max_timeout_seconds = 300; // 5 minutes absolute max
    let heartbeat_timeout_seconds = 300; // Allow very long blocking sections without heartbeats

    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
        elapsed_seconds += 1; // Each iteration is 0.5 seconds

        if let Some(res) = result.lock().await.as_ref() {
            let final_result = res.clone();

            // Close dev tools
            info!("🚪 Closing dev tools");
            browser_element.press_key("{F12}")?;

            info!("✅ Script execution completed: {}", final_result);
            return Ok(final_result);
        }

        // Check absolute timeout (5 minutes)
        if elapsed_seconds >= max_timeout_seconds * 2 {
            info!("⏰ Absolute timeout reached (5 minutes)");
            break;
        }

        // Check heartbeat timeout (35 seconds without heartbeat)
        // Give 15 seconds grace period before checking heartbeats
        let last_hb = *last_heartbeat.lock().await;
        if elapsed_seconds > 10 && last_hb.elapsed().as_secs() > heartbeat_timeout_seconds as u64 {
            info!(
                "💔 Heartbeat timeout - no heartbeat for {} seconds",
                last_hb.elapsed().as_secs()
            );
            break;
        }

        // Log progress every 10 seconds
        if elapsed_seconds % 20 == 0 {
            info!(
                "⏳ Still waiting... ({} seconds elapsed, last heartbeat: {:.1}s ago)",
                elapsed_seconds / 2,
                last_hb.elapsed().as_secs_f32()
            );
        }
    }

    // Timeout - close dev tools and return error
    browser_element.press_key("{F12}")?;
    Err(AutomationError::Timeout(format!(
        "Script execution timed out (elapsed: {} seconds)",
        elapsed_seconds / 2
    )))
}
