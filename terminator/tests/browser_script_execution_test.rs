use std::time::Duration;
use terminator::{AutomationError, Desktop};
use tracing::info;

#[tokio::test]
async fn test_simple_sync_script() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();
    info!("🧪 Testing simple synchronous script");

    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://example.com", None)?;
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Test simple math
    let result = browser.execute_browser_script("2 + 2").await?;
    assert_eq!(result.trim(), "4");

    // Test string concatenation
    let result = browser
        .execute_browser_script("'Hello' + ' ' + 'World'")
        .await?;
    assert_eq!(result.trim(), "Hello World");

    // Test object return
    let result = browser
        .execute_browser_script("({name: 'test', value: 42})")
        .await?;
    info!("Object result: {}", result);
    assert!(result.contains("test"));
    assert!(result.contains("42"));

    Ok(())
}

#[tokio::test]
async fn test_async_promise_script() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();
    info!("🧪 Testing async/promise-based script");

    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://example.com", None)?;
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Test immediate promise resolution
    let script = r#"
        Promise.resolve('Resolved immediately')
    "#;
    let result = browser.execute_browser_script(script).await?;
    assert_eq!(result.trim(), "Resolved immediately");

    // Test delayed promise (1 second)
    let script = r#"
        new Promise(resolve => {
            setTimeout(() => resolve('Resolved after 1 second'), 1000);
        })
    "#;
    let start = std::time::Instant::now();
    let result = browser.execute_browser_script(script).await?;
    let elapsed = start.elapsed();

    assert_eq!(result.trim(), "Resolved after 1 second");
    assert!(elapsed.as_secs() >= 1, "Should take at least 1 second");
    assert!(elapsed.as_secs() < 3, "Should not take more than 3 seconds");

    Ok(())
}

#[tokio::test]
async fn test_long_running_with_heartbeat() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();
    info!("🧪 Testing long-running script with heartbeats");

    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://example.com", None)?;
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Test script that runs for 20 seconds (should trigger heartbeat)
    let script = r#"
        new Promise(resolve => {
            const start = Date.now();
            const interval = setInterval(() => {
                if (Date.now() - start >= 20000) {
                    clearInterval(interval);
                    resolve('Completed after 20 seconds');
                }
            }, 100);
        })
    "#;

    let start = std::time::Instant::now();
    let result = browser.execute_browser_script(script).await?;
    let elapsed = start.elapsed();

    assert!(result.contains("Completed after 20 seconds"));
    assert!(elapsed.as_secs() >= 20, "Should take at least 20 seconds");
    assert!(
        elapsed.as_secs() < 25,
        "Should not take more than 25 seconds"
    );

    info!("✅ Script completed with heartbeats working");

    Ok(())
}

#[tokio::test]
async fn test_script_error_handling() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();
    info!("🧪 Testing script error handling");

    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://example.com", None)?;
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Test syntax error
    let result = browser
        .execute_browser_script("this is not valid javascript")
        .await?;
    assert!(
        result.starts_with("ERROR:"),
        "Should return error for invalid syntax"
    );

    // Test runtime error
    let result = browser
        .execute_browser_script("undefinedVariable.someMethod()")
        .await?;
    assert!(
        result.starts_with("ERROR:"),
        "Should return error for runtime error"
    );

    // Test thrown error
    let result = browser
        .execute_browser_script("throw new Error('Custom error')")
        .await?;
    assert!(
        result.contains("Custom error"),
        "Should contain the error message"
    );

    Ok(())
}

#[tokio::test]
async fn test_dom_manipulation() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();
    info!("🧪 Testing DOM manipulation scripts");

    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://example.com", None)?;
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Test getting page title
    let result = browser.execute_browser_script("document.title").await?;
    assert!(result.contains("Example Domain"));

    // Test counting elements
    let result = browser
        .execute_browser_script("document.querySelectorAll('p').length")
        .await?;
    let count: i32 = result.trim().parse().expect("Should be a number");
    assert!(count > 0, "Should find at least one paragraph");

    // Test getting text content
    let result = browser
        .execute_browser_script("document.querySelector('h1')?.textContent")
        .await?;
    assert!(result.contains("Example Domain"));

    Ok(())
}

#[tokio::test]
async fn test_fetch_api() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();
    info!("🧪 Testing fetch API in browser");

    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://httpbin.org", None)?;
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Test fetching JSON data
    let script = r#"
        fetch('https://httpbin.org/json')
            .then(response => response.json())
            .then(data => JSON.stringify(data))
    "#;

    let result = browser.execute_browser_script(script).await?;
    assert!(
        result.contains("slideshow"),
        "Should contain JSON data from httpbin"
    );

    Ok(())
}

#[tokio::test]
async fn test_multiple_sequential_scripts() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();
    info!("🧪 Testing multiple sequential script executions");

    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://example.com", None)?;
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Set a variable
    let result = browser
        .execute_browser_script("window.testVar = 42; window.testVar")
        .await?;
    assert_eq!(result.trim(), "42");

    // Read the variable in next script
    let result = browser.execute_browser_script("window.testVar * 2").await?;
    assert_eq!(result.trim(), "84");

    // Modify and read again
    let result = browser
        .execute_browser_script("window.testVar = 'changed'; window.testVar")
        .await?;
    assert_eq!(result.trim(), "changed");

    Ok(())
}

#[tokio::test]
#[ignore] // This test is for manual verification of timeout behavior
async fn test_timeout_scenario() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();
    info!("🧪 Testing timeout scenario (this will take time)");

    let desktop = Desktop::new(true, false)?;
    let browser = desktop.open_url("https://example.com", None)?;
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Script that would run forever without completing
    // This should timeout after 5 minutes or heartbeat timeout
    let script = r#"
        new Promise(() => {
            // Never resolves - will trigger timeout
            setInterval(() => {
                console.log('Still running...');
            }, 1000);
        })
    "#;

    let result = browser.execute_browser_script(script).await;

    // Should get a timeout error
    assert!(result.is_err(), "Should timeout");
    if let Err(AutomationError::Timeout(msg)) = result {
        info!("Got expected timeout: {}", msg);
        assert!(msg.contains("timed out"), "Should contain timeout message");
    }

    Ok(())
}
