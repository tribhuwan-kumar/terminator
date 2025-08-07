use std::time::Duration;
use terminator::{AutomationError, Desktop};
use tracing::info;

/// Minimal test to debug browser script execution
#[tokio::test]
async fn test_browser_script_minimal() -> Result<(), AutomationError> {
    // Initialize logging
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();

    info!("🚀 Starting minimal browser script test");

    // Create desktop and open browser
    info!("📱 Creating desktop instance");
    let desktop = Desktop::new(true, false)?;

    info!("🌐 Opening browser to example.com");
    let browser = desktop.open_url("https://example.com", None)?;

    // Give page more time to load
    info!("⏳ Waiting 5 seconds for page to fully load");
    tokio::time::sleep(Duration::from_millis(5000)).await;

    // Try the simplest possible script
    info!("📝 Executing simple script: 1 + 1");
    match browser.execute_browser_script("1 + 1").await {
        Ok(result) => {
            info!("✅ Success! Result: '{}'", result);
            assert_eq!(result.trim(), "2", "1 + 1 should equal 2");
        }
        Err(e) => {
            info!("❌ Error executing script: {:?}", e);
            // Let's try to understand what's happening
            info!("🔍 Attempting to find DevTools elements manually");

            // Try to find any DevTools related elements
            if let Ok(devtools) = desktop.locator("name:DevTools").all(None, None).await {
                info!("Found {} elements with name 'DevTools'", devtools.len());
            }

            if let Ok(console) = desktop.locator("name:Console").all(None, None).await {
                info!("Found {} elements with name 'Console'", console.len());
            }

            return Err(e);
        }
    }

    info!("🎉 Test completed successfully!");

    Ok(())
}

/// Test to check if we can at least open DevTools
#[tokio::test]
async fn test_open_devtools_only() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();

    info!("🚀 Testing DevTools opening");

    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://example.com", None)?;

    tokio::time::sleep(Duration::from_millis(3000)).await;

    info!("📱 Browser element info:");
    if let Some(name) = browser.name() {
        info!("  Name: {}", name);
    }
    let role = browser.role();
    info!("  Role: {}", role);

    // Try to open DevTools
    info!("⌨️ Pressing Ctrl+Shift+J to open DevTools");
    browser.press_key("{Ctrl}{Shift}J")?;

    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Check what we can find
    info!("🔍 Looking for DevTools elements...");

    // Try different selectors
    let selectors = vec![
        "name:Console prompt",
        "name:Console",
        "name:DevTools",
        "role:document|name:DevTools",
        "role:textbox",
    ];

    for selector in selectors {
        info!("  Trying selector: {}", selector);
        match desktop.locator(selector).first(None).await {
            Ok(element) => {
                info!(
                    "    ✅ Found! Name: {:?}, Role: {:?}",
                    element.name(),
                    element.role()
                );
            }
            Err(e) => {
                info!("    ❌ Not found: {}", e);
            }
        }
    }

    // Close DevTools
    info!("⌨️ Pressing F12 to close DevTools");
    browser.press_key("{F12}")?;

    Ok(())
}
