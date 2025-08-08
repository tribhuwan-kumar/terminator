use std::time::Duration;
use terminator::{AutomationError, Desktop};
use tracing::info;

/// Basic smoke test for browser script execution
#[tokio::test]
async fn test_browser_script_basic() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();

    info!("🚀 Starting browser script basic test");

    // Create desktop and open browser
    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://example.com", None)?;

    // Give page time to load
    tokio::time::sleep(Duration::from_millis(3000)).await;

    // Test 1: Simple arithmetic
    info!("📝 Test 1: Simple arithmetic");
    let result = browser.execute_browser_script("5 + 3").await?;
    info!("Result: '{}'", result);
    assert_eq!(result.trim(), "8", "5 + 3 should equal 8");

    // Test 2: String operation
    info!("📝 Test 2: String concatenation");
    let result = browser.execute_browser_script("'test' + '123'").await?;
    info!("Result: '{}'", result);
    assert_eq!(result.trim(), "test123", "String concatenation failed");

    // Test 3: Boolean
    info!("📝 Test 3: Boolean expression");
    let result = browser.execute_browser_script("10 > 5").await?;
    info!("Result: '{}'", result);
    assert_eq!(result.trim(), "true", "Boolean comparison failed");

    // Test 4: Array length
    info!("📝 Test 4: Array operations");
    let result = browser
        .execute_browser_script("[1, 2, 3, 4, 5].length")
        .await?;
    info!("Result: '{}'", result);
    assert_eq!(result.trim(), "5", "Array length should be 5");

    // Test 5: Math operations
    info!("📝 Test 5: Math operations");
    let result = browser
        .execute_browser_script("Math.max(10, 20, 5)")
        .await?;
    info!("Result: '{}'", result);
    assert_eq!(result.trim(), "20", "Math.max should return 20");

    info!("✅ All basic tests passed!");

    Ok(())
}

/// Test async/promise functionality
#[tokio::test]
async fn test_browser_script_async() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();

    info!("🚀 Starting browser script async test");

    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://example.com", None)?;
    tokio::time::sleep(Duration::from_millis(3000)).await;

    // Test immediate promise
    info!("📝 Testing immediate promise resolution");
    let script = "Promise.resolve('immediate')";
    let result = browser.execute_browser_script(script).await?;
    info!("Result: '{}'", result);
    assert_eq!(
        result.trim(),
        "immediate",
        "Promise should resolve to 'immediate'"
    );

    // Test delayed promise (2 seconds)
    info!("📝 Testing delayed promise (2 seconds)");
    let script = r#"
        new Promise(resolve => {
            setTimeout(() => resolve('delayed-2s'), 2000);
        })
    "#;

    let start = std::time::Instant::now();
    let result = browser.execute_browser_script(script).await?;
    let elapsed = start.elapsed();

    info!("Result: '{}' (took {:.1}s)", result, elapsed.as_secs_f32());
    assert_eq!(
        result.trim(),
        "delayed-2s",
        "Promise should resolve to 'delayed-2s'"
    );
    assert!(elapsed.as_secs() >= 2, "Should take at least 2 seconds");

    info!("✅ Async tests passed!");

    Ok(())
}

/// Test error handling
#[tokio::test]
async fn test_browser_script_errors() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();

    info!("🚀 Starting browser script error test");

    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://example.com", None)?;
    tokio::time::sleep(Duration::from_millis(3000)).await;

    // Test syntax error
    info!("📝 Testing syntax error handling");
    let result = browser
        .execute_browser_script("this is not valid JS")
        .await?;
    info!("Result: '{}'", result);
    assert!(
        result.starts_with("ERROR:"),
        "Should return ERROR for invalid syntax"
    );

    // Test undefined variable
    info!("📝 Testing undefined variable error");
    let result = browser.execute_browser_script("undefinedVar").await?;
    info!("Result: '{}'", result);
    // Note: undefined variables might just return "undefined" rather than error
    assert!(
        result.trim() == "undefined" || result.starts_with("ERROR:"),
        "Should handle undefined variable"
    );

    info!("✅ Error handling tests passed!");

    Ok(())
}

/// Test DOM access
#[tokio::test]
async fn test_browser_script_dom() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();

    info!("🚀 Starting browser script DOM test");

    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://example.com", None)?;
    tokio::time::sleep(Duration::from_millis(3000)).await;

    // Test document.title
    info!("📝 Testing document.title");
    let result = browser.execute_browser_script("document.title").await?;
    info!("Result: '{}'", result);
    assert!(result.contains("Example"), "Title should contain 'Example'");

    // Test querySelector
    info!("📝 Testing querySelector for h1");
    let result = browser
        .execute_browser_script("document.querySelector('h1') ? 'found' : 'not found'")
        .await?;
    info!("Result: '{}'", result);
    assert_eq!(result.trim(), "found", "Should find h1 element");

    // Test counting elements
    info!("📝 Testing element counting");
    let result = browser
        .execute_browser_script("document.querySelectorAll('*').length > 0")
        .await?;
    info!("Result: '{}'", result);
    assert_eq!(result.trim(), "true", "Should find elements on page");

    info!("✅ DOM tests passed!");

    Ok(())
}

/// Test with longer running script (20+ seconds to test heartbeat)
#[tokio::test]
#[ignore] // Ignore by default as it takes time
async fn test_browser_script_long_running() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();

    info!("🚀 Starting long-running script test (20 seconds)");

    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://example.com", None)?;
    tokio::time::sleep(Duration::from_millis(3000)).await;

    // Script that runs for 20 seconds (will trigger heartbeat)
    let script = r#"
        new Promise(resolve => {
            const start = Date.now();
            const checkInterval = setInterval(() => {
                const elapsed = Date.now() - start;
                console.log('Elapsed: ' + elapsed + 'ms');
                if (elapsed >= 20000) {
                    clearInterval(checkInterval);
                    resolve('Completed after 20 seconds');
                }
            }, 1000);
        })
    "#;

    info!("📝 Starting 20-second script...");
    let start = std::time::Instant::now();
    let result = browser.execute_browser_script(script).await?;
    let elapsed = start.elapsed();

    info!("Result: '{}' (took {:.1}s)", result, elapsed.as_secs_f32());
    assert!(result.contains("Completed after 20 seconds"));
    assert!(elapsed.as_secs() >= 19, "Should take at least 19 seconds");
    assert!(
        elapsed.as_secs() <= 25,
        "Should not take more than 25 seconds"
    );

    info!("✅ Long-running test with heartbeats passed!");

    Ok(())
}
