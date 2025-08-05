use terminator::Desktop;
use tracing::{debug, info};

/// Example demonstrating WebView2 script execution for element retrieval
/// This example shows how to:
/// 1. Connect to a WebView2 control
/// 2. Execute JavaScript to get elements by ID
/// 3. Extract text content from elements
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting WebView2 script execution example");

    // Create a desktop instance
    let desktop = Desktop::new(false, true)?;

    // Find applications with WebView2 controls
    // This could be Edge, Teams, VS Code, or any app using WebView2
    let applications = desktop.applications()?;

    for app in applications {
        let app_name = app.name().unwrap_or_else(|| "Unknown".to_string());
        info!("Checking application: {}", app_name);

        // Look for WebView2 elements in the application
        // WebView2 controls often have Custom role or Pane role
        if let Ok(children) = app.children() {
            for element in children {
                // Check if this might be a WebView2 control
                if let Some(url) = element.url() {
                    if !url.is_empty() {
                        info!("Found potential WebView2 element with URL: {}", url);

                        // Test basic element retrieval scripts
                        let test_scripts = vec![
                            // Get element by ID and return text content
                            (
                                "Get element text by ID",
                                "document.getElementById('content')?.textContent",
                            ),
                            // Get element by ID and return all properties
                            (
                                "Get element properties",
                                r#"(() => {
                                const el = document.getElementById('main');
                                if (!el) return null;
                                return JSON.stringify({
                                    text: el.textContent,
                                    html: el.innerHTML,
                                    tagName: el.tagName,
                                    visible: el.offsetWidth > 0 && el.offsetHeight > 0
                                });
                            })()"#,
                            ),
                            // Get page title and basic info
                            (
                                "Get page info",
                                r#"JSON.stringify({
                                title: document.title,
                                url: window.location.href,
                                elementCount: document.getElementsByTagName('*').length
                            })"#,
                            ),
                            // Search for common element IDs and get their text
                            (
                                "Find common elements",
                                r#"(() => {
                                const commonIds = ['header', 'main', 'content', 'body', 'nav', 'footer'];
                                const results = {};
                                commonIds.forEach(id => {
                                    const el = document.getElementById(id);
                                    if (el) {
                                        results[id] = {
                                            text: el.textContent?.substring(0, 100) || '',
                                            tagName: el.tagName,
                                            hasChildren: el.children.length > 0
                                        };
                                    }
                                });
                                return JSON.stringify(results);
                            })()"#,
                            ),
                            // Extract all text content from page
                            ("Extract all text", "document.body.textContent"),
                        ];

                        for (description, script) in test_scripts {
                            info!("Testing: {}", description);
                            debug!("Script: {}", script);

                            match element.execute_script(script) {
                                Ok(Some(result)) => {
                                    info!(
                                        "✅ {} - Result: {}",
                                        description,
                                        if result.len() > 100 {
                                            format!("{}...", &result[..100])
                                        } else {
                                            result
                                        }
                                    );
                                }
                                Ok(None) => {
                                    info!("⚠️  {} - No result returned", description);
                                }
                                Err(e) => {
                                    info!("❌ {} - Error: {}", description, e);
                                }
                            }
                        }

                        // Test element interaction patterns
                        info!("Testing element interaction patterns...");

                        let interaction_scripts = vec![
                            // Find all input elements and their values
                            (
                                "Get all inputs",
                                r#"Array.from(document.querySelectorAll('input')).map(input => ({
                                id: input.id,
                                name: input.name,
                                type: input.type,
                                value: input.value,
                                placeholder: input.placeholder
                            }))"#,
                            ),
                            // Find all buttons and their text
                            (
                                "Get all buttons",
                                r#"Array.from(document.querySelectorAll('button')).map(btn => ({
                                id: btn.id,
                                text: btn.textContent?.trim(),
                                enabled: !btn.disabled,
                                visible: btn.offsetWidth > 0
                            }))"#,
                            ),
                            // Get form data if any forms exist
                            (
                                "Get form data",
                                r#"(() => {
                                const forms = Array.from(document.querySelectorAll('form'));
                                return forms.map(form => {
                                    const formData = {};
                                    const inputs = form.querySelectorAll('input, select, textarea');
                                    inputs.forEach(input => {
                                        if (input.name) formData[input.name] = input.value;
                                    });
                                    return { id: form.id, action: form.action, data: formData };
                                });
                            })()"#,
                            ),
                        ];

                        for (description, script) in interaction_scripts {
                            match element.execute_script(script) {
                                Ok(Some(result)) => {
                                    info!(
                                        "✅ {}: {}",
                                        description,
                                        if result.len() > 200 {
                                            format!("{}...", &result[..200])
                                        } else {
                                            result
                                        }
                                    );
                                }
                                Ok(None) => {
                                    debug!("⚠️  {} - No result", description);
                                }
                                Err(e) => {
                                    debug!("❌ {} - Error: {}", description, e);
                                }
                            }
                        }

                        // Found one WebView2 control, that's enough for the example
                        break;
                    }
                }
            }
        }
    }

    info!("WebView2 script execution example completed");
    Ok(())
}

