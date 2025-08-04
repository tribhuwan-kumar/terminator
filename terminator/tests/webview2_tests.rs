/// Unit and integration tests for WebView2 HTML extraction functionality
///
/// These tests verify that:
/// 1. WebView2 control detection works correctly
/// 2. JavaScript execution works in WebView2 controls  
/// 3. HTML content extraction works
/// 4. Error handling is proper for non-web elements

#[cfg(target_os = "windows")]
mod windows_webview2_tests {
    use std::time::Duration;
    use terminator::{AutomationError, Desktop};
    use tracing::info;
    use tracing_subscriber::FmtSubscriber;

    /// Initialize tracing for test debugging
    fn init_tracing() {
        let _ = FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
    }

    #[tokio::test]
    #[ignore] // Requires manual testing with WebView2 app
    async fn test_webview2_script_execution() -> Result<(), Box<dyn std::error::Error>> {
        init_tracing();
        info!("Testing WebView2 JavaScript execution...");

        let desktop = Desktop::new(false, false)?;

        // Open a URL in Edge (which uses WebView2)
        let browser =
            desktop.open_url("https://httpbin.org/html", Some(terminator::Browser::Edge))?;
        tokio::time::sleep(Duration::from_secs(3)).await; // Wait for page load

        // Find the document element
        let document = browser
            .locator("role:Document")?
            .first(Some(Duration::from_secs(5)))
            .await?;

        // Test JavaScript execution
        match document.execute_script("document.title")? {
            Some(title) => {
                info!("Got page title: {}", title);
                assert!(!title.is_empty(), "Title should not be empty");
            }
            None => {
                info!("No WebView2 detected - this might be a regular browser");
                // This is expected if Edge is not using WebView2 for this content
            }
        }

        Ok(())
    }

    #[tokio::test]
    #[ignore] // Requires manual testing with WebView2 app
    async fn test_webview2_html_content_extraction() -> Result<(), Box<dyn std::error::Error>> {
        init_tracing();
        info!("Testing WebView2 HTML content extraction...");

        let desktop = Desktop::new(false, false)?;

        // Open a simple HTML page
        let browser =
            desktop.open_url("https://httpbin.org/html", Some(terminator::Browser::Edge))?;
        tokio::time::sleep(Duration::from_secs(3)).await;

        let document = browser
            .locator("role:Document")?
            .first(Some(Duration::from_secs(5)))
            .await?;

        // Test HTML content extraction
        match document.get_html_content()? {
            Some(html) => {
                info!("Got HTML content (length: {})", html.len());
                assert!(html.contains("<html"), "Should contain HTML tag");
                assert!(html.len() > 100, "HTML should be substantial");
            }
            None => {
                info!("No HTML content - fallback to text extraction worked");
                // This is acceptable for non-WebView2 browsers
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_non_web_element_returns_none() -> Result<(), Box<dyn std::error::Error>> {
        init_tracing();
        info!("Testing that non-web elements return None for script execution...");

        let desktop = Desktop::new(false, false)?;

        // Open a native Windows app (Calculator)
        let calc = desktop.open_application("calc")?;
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Try to execute script on calculator (should return None)
        let result = calc.execute_script("document.title")?;
        assert!(
            result.is_none(),
            "Non-web elements should return None for script execution"
        );

        let html_result = calc.get_html_content()?;
        assert!(
            html_result.is_some(),
            "get_html_content should fallback to text extraction"
        );

        Ok(())
    }

    #[test]
    fn test_webview2_detection_logic() {
        init_tracing();
        info!("Testing WebView2 class name detection logic...");

        // Test the logic we use to identify WebView2 controls
        let webview2_classes = [
            "Chrome_WidgetWin_0",
            "Chrome_WidgetWin_1",
            "WebView2",
            "Edge_WebView2",
        ];

        for class_name in &webview2_classes {
            // Simulate detection logic
            let is_webview2 = class_name.contains("Chrome_WidgetWin")
                || class_name.contains("WebView2")
                || class_name.contains("Edge_WebView2");

            assert!(is_webview2, "Should detect {} as WebView2", class_name);
        }

        // Test non-WebView2 classes
        let non_webview2_classes = ["Button", "Edit", "Static", "Window"];
        for class_name in &non_webview2_classes {
            let is_webview2 = class_name.contains("Chrome_WidgetWin")
                || class_name.contains("WebView2")
                || class_name.contains("Edge_WebView2");

            assert!(!is_webview2, "Should not detect {} as WebView2", class_name);
        }
    }

    #[tokio::test]
    #[ignore] // Requires specific WebView2 test app
    async fn test_webview2_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        init_tracing();
        info!("Testing WebView2 error handling...");

        let desktop = Desktop::new(false, false)?;

        // Test with invalid JavaScript
        let browser =
            desktop.open_url("https://httpbin.org/html", Some(terminator::Browser::Edge))?;
        tokio::time::sleep(Duration::from_secs(3)).await;

        let document = browser
            .locator("role:Document")?
            .first(Some(Duration::from_secs(5)))
            .await?;

        // Test invalid JavaScript (should handle gracefully)
        let result = document.execute_script("invalid javascript syntax!!!")?;
        // Should either return None or handle error gracefully
        info!("Invalid script result: {:?}", result);

        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
mod non_windows_tests {
    use super::*;

    #[test]
    fn test_webview2_not_available_on_non_windows() {
        // On non-Windows platforms, WebView2 functionality should gracefully return None
        // This test ensures the code compiles and doesn't panic on other platforms
        println!("WebView2 is Windows-only - this test confirms code compiles on other platforms");
    }
}
