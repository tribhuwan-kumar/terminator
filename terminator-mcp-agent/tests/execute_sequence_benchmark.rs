use anyhow::Result;
use rmcp::transport::TokioChildProcess;
use rmcp::{model::CallToolRequestParam, ServiceExt};
use std::env;
use std::path::PathBuf;
use std::time::Instant;
use tokio::process::Command;

/// Helper to get the path to the MCP agent binary (release build)
fn get_agent_binary_path() -> PathBuf {
    let mut path = env::current_exe().expect("Failed to get current_exe");
    path.pop(); // test binary name
    path.pop(); // deps
    path.pop(); // debug or release
    path.push("release");
    path.push("terminator-mcp-agent");
    #[cfg(target_os = "windows")]
    path.set_extension("exe");
    path
}

/// Runs a small real-world workflow through `execute_sequence` and prints the overall duration.
///
/// NOTE:
/// 1. This test requires a graphical environment with a browser installed as it navigates to a real website.
/// 2. The target URL can be set via the `MCP_BENCH_TARGET_URL` environment variable. If not provided, it defaults to `https://example.com`.
/// 3. The test is ignored by default – run with `cargo test -- --ignored` to execute the benchmark.
#[tokio::test]
#[ignore]
async fn benchmark_execute_sequence_real_website() -> Result<()> {
    let agent_path = get_agent_binary_path();
    if !agent_path.exists() {
        eprintln!(
            "Skipping benchmark: MCP agent binary not found at {:?}. Build it first with `cargo build --release --bin terminator-mcp-agent`.",
            agent_path
        );
        return Ok(());
    }

    let target_url =
        env::var("MCP_BENCH_TARGET_URL").unwrap_or_else(|_| "https://example.com".to_string());

    // Spawn the MCP agent in stdio transport mode
    let mut cmd = Command::new(&agent_path);
    cmd.args(["-t", "stdio"]);
    let service = ().serve(TokioChildProcess::new(cmd)?).await?;

    // Build a minimal but realistic workflow: open the website and wait for <body>
    let steps = serde_json::json!([
        {
            "tool_name": "navigate_browser",
            "arguments": { "url": target_url },
        },
        {
            "tool_name": "wait_for_element",
            "arguments": {
                "selector": "body",
                "timeout_ms": 10000
            }
        }
    ]);

    let args = serde_json::json!({
        "steps": steps,
        "stop_on_error": true,
        "include_detailed_results": false
    });

    // Measure total wall-clock time around the call
    let start = Instant::now();
    let result = service
        .call_tool(CallToolRequestParam {
            name: "execute_sequence".into(),
            arguments: Some(args),
        })
        .await?;
    let elapsed = start.elapsed();

    // Parse the response to extract the reported `total_duration_ms`
    if let Some(content) = result.content.get(0) {
        let json_str = serde_json::to_string(content)?;
        let parsed: serde_json::Value = serde_json::from_str(&json_str)?;
        if let Some(text) = parsed.get("text").and_then(|t| t.as_str()) {
            let response: serde_json::Value = serde_json::from_str(text)?;
            // Log both client-side and server-reported durations for comparison
            let reported_ms = response
                .get("total_duration_ms")
                .and_then(|v| v.as_i64())
                .unwrap_or_default();

            println!(
                "🏎️  execute_sequence completed in {:?} (client-side) – server reported {} ms",
                elapsed, reported_ms
            );

            // Basic sanity check: server-reported duration should not exceed client-side measurement
            assert!(
                reported_ms as u128 <= elapsed.as_millis(),
                "Server-reported duration ({reported_ms} ms) is larger than wall-clock measurement ({} ms)",
                elapsed.as_millis()
            );
        }
    }

    // Clean up
    service.cancel().await?;
    Ok(())
}
