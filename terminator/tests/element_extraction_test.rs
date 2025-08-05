use std::time::Duration;
use terminator::{Browser, Desktop};
use tracing::{debug, info};

#[tokio::test]
async fn test_element_text_extraction() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging for the test
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init()
        .ok(); // Ignore error if already initialized

    info!("🧪 Testing element text extraction from webpage");

    let desktop = Desktop::new(false, false)?;

    // Open Edge browser with httpbin.org/html which has known content structure
    info!("📘 Opening Edge browser with test page...");
    let browser_window = desktop.open_url("https://httpbin.org/html", Some(Browser::Edge))?;

    // Wait for page to load
    tokio::time::sleep(Duration::from_secs(4)).await;

    // Try to find document element first
    info!("📄 Looking for document element...");
    let document_locator = browser_window.locator("role:Document")?;
    let document = document_locator.first(Some(Duration::from_secs(5))).await?;

    info!("✅ Found document element");

    // Test 1: Extract title element text
    info!("🧪 Test 1: Extract heading/title text");
    match document.locator("role:Heading") {
        Ok(heading_locator) => match heading_locator.first(Some(Duration::from_secs(3))).await {
            Ok(heading) => {
                if let Some(heading_text) = heading.name() {
                    info!("  ✅ SUCCESS: Found heading text: '{}'", heading_text);
                } else {
                    info!("  ⚠️ Heading found but no text content");
                }
            }
            Err(e) => {
                info!("  ❌ Could not find heading element: {}", e);
            }
        },
        Err(e) => {
            info!("  ❌ Could not create heading locator: {}", e);
        }
    }

    // Test 2: Extract paragraph text
    info!("🧪 Test 2: Extract paragraph text");
    match document.locator("role:Text") {
        Ok(text_locator) => match text_locator.all(Some(Duration::from_secs(3)), None).await {
            Ok(text_elements) => {
                info!("  ✅ Found {} text elements", text_elements.len());
                for (i, text_elem) in text_elements.iter().take(3).enumerate() {
                    if let Some(text_content) = text_elem.name() {
                        let preview = if text_content.len() > 50 {
                            format!("{}...", &text_content[..50])
                        } else {
                            text_content
                        };
                        info!("    Text {}: '{}'", i + 1, preview);
                    }
                }
            }
            Err(e) => {
                info!("  ❌ Could not find text elements: {}", e);
            }
        },
        Err(e) => {
            info!("  ❌ Could not create text locator: {}", e);
        }
    }

    // Test 3: Try to extract specific element by automation ID or class
    info!("🧪 Test 3: Extract specific element content");
    // Try different selectors for content extraction
    let test_selectors = ["role:Document", "role:Pane", "role:Group", "role:Text"];

    for selector in &test_selectors {
        info!("  🔎 Trying selector: {}", selector);
        match document.locator(*selector) {
            Ok(locator) => {
                match locator.first(Some(Duration::from_secs(2))).await {
                    Ok(element) => {
                        if let Some(content) = element.name() {
                            if !content.trim().is_empty() {
                                let preview = if content.len() > 100 {
                                    format!("{}...", &content[..100])
                                } else {
                                    content
                                };
                                info!("    ✅ Content from {}: '{}'", selector, preview);
                                break; // Found content, stop looking
                            }
                        }
                    }
                    Err(_) => {
                        debug!("    No element found for {}", selector);
                    }
                }
            }
            Err(_) => {
                debug!("    Could not create locator for {}", selector);
            }
        }
    }

    // Test 4: Use JavaScript to extract element content (if available)
    info!("🧪 Test 4: Extract content using JavaScript");
    match document
        .execute_browser_script(
            "document.body.innerText || document.body.textContent || 'No text found'",
        )
        .await
    {
        Ok(body_text) => {
            let preview = if body_text.len() > 200 {
                format!("{}...", &body_text[..200])
            } else {
                body_text
            };
            info!("  ✅ SUCCESS: Body text via JS: '{}'", preview);
        }
        Err(e) => {
            info!("  💥 Browser script execution failed: {}", e);
        }
    }

    // Test 5: Try to extract HTML content using browser script
    info!("🧪 Test 5: Extract HTML content using browser script");
    match document
        .execute_browser_script("document.documentElement.outerHTML")
        .await
    {
        Ok(html) => {
            let preview = if html.len() > 300 {
                format!("{}...", &html[..300])
            } else {
                html
            };
            info!(
                "  ✅ SUCCESS: HTML content extracted via browser script ({} chars): '{}'",
                preview.len(),
                preview
            );
        }
        Err(e) => {
            info!(
                "  💥 HTML content extraction via browser script failed: {}",
                e
            );
        }
    }

    // Clean up
    info!("🧹 Closing browser...");
    let _ = browser_window.close();

    info!("✅ Element extraction test completed!");
    Ok(())
}

#[tokio::test]
async fn test_specific_element_subtree() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init()
        .ok();

    info!("🧪 Testing specific element subtree extraction");

    let desktop = Desktop::new(false, false)?;
    let browser_window = desktop.open_url("https://httpbin.org/html", Some(Browser::Edge))?;

    tokio::time::sleep(Duration::from_secs(3)).await;

    let document_locator = browser_window.locator("role:Document")?;
    let document = document_locator.first(Some(Duration::from_secs(5))).await?;

    // Test: Find elements and extract their complete text subtree
    info!("🔍 Searching for elements with substantial content...");

    // Try to find any element that contains text content
    match document.locator("role:Text") {
        Ok(locator) => {
            match locator.all(Some(Duration::from_secs(3)), None).await {
                Ok(elements) => {
                    info!(
                        "Found {} text elements, extracting subtree content:",
                        elements.len()
                    );

                    for (i, element) in elements.iter().enumerate() {
                        if let Some(text) = element.name() {
                            if text.trim().len() > 10 {
                                // Only show substantial content
                                info!("  📄 Element {}: '{}'", i + 1, text);

                                // Try to get children if possible
                                // Note: This depends on the automation API's ability to traverse
                                debug!("    Attempting to extract child elements...");
                            }
                        }
                    }
                }
                Err(e) => {
                    info!("Could not find text elements: {}", e);
                }
            }
        }
        Err(e) => {
            info!("Could not create text locator: {}", e);
        }
    }

    let _ = browser_window.close();
    Ok(())
}
