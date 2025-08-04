use std::time::Duration;
use terminator::{Browser, Desktop};
use tracing::info;

#[tokio::test]
async fn test_div_by_id_extraction() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init()
        .ok();

    info!("🧪 Testing DIV extraction by ID from complex webpage");

    let desktop = Desktop::new(false, false)?;

    // Use a page with known structure and IDs - example.com has simple structure
    info!("📘 Opening test page with known DIV structure...");
    let browser_window = desktop.open_url("https://example.com", Some(Browser::Edge))?;

    // Wait for page to load
    tokio::time::sleep(Duration::from_secs(4)).await;

    // Try to find document element first, with fallback options
    info!("📄 Looking for document element...");
    let document = match browser_window.locator("role:Document") {
        Ok(doc_locator) => {
            match doc_locator.first(Some(Duration::from_secs(3))).await {
                Ok(doc) => {
                    info!("✅ Found document element via role:Document");
                    doc
                }
                Err(_) => {
                    info!("⚠️ Document not found via role, using browser window as document");
                    // Use the browser window itself as the document for script execution
                    browser_window.clone()
                }
            }
        }
        Err(_) => {
            info!("⚠️ Could not create document locator, using browser window");
            browser_window.clone()
        }
    };

    // Test 1: Extract content using JavaScript to find specific div by ID
    info!("🧪 Test 1: Extract DIV content using JavaScript ID selector");

    let js_scripts = [
        "document.getElementById('main')",
        "document.querySelector('#main')",
        "document.querySelector('div')",
        "document.getElementsByTagName('div')[0]",
        "document.body.querySelector('div')",
    ];

    for script in &js_scripts {
        info!("  🔎 Trying JS: {}", script);
        match document.execute_script(script) {
            Ok(Some(result)) => {
                info!("    ✅ SUCCESS: Result: '{}'", result);
                break;
            }
            Ok(None) => {
                info!("    ⚠️ Script executed but returned no result");
            }
            Err(e) => {
                info!("    ❌ Script failed: {}", e);
            }
        }
    }

    // Test 2: Extract text content from main content area
    info!("🧪 Test 2: Extract main content text using JavaScript");

    let content_scripts = [
        "document.body.innerText",
        "document.body.textContent",
        "document.querySelector('h1') ? document.querySelector('h1').textContent : 'No H1'",
        "document.querySelector('p') ? document.querySelector('p').textContent : 'No paragraph'",
        "Array.from(document.querySelectorAll('p')).map(p => p.textContent).join(' | ')",
    ];

    for script in &content_scripts {
        info!("  🔎 Content extraction: {}", script);
        match document.execute_script(script) {
            Ok(Some(content)) => {
                let preview = if content.len() > 100 {
                    format!("{}...", &content[..100])
                } else {
                    content
                };
                info!("    ✅ Content found: '{}'", preview);
            }
            Ok(None) => {
                info!("    ⚠️ No content returned");
            }
            Err(e) => {
                info!("    ❌ Content extraction failed: {}", e);
            }
        }
    }

    // Test 3: Extract complete DOM structure
    info!("🧪 Test 3: Extract DOM structure for analysis");

    let structure_scripts = [
        "document.documentElement.outerHTML",
        "document.body.innerHTML",
        "JSON.stringify({title: document.title, url: location.href, elementCount: document.querySelectorAll('*').length})"
    ];

    for script in &structure_scripts {
        info!("  🔎 Structure: {}", script);
        match document.execute_script(script) {
            Ok(Some(structure)) => {
                let structure_len = structure.len();
                let preview = if structure.len() > 200 {
                    format!("{}...", &structure[..200])
                } else {
                    structure
                };
                info!("    ✅ Structure ({} chars): '{}'", structure_len, preview);
            }
            Ok(None) => {
                info!("    ⚠️ No structure returned");
            }
            Err(e) => {
                info!("    ❌ Structure extraction failed: {}", e);
            }
        }
    }

    // Test 4: Use accessibility API to find elements by role and name
    info!("🧪 Test 4: Extract using accessibility selectors");

    let accessibility_selectors = [
        "role:Heading",
        "role:Text",
        "role:Link",
        "role:Group",
        "name:Example Domain", // example.com has this text
    ];

    for selector in &accessibility_selectors {
        info!("  🔎 Accessibility selector: {}", selector);
        match document.locator(*selector) {
            Ok(locator) => match locator.all(Some(Duration::from_secs(2)), Some(3)).await {
                Ok(elements) => {
                    info!("    ✅ Found {} elements", elements.len());
                    for (i, elem) in elements.iter().enumerate() {
                        if let Some(text) = elem.name() {
                            let preview = if text.len() > 80 {
                                format!("{}...", &text[..80])
                            } else {
                                text
                            };
                            info!("      📄 Element {}: '{}'", i + 1, preview);
                        }
                    }
                }
                Err(e) => {
                    info!("    ❌ Could not get elements: {}", e);
                }
            },
            Err(e) => {
                info!("    ❌ Could not create locator: {}", e);
            }
        }
    }

    info!("🧹 Closing browser...");
    let _ = browser_window.close();

    info!("✅ DIV extraction test completed!");
    Ok(())
}

