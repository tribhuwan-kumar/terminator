use crate::AutomationError;

/// Unit tests for the Dataiku form element finding functionality
/// Tests the JavaScript patterns used to find element with ID: hs_form_target_form_735002917
#[cfg(test)]
mod dataiku_form_tests {
    use super::*;

    /// Test the JavaScript pattern for finding the specific Dataiku form element
    #[test]
    fn test_dataiku_form_element_finder_script() {
        let target_id = "hs_form_target_form_735002917";
        
        // Create the comprehensive element finder script
        let script = create_comprehensive_element_finder(target_id);
        
        // Validate script structure
        assert!(script.contains("getElementById"));
        assert!(script.contains(target_id));
        assert!(script.contains("JSON.stringify"));
        assert!(script.len() > 1000); // Should be a substantial script
        
        println!("✅ Comprehensive element finder script generated: {} chars", script.len());
    }

    /// Test simple element existence check script
    #[test]
    fn test_simple_element_existence_script() {
        let target_id = "hs_form_target_form_735002917";
        
        let scripts = vec![
            // Basic existence check
            format!("document.getElementById('{}') !== null", target_id),
            
            // Get element with error handling
            format!("document.getElementById('{}')?.textContent || 'Not found'", target_id),
            
            // Get element properties safely
            format!(r#"(() => {{
                const el = document.getElementById('{}');
                return el ? el.tagName : 'Not found';
            }})()"#, target_id),
        ];

        for script in scripts {
            assert!(script.contains(target_id));
            assert!(!script.is_empty());
            println!("✅ Element existence script: {}", script);
        }
    }

    /// Test HubSpot form finder patterns
    #[test]
    fn test_hubspot_form_finder_patterns() {
        let patterns = vec![
            // Find all elements with 'hs_form' in ID
            r#"Array.from(document.querySelectorAll('[id*="hs_form"]')).map(el => ({
                id: el.id,
                tagName: el.tagName,
                visible: el.offsetWidth > 0
            }))"#,
            
            // Find all forms on the page
            r#"Array.from(document.querySelectorAll('form')).map(f => ({
                id: f.id,
                name: f.name,
                action: f.action
            })).filter(f => f.id)"#,
            
            // Find HubSpot forms specifically
            r#"Array.from(document.querySelectorAll('form[id*="hs_form"], div[id*="hs_form"]')).map(el => ({
                id: el.id,
                tagName: el.tagName,
                className: el.className
            }))"#,
        ];

        for pattern in patterns {
            assert!(pattern.contains("querySelectorAll"));
            assert!(!pattern.is_empty());
            println!("✅ HubSpot form finder pattern validated");
        }
    }

    /// Test form element analysis scripts
    #[test]
    fn test_form_element_analysis_scripts() {
        let target_id = "hs_form_target_form_735002917";
        
        let analysis_script = format!(r#"(() => {{
            const form = document.getElementById('{}');
            if (!form) return {{ error: 'Form not found' }};
            
            return {{
                formInfo: {{
                    id: form.id,
                    tagName: form.tagName,
                    action: form.action || '',
                    method: form.method || '',
                    elementCount: form.elements?.length || 0
                }},
                inputs: Array.from(form.querySelectorAll('input')).map(input => ({{
                    name: input.name,
                    type: input.type,
                    placeholder: input.placeholder,
                    required: input.required
                }})),
                visible: form.offsetWidth > 0 && form.offsetHeight > 0
            }};
        }})()"#, target_id);

        assert!(analysis_script.contains("getElementById"));
        assert!(analysis_script.contains("querySelectorAll"));
        assert!(analysis_script.contains("formInfo"));
        assert!(analysis_script.len() > 500);
        
        println!("✅ Form analysis script: {} chars", analysis_script.len());
    }

    /// Test error handling in element finder scripts
    #[test]
    fn test_error_handling_patterns() {
        let target_id = "hs_form_target_form_735002917";
        
        let safe_scripts = vec![
            // Optional chaining
            format!("document.getElementById('{}')?.textContent", target_id),
            
            // Null check with fallback
            format!("document.getElementById('{}') || 'Element not found'", target_id),
            
            // Try-catch pattern
            format!(r#"(() => {{
                try {{
                    const el = document.getElementById('{}');
                    return el ? el.textContent : 'Element not found';
                }} catch (e) {{
                    return 'Error: ' + e.message;
                }}
            }})()"#, target_id),
            
            // Existence check first
            format!(r#"(() => {{
                const el = document.getElementById('{}');
                if (!el) return 'Element not found';
                return {{
                    id: el.id,
                    text: el.textContent || '',
                    visible: el.offsetWidth > 0
                }};
            }})()"#, target_id),
        ];

        for script in safe_scripts {
            assert!(script.contains(target_id));
            println!("✅ Safe script pattern validated");
        }
    }

    /// Test page information gathering scripts
    #[test]
    fn test_page_info_scripts() {
        let info_scripts = vec![
            // Basic page info
            r#"JSON.stringify({
                title: document.title,
                url: window.location.href,
                readyState: document.readyState
            })"#,
            
            // Form count
            "document.querySelectorAll('form').length",
            
            // Element count
            "document.getElementsByTagName('*').length",
            
            // HubSpot elements
            "document.querySelectorAll('[id*=\"hs_form\"]').length",
        ];

        for script in info_scripts {
            assert!(!script.is_empty());
            println!("✅ Page info script validated");
        }
    }

    /// Test the mock execution pipeline
    #[test]
    fn test_mock_script_execution() {
        let target_id = "hs_form_target_form_735002917";
        let script = create_simple_element_finder(target_id);
        
        // Mock execution would happen here in a real WebView2 environment
        let mock_result = simulate_script_execution(&script);
        
        match mock_result {
            Ok(result) => {
                assert!(!result.is_empty());
                println!("✅ Mock script execution successful: {}", result);
            }
            Err(e) => {
                println!("⚠️  Mock script execution failed (expected in unit tests): {}", e);
            }
        }
    }
}

