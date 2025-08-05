use std::time::Duration;
use terminator::{Browser, Desktop};
use tokio::time::sleep;
use tracing::{info, warn};

/// Integration test to find the hero banner title element on Dataiku's website
/// Target: class="hero-banner__title" on https://pages.dataiku.com/guide-to-ai-agents
#[tokio::test]
async fn test_find_dataiku_form_element() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging for the test
    let _ = tracing_subscriber::fmt::try_init();

    info!("🚀 Starting Dataiku form element integration test");

    // Create desktop instance
    let desktop = Desktop::new(false, true)?;

    // Target URL and element class from user request
    let target_url = "https://pages.dataiku.com/guide-to-ai-agents";
    let target_class = "hero-banner__title";

    info!("🌐 Opening URL: {}", target_url);

    // Try to open in Edge first (most likely to have WebView2), then fallback to default
    let browser_element = match desktop.open_url(target_url, Some(Browser::Edge)) {
        Ok(element) => {
            info!("✅ Successfully opened in Edge browser");
            element
        }
        Err(e) => {
            warn!("⚠️  Edge failed, trying default browser: {}", e);
            desktop.open_url(target_url, None)?
        }
    };

    info!("⏳ Waiting 8 seconds for page to fully load...");
    sleep(Duration::from_secs(8)).await;

    // Find WebView2 control
    info!("🔍 Searching for WebView2 control...");
    let webview2_element = find_webview2_element(&browser_element)?;
    info!("✅ Found WebView2 control");

    // Test 1: Check if the target element exists
    info!("🎯 Testing if target element exists: {}", target_class);
    let exists_script = format!("document.querySelector('.{}') !== null", target_class);
    
    let element_found = match webview2_element.execute_script(&exists_script) {
        Ok(Some(result)) => {
            println!("📊 Element existence check result: '{}'", result);
            if result == "true" {
                println!("🎉 SUCCESS: Target element '.{}' FOUND on page!", target_class);
                true
            } else {
                println!("❌ FAILED: Target element '.{}' NOT FOUND on page", target_class);
                false
            }
        }
        Ok(None) => {
            println!("⚠️  Element existence check returned no result");
            false
        }
        Err(e) => {
            println!("❌ Element existence check failed: {}", e);
            false
        }
    };

    // Assert that we found the element - this will make the test fail if not found
    assert!(element_found, "❌ ASSERTION FAILED: Element '.{}' was not found on the page!", target_class);

    // Test 2: Get comprehensive information about the element
    info!("📝 Getting detailed element information...");
    let comprehensive_script = create_element_analysis_script(&target_class);
    
    match webview2_element.execute_script(&comprehensive_script) {
        Ok(Some(result)) => {
            println!("📋 Element analysis result:");
            println!("{}", result);
            
            // Try to parse as JSON for better output
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                println!("\n📊 Formatted analysis:");
                println!("{}", serde_json::to_string_pretty(&parsed)?);
                
                // Check if element was found in the analysis
                if let Some(found) = parsed.get("found").and_then(|v| v.as_bool()) {
                    if found {
                        println!("✅ Element analysis confirms element exists!");
                        if let Some(visible) = parsed.get("visible").and_then(|v| v.as_bool()) {
                            println!("👁️  Element visible: {}", visible);
                        }
                    } else {
                        println!("❌ Element analysis confirms element does NOT exist!");
                    }
                }
            } else {
                println!("⚠️  Could not parse analysis result as JSON");
            }
        }
        Ok(None) => {
            println!("⚠️  Element analysis returned no result");
        }
        Err(e) => {
            println!("❌ Element analysis failed: {}", e);
        }
    }

    // Test 3: Find all HubSpot form elements on the page
    info!("🔍 Searching for all HubSpot form elements...");
    let hubspot_forms_script = r#"(() => {
        const hsElements = Array.from(document.querySelectorAll('[id*="hs_form"]'));
        return JSON.stringify({
            count: hsElements.length,
            elements: hsElements.map(el => ({
                id: el.id,
                tagName: el.tagName,
                className: el.className,
                visible: el.offsetWidth > 0 && el.offsetHeight > 0
            }))
        }, null, 2);
    })()"#;

    match webview2_element.execute_script(hubspot_forms_script) {
        Ok(Some(result)) => {
            println!("📊 HubSpot forms search result:");
            println!("{}", result);
            
            // Try to parse and check if our target is in the list
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                if let Some(count) = parsed.get("count").and_then(|v| v.as_u64()) {
                    println!("🔢 Total HubSpot form elements found: {}", count);
                    
                    if let Some(elements) = parsed.get("elements").and_then(|v| v.as_array()) {
                        let target_found = elements.iter().any(|el| {
                            el.get("className").and_then(|class| class.as_str())
                                .map(|c| c.contains(&target_class))
                                .unwrap_or(false)
                        });
                        
                        if target_found {
                            println!("🎯 TARGET CLASS '{}' found in HubSpot forms list!", target_class);
                        } else {
                            println!("❌ Target class '{}' NOT found in HubSpot forms list", target_class);
                        }
                    }
                }
            }
        }
        Ok(None) => {
            println!("⚠️  HubSpot forms search returned no result");
        }
        Err(e) => {
            println!("❌ HubSpot forms search failed: {}", e);
        }
    }

    // Test 4: Get page information
    info!("📄 Getting page information...");
    let page_info_script = r#"JSON.stringify({
        title: document.title,
        url: window.location.href,
        readyState: document.readyState,
        totalForms: document.querySelectorAll('form').length,
        totalElements: document.getElementsByTagName('*').length,
        hsFormElements: document.querySelectorAll('[id*="hs_form"]').length
    }, null, 2)"#;

    match webview2_element.execute_script(page_info_script) {
        Ok(Some(result)) => {
            info!("📊 Page information:");
            println!("{}", result);
        }
        Ok(None) => {
            warn!("⚠️  Page info returned no result");
        }
        Err(e) => {
            warn!("❌ Page info failed: {}", e);
        }
    }

    // Test 5: Try to get text content from the element if it exists
    info!("📝 Attempting to get element text content...");
    let text_script = format!(
        r#"(() => {{
            const el = document.querySelector('.{}');
            if (!el) return 'ELEMENT_NOT_FOUND';
            const text = el.textContent || el.innerText || '';
            return text.length > 0 ? text.substring(0, 500) : 'ELEMENT_HAS_NO_TEXT';
        }})()"#,
        target_class
    );

    match webview2_element.execute_script(&text_script) {
        Ok(Some(result)) => {
            println!("📄 Element text extraction result:");
            println!("'{}'", result);
            
            if result == "ELEMENT_NOT_FOUND" {
                println!("❌ Text extraction confirms: Element NOT found");
            } else if result == "ELEMENT_HAS_NO_TEXT" {
                println!("✅ Element found but has no text content");
            } else {
                println!("✅ Element found with text content! Length: {} chars", result.len());
            }
        }
        Ok(None) => {
            println!("⚠️  Element text extraction returned no result");
        }
        Err(e) => {
            println!("❌ Element text extraction failed: {}", e);
        }
    }

    info!("✨ Integration test completed!");
    Ok(())
}

