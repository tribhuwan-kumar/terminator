/// Integration tests for WebView2 HTML extraction with real applications
///
/// These tests demonstrate real-world usage scenarios:
/// 1. Opening web content in different browsers
/// 2. Extracting HTML from various web elements
/// 3. Executing custom JavaScript for data extraction
/// 4. Performance and reliability testing

#[cfg(target_os = "windows")]
mod webview2_integration {
    use std::time::Duration;
    use terminator::{AutomationError, Browser, Desktop};
    use tracing::{debug, info, warn};
    use tracing_subscriber::FmtSubscriber;

    /// Initialize comprehensive tracing for integration tests
    fn init_tracing() {
        let _ = FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
    }

    /// Test complete workflow: open web page, extract HTML, execute JavaScript
    #[tokio::test]
    #[ignore] // Manual test - requires user interaction to verify
    async fn test_complete_webview2_workflow() -> Result<(), Box<dyn std::error::Error>> {
        init_tracing();
        info!("🚀 Starting complete WebView2 workflow test...");

        let desktop = Desktop::new(false, false)?;

        // Test with multiple browsers to see which support WebView2
        let browsers_to_test = vec![("Edge", Browser::Edge), ("Chrome", Browser::Chrome)];

        for (browser_name, browser) in browsers_to_test {
            info!("🌐 Testing with {browser_name}...");

            match test_browser_webview2_support(&desktop, browser_name, browser).await {
                Ok(has_webview2) => {
                    if has_webview2 {
                        info!("✅ {browser_name} supports WebView2 HTML extraction");
                    } else {
                        info!("ℹ️  {browser_name} doesn't use WebView2 (uses text fallback)");
                    }
                }
                Err(e) => {
                    warn!("⚠️  Failed to test {browser_name}: {}", e);
                }
            }

            // Wait between browser tests
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        Ok(())
    }

    /// Test WebView2 support for a specific browser
    async fn test_browser_webview2_support(
        desktop: &Desktop,
        browser_name: &str,
        browser: Browser,
    ) -> Result<bool, AutomationError> {
        // Open a test page with known content
        let test_url = "data:text/html,<html><head><title>WebView2 Test</title></head><body><h1>Hello WebView2!</h1><p id='test'>Test content</p></body></html>";

        let browser_window = desktop.open_url(test_url, Some(browser))?;
        tokio::time::sleep(Duration::from_secs(3)).await; // Wait for load

        // Find document element
        let document = browser_window
            .locator("role:Document")?
            .first(Some(Duration::from_secs(5)))
            .await?;

        debug!("Found document element in {browser_name}");

        // Test 1: JavaScript execution
        let has_js_support = match document.execute_script("document.title")? {
            Some(title) => {
                info!("📜 {browser_name} JS execution: Got title '{title}'");
                true
            }
            None => {
                debug!("📜 {browser_name}: No JavaScript execution support");
                false
            }
        };

        // Test 2: HTML content extraction
        let has_html_support = match document.get_html_content()? {
            Some(html) => {
                let is_real_html = html.contains("<html") && html.contains("<title>");
                if is_real_html {
                    info!(
                        "📄 {browser_name} HTML extraction: Got real HTML ({} chars)",
                        html.len()
                    );
                    debug!("📄 HTML preview: {}", &html[..html.len().min(100)]);
                } else {
                    info!(
                        "📄 {browser_name} HTML extraction: Got text fallback ({} chars)",
                        html.len()
                    );
                }
                is_real_html
            }
            None => {
                debug!("📄 {browser_name}: No HTML content returned");
                false
            }
        };

        // Test 3: Custom JavaScript for data extraction
        if has_js_support {
            let custom_script = "document.getElementById('test') ? document.getElementById('test').textContent : 'Element not found'";
            match document.execute_script(custom_script)? {
                Some(content) => {
                    info!("🎯 {browser_name} custom JS: Got content '{content}'");
                }
                None => {
                    debug!("🎯 {browser_name}: Custom JS returned None");
                }
            }
        }

        // Clean up - close the browser tab/window
        if let Err(e) = browser_window.close() {
            debug!("Failed to close browser window: {}", e);
        }

        Ok(has_js_support && has_html_support)
    }

    /// Test WebView2 performance with large HTML content
    #[tokio::test]
    #[ignore] // Performance test - run manually
    async fn test_webview2_performance() -> Result<(), Box<dyn std::error::Error>> {
        init_tracing();
        info!("⚡ Testing WebView2 performance with large content...");

        let desktop = Desktop::new(false, false)?;

        // Open a content-heavy page
        let test_url = "https://httpbin.org/html";
        let browser = desktop.open_url(test_url, Some(Browser::Edge))?;
        tokio::time::sleep(Duration::from_secs(5)).await; // Wait for full load

        let document = browser
            .locator("role:Document")?
            .first(Some(Duration::from_secs(5)))
            .await?;

        // Performance test: HTML extraction
        let start = std::time::Instant::now();
        let html_result = document.get_html_content()?;
        let html_duration = start.elapsed();

        // Performance test: JavaScript execution
        let start = std::time::Instant::now();
        let js_result = document.execute_script("document.documentElement.outerHTML")?;
        let js_duration = start.elapsed();

        info!("⏱️  HTML extraction took: {:?}", html_duration);
        info!("⏱️  JavaScript execution took: {:?}", js_duration);

        if let Some(html) = html_result {
            info!("📊 HTML extraction: {} characters", html.len());
        }

        if let Some(js_html) = js_result {
            info!("📊 JavaScript extraction: {} characters", js_html.len());
        }

        // Performance should be reasonable (under 1 second for most pages)
        assert!(
            html_duration < Duration::from_secs(1),
            "HTML extraction should be fast"
        );
        assert!(
            js_duration < Duration::from_secs(1),
            "JavaScript execution should be fast"
        );

        Ok(())
    }

    /// Test WebView2 with complex JavaScript operations
    #[tokio::test]
    #[ignore] // Complex test - run manually
    async fn test_webview2_complex_javascript() -> Result<(), Box<dyn std::error::Error>> {
        init_tracing();
        info!("🧠 Testing WebView2 with complex JavaScript operations...");

        let desktop = Desktop::new(false, false)?;

        // Create a test page with complex content
        let complex_html = r#"
        <html>
        <head><title>Complex WebView2 Test</title></head>
        <body>
            <div id="container">
                <ul id="list">
                    <li class="item">Item 1</li>
                    <li class="item">Item 2</li>
                    <li class="item">Item 3</li>
                </ul>
            </div>
            <script>
                function getData() {
                    return {
                        title: document.title,
                        itemCount: document.querySelectorAll('.item').length,
                        items: Array.from(document.querySelectorAll('.item')).map(el => el.textContent)
                    };
                }
            </script>
        </body>
        </html>
        "#;

        let data_url = format!("data:text/html,{}", urlencoding::encode(complex_html));
        let browser = desktop.open_url(&data_url, Some(Browser::Edge))?;
        tokio::time::sleep(Duration::from_secs(3)).await;

        let document = browser
            .locator("role:Document")?
            .first(Some(Duration::from_secs(5)))
            .await?;

        // Test complex JavaScript operations
        let test_scripts = vec![
            ("Get title", "document.title"),
            ("Count elements", "document.querySelectorAll('.item').length"),
            ("Get all text", "Array.from(document.querySelectorAll('.item')).map(el => el.textContent).join(', ')"),
            ("JSON data", "JSON.stringify(typeof getData === 'function' ? getData() : {error: 'Function not available'})"),
        ];

        for (test_name, script) in test_scripts {
            match document.execute_script(script)? {
                Some(result) => {
                    info!("✅ {test_name}: {result}");
                }
                None => {
                    info!("❌ {test_name}: No result (WebView2 not available)");
                }
            }
        }

        Ok(())
    }

    /// Test error scenarios and edge cases
    #[tokio::test]
    #[ignore] // Error testing - run manually
    async fn test_webview2_error_scenarios() -> Result<(), Box<dyn std::error::Error>> {
        init_tracing();
        info!("🔥 Testing WebView2 error scenarios...");

        let desktop = Desktop::new(false, false)?;
        let browser = desktop.open_url("https://httpbin.org/html", Some(Browser::Edge))?;
        tokio::time::sleep(Duration::from_secs(3)).await;

        let document = browser
            .locator("role:Document")?
            .first(Some(Duration::from_secs(5)))
            .await?;

        // Test various error scenarios
        let error_scripts = vec![
            "syntax error!!!",
            "throw new Error('Test error')",
            "undefined.property",
            "nonExistentFunction()",
        ];

        for script in error_scripts {
            match document.execute_script(script)? {
                Some(result) => {
                    info!("🤔 Unexpected success for '{script}': {result}");
                }
                None => {
                    info!("✅ Correctly handled error for: {script}");
                }
            }
        }

        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
mod non_windows_integration {
    #[test]
    fn test_webview2_not_available() {
        println!("WebView2 integration tests are Windows-only");
        // Ensure code compiles on other platforms
    }
}