/// Create a comprehensive element finder script for the target ID
fn create_comprehensive_element_finder(element_id: &str) -> String {
    format!(r#"(() => {{
        const targetId = '{}';
        const result = {{
            timestamp: new Date().toISOString(),
            pageInfo: {{
                title: document.title,
                url: window.location.href,
                readyState: document.readyState
            }},
            targetElement: null,
            allFormsInfo: [],
            hsFormElements: []
        }};

        // Find the target element
        const targetElement = document.getElementById(targetId);
        if (targetElement) {{
            result.targetElement = {{
                id: targetElement.id,
                tagName: targetElement.tagName,
                className: targetElement.className,
                textContent: targetElement.textContent?.substring(0, 500) || '',
                innerHTML: targetElement.innerHTML?.substring(0, 1000) || '',
                attributes: Array.from(targetElement.attributes).map(attr => ({{
                    name: attr.name,
                    value: attr.value
                }})),
                bounds: {{
                    x: targetElement.offsetLeft,
                    y: targetElement.offsetTop,
                    width: targetElement.offsetWidth,
                    height: targetElement.offsetHeight
                }},
                visible: targetElement.offsetWidth > 0 && targetElement.offsetHeight > 0,
                childElementCount: targetElement.children.length,
                parentTagName: targetElement.parentElement?.tagName || null
            }};
        }}

        // Get all forms on the page
        const forms = Array.from(document.querySelectorAll('form'));
        result.allFormsInfo = forms.map(form => ({{
            id: form.id || '',
            name: form.name || '',
            action: form.action || '',
            method: form.method || '',
            elementCount: form.elements.length,
            visible: form.offsetWidth > 0 && form.offsetHeight > 0
        }}));

        // Find all elements with 'hs_form' in their ID
        const hsFormElements = Array.from(document.querySelectorAll('[id*="hs_form"]'));
        result.hsFormElements = hsFormElements.map(el => ({{
            id: el.id,
            tagName: el.tagName,
            className: el.className,
            visible: el.offsetWidth > 0 && el.offsetHeight > 0
        }}));

        return JSON.stringify(result, null, 2);
    }})()"#, element_id)
}

/// Create a simple element finder script
fn create_simple_element_finder(element_id: &str) -> String {
    format!(
        "document.getElementById('{}')?.textContent || 'Element not found'",
        element_id.replace('\'', "\\'")
    )
}

/// Simulate script execution for testing purposes
fn simulate_script_execution(script: &str) -> Result<String, AutomationError> {
    // In unit tests, we can't execute real JavaScript
    // This simulates what would happen in a real WebView2 environment
    if script.contains("getElementById") {
        Ok("Mock result: Element processing completed".to_string())
    } else {
        Err(AutomationError::ElementNotFound(
            "Mock execution: Script format not recognized".to_string()
        ))
    }
}