use std::time::Duration;
use terminator::{AutomationError, Desktop};
use tracing::info;

#[tokio::test]
async fn test_simple_script() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();

    info!("🚀 Starting simple script test");

    let desktop = Desktop::new(true, false)?;

    // Open a simple page
    info!("🌐 Opening browser");
    let browser = desktop.open_url("https://example.com", None)?;

    // Wait a bit
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Test a super simple script
    info!("📝 Testing simple math");
    let result = browser.execute_browser_script("2 + 2").await?;

    info!("✅ Got result: {}", result);
    assert_eq!(result.trim(), "4");

    Ok(())
}
