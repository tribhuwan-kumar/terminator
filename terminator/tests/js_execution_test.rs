use std::time::Duration;
use terminator::{Browser, Desktop};
use tracing::{debug, info};

#[tokio::test]
async fn test_js_execution_basic() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging for the test
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init()
        .ok(); // Ignore error if already initialized

    info!("🧪 Testing JavaScript execution functionality");

    let desktop = Desktop::new(false, false)?;

    // Open Edge browser with a simple page
    info!("📘 Opening Edge browser...");
    let browser_window = desktop.open_url("https://httpbin.org/html", Some(Browser::Edge))?;

    // Wait for page to load
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Try to find document element
    info!("📄 Looking for document element...");
    let document_locator = browser_window.locator("role:Document")?;
    let document = document_locator.first(Some(Duration::from_secs(5))).await?;

    info!("✅ Found document element");

    // Test 1: Simple JavaScript expression
    info!("🧪 Test 1: Simple JavaScript expression");
    match document.execute_script("'Hello from JS'") {
        Ok(Some(result)) => {
            info!("  ✅ SUCCESS: Script returned: '{}'", result);
            assert_eq!(result, "Hello from JS");
        }
        Ok(None) => {
            info!("  ❌ FAIL: Script returned None (WebView2 not working)");
            panic!("JavaScript execution returned None - not implemented properly");
        }
        Err(e) => {
            info!("  💥 ERROR: Script execution failed: {}", e);
            panic!("JavaScript execution failed: {}", e);
        }
    }

    // Test 2: DOM manipulation
    info!("🧪 Test 2: DOM query");
    match document.execute_script("document.title") {
        Ok(Some(result)) => {
            info!("  ✅ SUCCESS: Page title: '{}'", result);
        }
        Ok(None) => {
            info!("  ❌ FAIL: DOM query returned None");
        }
        Err(e) => {
            info!("  💥 ERROR: DOM query failed: {}", e);
        }
    }

    // Test 3: Math operation
    info!("🧪 Test 3: Math operation");
    match document.execute_script("2 + 3") {
        Ok(Some(result)) => {
            info!("  ✅ SUCCESS: Math result: '{}'", result);
            assert_eq!(result, "5");
        }
        Ok(None) => {
            info!("  ❌ FAIL: Math operation returned None");
        }
        Err(e) => {
            info!("  💥 ERROR: Math operation failed: {}", e);
        }
    }

    // Clean up
    info!("🧹 Closing browser...");
    let _ = browser_window.close();

    info!("✅ JavaScript execution test completed!");
    Ok(())
}

#[tokio::test]
async fn test_html_content_extraction() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init()
        .ok();

    info!("🧪 Testing HTML content extraction");

    let desktop = Desktop::new(false, false)?;
    let browser_window = desktop.open_url("https://httpbin.org/html", Some(Browser::Edge))?;

    tokio::time::sleep(Duration::from_secs(3)).await;

    let document_locator = browser_window.locator("role:Document")?;
    let document = document_locator.first(Some(Duration::from_secs(5))).await?;

    // Test HTML content extraction
    info!("📝 Testing get_html_content...");
    match document.get_html_content() {
        Ok(Some(html)) => {
            info!("  ✅ SUCCESS: HTML extracted ({} chars)", html.len());
            assert!(html.contains("<html") || html.contains("<!DOCTYPE"));
        }
        Ok(None) => {
            info!("  ❌ FAIL: HTML extraction returned None");
            panic!("HTML extraction returned None - not implemented properly");
        }
        Err(e) => {
            info!("  💥 ERROR: HTML extraction failed: {}", e);
            panic!("HTML extraction failed: {}", e);
        }
    }

    let _ = browser_window.close();
    Ok(())
}
