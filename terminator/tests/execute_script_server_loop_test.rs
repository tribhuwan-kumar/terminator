use std::time::Duration;
use terminator::{AutomationError, Desktop};
use tracing::info;

/// Ensures the local HTTP server keeps accepting connections (heartbeats)
/// and does not return until the final result arrives.
#[tokio::test]
async fn test_heartbeat_then_result() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();

    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://example.com", None)?;
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // 16s promise → wrapper sends a heartbeat at 15s, then final result at 16s
    let script = r#"
        new Promise(resolve => {
            setTimeout(() => resolve('ok-16s'), 16000);
        })
    "#;

    let start = std::time::Instant::now();
    let result = browser.execute_browser_script(script).await?;
    let elapsed = start.elapsed();

    info!(
        "✅ Got result: {} (elapsed: {:.1}s)",
        result,
        elapsed.as_secs_f32()
    );
    assert_eq!(result.trim(), "ok-16s");
    assert!(elapsed.as_secs() >= 16, "should wait past first heartbeat");
    assert!(elapsed.as_secs() < 30, "should not exceed reasonable bound");

    Ok(())
}

/// Ensures thrown errors in the browser are propagated back through the server path
#[tokio::test]
async fn test_error_propagation_from_browser() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();

    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://example.com", None)?;
    tokio::time::sleep(Duration::from_millis(2000)).await;

    let script = "throw new Error('boom')";
    let result = browser.execute_browser_script(script).await?;
    info!("✅ Got error result: {}", result);
    assert!(
        result.starts_with("ERROR:"),
        "expected server to mark error"
    );
    assert!(result.contains("boom"));

    Ok(())
}
