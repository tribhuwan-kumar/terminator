use std::time::Duration;
use terminator::{AutomationError, Desktop};
use tracing::info;

#[tokio::test]
async fn test_browser_script_execution() -> Result<(), AutomationError> {
    // Initialize tracing for test output
    let _ = tracing_subscriber::fmt::try_init();

    info!("🚀 Starting browser script integration test");

    // Create desktop instance
    let desktop = Desktop::new(true, false)?;

    // Use terminator's built-in browser navigation
    info!("🌐 Opening browser and navigating to Wikipedia...");
    let browser_element = desktop.open_url(
        "https://en.wikipedia.org/wiki/Rust_(programming_language)",
        None,
    )?;

    info!("✅ Browser opened");

    // Wait for page to load completely
    tokio::time::sleep(Duration::from_millis(5000)).await;

    info!(
        "✅ Found browser element: {}",
        browser_element.name().unwrap_or_default()
    );

    // Test: Simple text extraction
    info!("📝 Testing: Extract page title");
    let title_script = "document.title";

    let result = browser_element.execute_browser_script(title_script).await?;
    assert!(!result.is_empty(), "Page title should not be empty");
    assert!(result.contains("Rust"), "Title should contain 'Rust'");
    info!("✅ SUCCESS - Page title: {}", result);

    // Test: Extract specific element
    info!("📝 Testing: Extract main heading");
    let heading_script = r#"
        const heading = document.querySelector('h1');
        heading ? heading.textContent.trim() : 'No heading found'
    "#;

    let result = browser_element
        .execute_browser_script(heading_script)
        .await?;
    assert!(!result.is_empty(), "Heading should not be empty");
    assert!(result.contains("Rust"), "Heading should contain 'Rust'");
    info!("✅ SUCCESS - Main heading: {}", result);

    // Test: Error handling
    info!("📝 Testing: Error handling");
    let error_script = "throw new Error('Test error message');";

    let result = browser_element.execute_browser_script(error_script).await?;
    assert!(
        result.contains("ERROR: Test error message"),
        "Should handle errors correctly"
    );
    info!("✅ SUCCESS - Error handling works: {}", result);

    // Test: JSON object extraction
    info!("📝 Testing: JSON object extraction");
    let json_script = r#"
        ({
            title: document.title,
            url: window.location.href,
            hasContent: document.body.children.length > 0
        })
    "#;

    let result = browser_element.execute_browser_script(json_script).await?;
    assert!(result.contains("\"title\""), "Should contain title field");
    assert!(result.contains("\"url\""), "Should contain url field");
    assert!(
        result.contains("\"hasContent\":true"),
        "Should have content"
    );
    info!(
        "✅ SUCCESS - JSON extraction: {}",
        &result[..result.len().min(200)]
    );

    info!("🎉 All browser script tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_browser_script_simple() -> Result<(), AutomationError> {
    let _ = tracing_subscriber::fmt::try_init();

    info!("🚀 Starting simple browser script test");

    let desktop = Desktop::new(true, false)?;

    // Use a simpler page for faster testing
    let browser_element = desktop.open_url("https://httpbin.org/html", None)?;

    // Wait for page to load
    tokio::time::sleep(Duration::from_millis(3000)).await;

    // Simple test: get page title
    let result = browser_element
        .execute_browser_script("document.title")
        .await?;
    assert!(!result.is_empty(), "Should have a title");

    info!("✅ Simple test passed - Title: {}", result);
    Ok(())
}