#[tokio::test]
async fn test_complex_page_div_extraction() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init()
        .ok();

    info!("🧪 Testing complex page DIV extraction with specific targeting");

    let desktop = Desktop::new(false, false)?;

    // Use GitHub.com which has complex div structure with IDs
    info!("📘 Opening GitHub homepage for complex div extraction...");
    let browser_window = desktop.open_url("https://github.com", Some(Browser::Edge))?;

    // Wait longer for complex page to load
    tokio::time::sleep(Duration::from_secs(6)).await;

    let document = match browser_window.locator("role:Document") {
        Ok(doc_locator) => match doc_locator.first(Some(Duration::from_secs(3))).await {
            Ok(doc) => {
                info!("✅ Found document element via role:Document");
                doc
            }
            Err(_) => {
                info!("⚠️ Document not found via role, using browser window as document");
                browser_window.clone()
            }
        },
        Err(_) => {
            info!("⚠️ Could not create document locator, using browser window");
            browser_window.clone()
        }
    };

    // Test extracting specific GitHub page sections
    info!("🧪 Testing GitHub page section extraction");

    let github_selectors = [
        // Common GitHub div IDs and classes
        "document.querySelector('.application-main')",
        "document.querySelector('[data-turbo-body]')", 
        "document.querySelector('main')",
        "document.querySelector('header')",
        "document.querySelector('.Header')",
        // Content extraction
        "document.title",
        "document.querySelector('h1') ? document.querySelector('h1').textContent : 'No main heading'",
        "Array.from(document.querySelectorAll('a')).slice(0,3).map(a => a.textContent.trim()).filter(t => t).join(' | ')",
        "document.querySelectorAll('div').length + ' total divs found'"
    ];

    for script in &github_selectors {
        info!("  🔎 GitHub extraction: {}", script);
        match document.execute_script(script) {
            Ok(Some(result)) => {
                let preview = if result.len() > 150 {
                    format!("{}...", &result[..150])
                } else {
                    result
                };
                info!("    ✅ SUCCESS: '{}'", preview);
            }
            Ok(None) => {
                info!("    ⚠️ No result returned");
            }
            Err(e) => {
                info!("    ❌ Failed: {}", e);
            }
        }
    }

    // Test extracting navigation and content structure
    info!("🧪 Testing navigation and content structure extraction");

    let nav_scripts = [
        "Array.from(document.querySelectorAll('nav a')).slice(0,5).map(a => a.textContent.trim()).filter(t => t).join(' | ')",
        "document.querySelector('nav') ? 'Navigation found' : 'No navigation'",
        "document.querySelectorAll('[id]').length + ' elements with IDs'",
        "Array.from(document.querySelectorAll('[id]')).slice(0,5).map(el => el.id).filter(id => id).join(' | ')"
    ];

    for script in &nav_scripts {
        info!("  🔎 Navigation: {}", script);
        match document.execute_script(script) {
            Ok(Some(result)) => {
                info!("    ✅ Nav result: '{}'", result);
            }
            Ok(None) => {
                info!("    ⚠️ No nav result");
            }
            Err(e) => {
                info!("    ❌ Nav failed: {}", e);
            }
        }
    }

    info!("🧹 Closing browser...");
    let _ = browser_window.close();

    info!("✅ Complex page DIV extraction test completed!");
    Ok(())
}
