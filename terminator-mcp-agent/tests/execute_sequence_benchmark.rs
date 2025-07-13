// 07/13/25 11:51 AM PST
// 🔍 Per-tool timings:
//   • navigate_browser     |  2441 ms | success
//   • wait_for_element     |    41 ms | success
//   • type_into_element    |  1771 ms | success
//   • type_into_element    |  1239 ms | success
//   • click_element        |  2298 ms | success
//   • click_element        |  2317 ms | success
//   • wait_for_element     |    64 ms | success

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
/// 2. The target URL can be set via the `MCP_BENCH_TARGET_URL` environment variable. If not provided, it defaults to Selenium's demo form `https://www.selenium.dev/selenium/web/web-form.html`.
/// 3. The test is ignored by default – run with `cargo test -- --ignored` to execute the benchmark.
#[tokio::test]
#[ignore]
async fn benchmark_execute_sequence_real_website() -> Result<()> {
    let agent_path = get_agent_binary_path();
    if !agent_path.exists() {
        eprintln!(
            "Skipping benchmark: MCP agent binary not found at {agent_path:?}. Build it first with `cargo build --release --bin terminator-mcp-agent`."
        );
        return Ok(());
    }

    // Default to a stable, public demo form if no URL is provided
    let target_url = env::var("MCP_BENCH_TARGET_URL")
        .unwrap_or_else(|_| "https://www.selenium.dev/selenium/web/web-form.html".to_string());

    // Spawn the MCP agent in stdio transport mode
    let mut cmd = Command::new(&agent_path);
    cmd.args(["-t", "stdio"]);
    let service = ().serve(TokioChildProcess::new(cmd)?).await?;

    // Build a realistic workflow that fills out the form and submits it.
    // Field selectors correspond to the Selenium demo page.
    let steps = serde_json::json!([
        {
            "tool_name": "navigate_browser",
            "arguments": { "url": target_url }
        },
        {
            "tool_name": "wait_for_element",
            "arguments": {"selector": "role:Edit|name:Text input", "condition": "visible", "timeout_ms": 10000 }
        },
        {
            "tool_name": "type_into_element",
            "arguments": {"selector": "role:Edit|name:Text input", "text_to_type": "Terminator Bot", "clear_before_typing": true, "verify_action": false }
        },
        {
            "tool_name": "type_into_element",
            "arguments": {"selector": "role:Edit|name:Password", "text_to_type": "Secret123", "clear_before_typing": true, "verify_action": false }
        },
        {
            "tool_name": "click_element",
            "arguments": {"selector": "role:CheckBox|name:Default checkbox"}
        },
        {
            "tool_name": "click_element",
            "arguments": {"selector": "role:Button|name:Submit"}
        },
        {
            "tool_name": "wait_for_element",
            "arguments": {"selector": "role:Button|name:Submit", "condition": "visible", "timeout_ms": 5000 }
        }
    ]);

    let args = serde_json::json!({
        "steps": steps,
        "stop_on_error": true,
        "include_detailed_results": true
    });

    // Measure total wall-clock time around the call
    let start = Instant::now();
    let result = service
        .call_tool(CallToolRequestParam {
            name: "execute_sequence".into(),
            arguments: Some(args.as_object().unwrap().clone()),
        })
        .await?;
    let elapsed = start.elapsed();

    // Parse the response to extract timings and print per-tool metrics
    if let Some(content) = result.content.first() {
        let json_str = serde_json::to_string(content)?;
        let parsed: serde_json::Value = serde_json::from_str(&json_str)?;
        if let Some(text) = parsed.get("text").and_then(|t| t.as_str()) {
            let response: serde_json::Value = serde_json::from_str(text)?;

            // Overall duration comparison
            let reported_ms = response
                .get("total_duration_ms")
                .and_then(|v| v.as_i64())
                .unwrap_or_default();

            println!(
                "� execute_sequence: wall-clock {elapsed:?} – server reported {reported_ms} ms"
            );

            // Per-tool timings
            if let Some(results) = response.get("results").and_then(|r| r.as_array()) {
                println!("\n🔍 Per-tool timings:");
                for step in results {
                    let name = step
                        .get("tool_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<unknown>");
                    let duration = step
                        .get("duration_ms")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let status = step
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<n/a>");
                    println!("  • {name:<20} | {duration:>5} ms | {status}");
                }
            }

            // Sanity: success expected
            assert_eq!(
                response.get("status").and_then(|v| v.as_str()),
                Some("success")
            );

            // Ensure reported duration is not longer than wall-clock
            assert!(
                reported_ms as u128 <= elapsed.as_millis(),
                "Server-reported duration ({reported_ms} ms) > wall-clock ({} ms)",
                elapsed.as_millis()
            );
        }
    }

    // Clean up
    service.cancel().await?;
    Ok(())
}