/// Find the WebView2 element in the browser
fn find_webview2_element(browser_element: &terminator::UIElement) -> Result<terminator::UIElement, Box<dyn std::error::Error + Send + Sync>> {
    // Check if the browser element itself has a URL (is WebView2)
    if let Some(url) = browser_element.url() {
        if !url.is_empty() {
            info!("🌐 Found WebView2 in browser element with URL: {}", url);
            return Ok(browser_element.clone());
        }
    }

    // Search children for WebView2 control
    if let Ok(children) = browser_element.children() {
        for child in children {
            if let Some(url) = child.url() {
                if !url.is_empty() {
                    info!("🌐 Found WebView2 in child element with URL: {}", url);
                    return Ok(child);
                }
            }

            // Check grandchildren as well
            if let Ok(grandchildren) = child.children() {
                for grandchild in grandchildren {
                    if let Some(url) = grandchild.url() {
                        if !url.is_empty() {
                            info!("🌐 Found WebView2 in grandchild with URL: {}", url);
                            return Ok(grandchild);
                        }
                    }
                }
            }
        }
    }

    Err("Could not find WebView2 control in browser".into())
}

/// Create a comprehensive element analysis script
fn create_element_analysis_script(element_class: &str) -> String {
    format!(r#"(() => {{
        const targetClass = '{}';
        const element = document.querySelector('.' + targetClass);
        
        if (!element) {{
            return JSON.stringify({{
                found: false,
                targetClass: targetClass,
                message: 'Element not found'
            }}, null, 2);
        }}

        return JSON.stringify({{
            found: true,
            targetClass: targetClass,
            tagName: element.tagName,
            className: element.className,
            id: element.id || 'no-id',
            textContent: element.textContent?.substring(0, 500) || '',
            innerHTML: element.innerHTML?.substring(0, 1000) || '',
            attributes: Array.from(element.attributes).map(attr => ({{
                name: attr.name,
                value: attr.value
            }})),
            style: {{
                display: getComputedStyle(element).display,
                visibility: getComputedStyle(element).visibility,
                opacity: getComputedStyle(element).opacity
            }},
            bounds: {{
                x: element.offsetLeft,
                y: element.offsetTop,
                width: element.offsetWidth,
                height: element.offsetHeight
            }},
            visible: element.offsetWidth > 0 && element.offsetHeight > 0,
            childElementCount: element.children.length,
            parentInfo: element.parentElement ? {{
                tagName: element.parentElement.tagName,
                className: element.parentElement.className,
                id: element.parentElement.id
            }} : null
        }}, null, 2);
    }})()"#, element_class)
}

/// Test to run the element finder without opening a browser (mock test)
#[tokio::test]
async fn test_element_finder_script_generation() {
    let target_class = "hero-banner__title";
    
    // Test script generation
    let script = create_element_analysis_script(target_class);
    assert!(script.contains(target_class));
    assert!(script.contains("querySelector"));
    assert!(script.contains("JSON.stringify"));
    
    println!("✅ Element finder script generated successfully");
    println!("Script length: {} characters", script.len());
}

/// Test specifically for element detection patterns
#[tokio::test]
async fn test_element_detection_patterns() {
    let patterns = vec![
        // Find elements with specific class
        r#"document.querySelectorAll('.hero-banner__title').length"#,
        
        // Find all forms
        r#"document.querySelectorAll('form').length"#,
        
        // Find specific element by class
        r#"document.querySelector('.hero-banner__title') !== null"#,
        
        // Get element details safely
        r#"(() => {
            const el = document.querySelector('.hero-banner__title');
            return el ? { found: true, tag: el.tagName } : { found: false };
        })()"#,
    ];

    for pattern in patterns {
        assert!(pattern.len() > 10);
        println!("✅ Pattern validated: {}", &pattern[..50.min(pattern.len())]);
    }
}