/// Helper function to demonstrate element-by-ID text extraction
/// This shows the pattern for robust element text extraction
#[allow(dead_code)]
fn create_robust_text_extraction_script(element_id: &str) -> String {
    format!(
        r#"(() => {{
        const extractText = (elementId) => {{
            const element = document.getElementById(elementId);
            if (!element) {{
                return {{ error: 'Element not found', elementId }};
            }}

            // Multiple text extraction strategies
            const strategies = {{
                textContent: element.textContent,
                innerText: element.innerText,
                innerHTML: element.innerHTML,
                value: element.value || null,
                title: element.title || null,
                alt: element.alt || null,
                placeholder: element.placeholder || null
            }};

            // Get all text nodes recursively
            const getTextNodes = (node) => {{
                let text = '';
                if (node.nodeType === Node.TEXT_NODE) {{
                    text = node.textContent.trim();
                }} else {{
                    for (let child of node.childNodes) {{
                        text += getTextNodes(child) + ' ';
                    }}
                }}
                return text.trim();
            }};

            strategies.allTextNodes = getTextNodes(element);

            // Get visible text only
            if (element.offsetWidth > 0 && element.offsetHeight > 0) {{
                strategies.visibleText = element.innerText;
            }} else {{
                strategies.visibleText = null;
            }}

            return {{
                elementId,
                tagName: element.tagName,
                strategies,
                metadata: {{
                    childElementCount: element.children.length,
                    hasAttributes: element.attributes.length > 0,
                    isVisible: element.offsetWidth > 0 && element.offsetHeight > 0
                }}
            }};
        }};

        return extractText('{}');
    }})()"#,
        element_id
    )
}

/// Helper function to create a script that safely gets element text by ID
#[allow(dead_code)]
fn create_safe_get_element_text_script(element_id: &str) -> String {
    format!(
        "document.getElementById('{}')?.textContent || 'Element not found'",
        element_id.replace('\'', "\\'") // Escape single quotes for safety
    )
}

/// Helper function to create a script that gets element properties by ID
#[allow(dead_code)]
fn create_get_element_properties_script(element_id: &str) -> String {
    format!(
        r#"(() => {{
        const el = document.getElementById('{}');
        if (!el) return null;
        return JSON.stringify({{
            text: el.textContent,
            html: el.innerHTML,
            tagName: el.tagName,
            className: el.className,
            id: el.id,
            visible: el.offsetWidth > 0 && el.offsetHeight > 0,
            enabled: !el.disabled,
            bounds: {{
                x: el.offsetLeft,
                y: el.offsetTop,
                width: el.offsetWidth,
                height: el.offsetHeight
            }}
        }});
    }})()"#,
        element_id.replace('\'', "\\'")
    )
}
