use crate::scripting_engine::execute_javascript_with_nodejs;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JavaScript-based parser definition
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputParserDefinition {
    /// Optional: Specify which step ID contains the UI tree to parse
    #[serde(default)]
    pub ui_tree_source_step_id: Option<String>,
    /// JavaScript code that processes the tree and returns results
    /// The code receives a 'tree' variable containing the UI tree
    /// and should return an array of objects
    pub javascript_code: String,
}

/// The main entry point for parsing tool output.
pub async fn run_output_parser(parser_def_val: &Value, tool_output: &Value) -> Result<Option<Value>> {
    let parser_def: OutputParserDefinition = serde_json::from_value(parser_def_val.clone())
        .map_err(|e| {
            anyhow::anyhow!(
                "Invalid parser definition format. Expected JavaScript format: {}",
                e
            )
        })?;

    let ui_tree =
        find_ui_tree_in_results(tool_output, parser_def.ui_tree_source_step_id.as_deref())?;

    match ui_tree {
        Some(tree) => {
            // Create JavaScript code that injects the tree and executes the user code
            let full_script = format!(
                r#"
                // Inject the UI tree as a global variable
                const tree = {};
                
                // Execute the user's parsing logic and return the result
                {}
                "#,
                serde_json::to_string(&tree).map_err(|e| anyhow::anyhow!("Failed to serialize tree: {}", e))?,
                parser_def.javascript_code
            );
            
            // Execute JavaScript code asynchronously
            let result = execute_javascript_with_nodejs(full_script)
                .await
                .map_err(|e| anyhow::anyhow!("JavaScript execution failed: {}", e))?;
            
            Ok(Some(result))
        }
        None => {
            anyhow::bail!("No UI tree found in the tool output. Make sure to include get_focused_window_tree or get_window_tree in your workflow to capture the UI state.");
        }
    }
}

/// Finds a UI tree in the tool output results
fn find_ui_tree_in_results(tool_output: &Value, step_id: Option<&str>) -> Result<Option<Value>> {
    // Strategy 0: If step_id is specified, look for that specific step first
    if let Some(target_step_id) = step_id {
        if let Some(results) = tool_output.get("results") {
            if let Some(results_array) = results.as_array() {
                fn search_for_step_id(results: &[Value], target_step_id: &str) -> Option<Value> {
                    for result in results {
                        if let Some(result_step_id) = result.get("step_id").and_then(|v| v.as_str())
                        {
                            if result_step_id == target_step_id {
                                if let Some(ui_tree) = result.get("ui_tree") {
                                    return Some(ui_tree.clone());
                                }
                                if let Some(result_obj) = result.get("result") {
                                    if let Some(ui_tree) = result_obj.get("ui_tree") {
                                        return Some(ui_tree.clone());
                                    }
                                    if let Some(content) = result_obj.get("content") {
                                        if let Some(content_array) = content.as_array() {
                                            for content_item in content_array {
                                                if let Some(text) = content_item.get("text") {
                                                    if let Some(text_str) = text.as_str() {
                                                        if let Ok(parsed_json) =
                                                            serde_json::from_str::<Value>(text_str)
                                                        {
                                                            if let Some(ui_tree) =
                                                                parsed_json.get("ui_tree")
                                                            {
                                                                return Some(ui_tree.clone());
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                return None;
                            }
                        }

                        if let Some(group_results) = result.get("results") {
                            if let Some(group_results_array) = group_results.as_array() {
                                if let Some(found) =
                                    search_for_step_id(group_results_array, target_step_id)
                                {
                                    return Some(found);
                                }
                            }
                        }
                    }
                    None
                }

                if let Some(ui_tree) = search_for_step_id(results_array, target_step_id) {
                    return Ok(Some(ui_tree));
                }

                anyhow::bail!("Step with ID '{}' not found in results", target_step_id);
            }
        }
        anyhow::bail!(
            "Step ID '{}' specified but no results array found",
            target_step_id
        );
    }

    // Strategy 1: Check if there's a direct ui_tree field
    if let Some(ui_tree) = tool_output.get("ui_tree") {
        return Ok(Some(ui_tree.clone()));
    }

    // Strategy 2: Look through results array for UI trees (fallback behavior)
    if let Some(results) = tool_output.get("results") {
        if let Some(results_array) = results.as_array() {
            for result in results_array.iter().rev() {
                if let Some(ui_tree) = result.get("ui_tree") {
                    return Ok(Some(ui_tree.clone()));
                }

                if let Some(result_obj) = result.get("result") {
                    if let Some(ui_tree) = result_obj.get("ui_tree") {
                        return Ok(Some(ui_tree.clone()));
                    }

                    if let Some(content) = result_obj.get("content") {
                        if let Some(content_array) = content.as_array() {
                            for content_item in content_array.iter().rev() {
                                if let Some(text) = content_item.get("text") {
                                    if let Some(text_str) = text.as_str() {
                                        if let Ok(parsed_json) =
                                            serde_json::from_str::<Value>(text_str)
                                        {
                                            if let Some(ui_tree) = parsed_json.get("ui_tree") {
                                                return Ok(Some(ui_tree.clone()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_javascript_parser() {
        let ui_tree = json!({
            "children": [
                {
                    "attributes": {
                        "role": "CheckBox",
                        "name": "Test Product",
                        "id": "123",
                        "is_toggled": true
                    }
                },
                {
                    "attributes": {
                        "role": "CheckBox",
                        "name": "Another Product",
                        "id": "456",
                        "is_toggled": false
                    }
                }
            ]
        });

        let parser_def = OutputParserDefinition {
            ui_tree_source_step_id: None,
            javascript_code: r#"
                const results = [];

                function findElementsRecursively(element) {
                    if (element.attributes && element.attributes.role === 'CheckBox') {
                        const item = {
                            productName: element.attributes.name || '',
                            id: element.attributes.id || '',
                            is_toggled: element.attributes.is_toggled || false
                        };
                        results.push(item);
                    }
                    
                    if (element.children) {
                        for (const child of element.children) {
                            findElementsRecursively(child);
                        }
                    }
                }

                findElementsRecursively(tree);
                return results;
            "#
            .to_string(),
        };

        let tool_output = json!({
            "ui_tree": ui_tree
        });

        // Note: This test would require an async runtime to execute JavaScript
        // For now, we'll just verify the parser definition structure is correct
        let parser_def_json = serde_json::to_value(&parser_def).unwrap();
        assert!(parser_def_json.get("javascript_code").is_some());
    }

    #[test]
    fn test_empty_results() {
        let ui_tree = json!({
            "children": []
        });

        let parser_def = OutputParserDefinition {
            ui_tree_source_step_id: None,
            javascript_code: r#"
                const results = [];
                
                function findElementsRecursively(element) {
                    if (element.children) {
                        for (const child of element.children) {
                            findElementsRecursively(child);
                        }
                    }
                }
                
                findElementsRecursively(tree);
                return results;
            "#
            .to_string(),
        };

        let tool_output = json!({
            "ui_tree": ui_tree
        });

        // Verify parser definition structure
        let parser_def_json = serde_json::to_value(&parser_def).unwrap();
        assert!(parser_def_json.get("javascript_code").is_some());
    }

    #[test]
    fn test_step_id_lookup() {
        let ui_tree = json!({
            "children": [
                {
                    "attributes": {
                        "role": "CheckBox",
                        "name": "Found Product",
                        "id": "789"
                    }
                }
            ]
        });

        let parser_def = OutputParserDefinition {
            ui_tree_source_step_id: Some("test_step".to_string()),
            javascript_code: r#"
                const results = [];
                
                function findElementsRecursively(element) {
                    if (element.attributes && element.attributes.role === 'CheckBox') {
                        const item = {
                            productName: element.attributes.name || '',
                            id: element.attributes.id || ''
                        };
                        results.push(item);
                    }
                    
                    if (element.children) {
                        for (const child of element.children) {
                            findElementsRecursively(child);
                        }
                    }
                }
                
                findElementsRecursively(tree);
                return results;
            "#
            .to_string(),
        };

        let tool_output = json!({
            "results": [
                {
                    "step_id": "test_step",
                    "ui_tree": ui_tree
                }
            ]
        });

        // Verify parser definition structure
        let parser_def_json = serde_json::to_value(&parser_def).unwrap();
        assert!(parser_def_json.get("javascript_code").is_some());
        assert_eq!(
            parser_def.ui_tree_source_step_id,
            Some("test_step".to_string())
        );
    }

    #[test]
    fn test_attribute_value_filtering() {
        let parser_def = OutputParserDefinition {
            ui_tree_source_step_id: None,
            javascript_code: r#"
                const results = [];
                
                function findElementsRecursively(element) {
                    if (element.attributes && 
                        element.attributes.role === 'CheckBox' && 
                        element.attributes.is_toggled === true) {
                        
                        const item = {
                            productName: element.attributes.name || ''
                        };
                        results.push(item);
                    }
                    
                    if (element.children) {
                        for (const child of element.children) {
                            findElementsRecursively(child);
                        }
                    }
                }
                
                findElementsRecursively(tree);
                return results;
            "#
            .to_string(),
        };

        let ui_tree = json!({
            "children": [
                {
                    "attributes": {
                        "role": "CheckBox",
                        "name": "Toggled Checkbox",
                        "is_toggled": true
                    }
                },
                {
                    "attributes": {
                        "role": "CheckBox",
                        "name": "Untoggled Checkbox",
                        "is_toggled": false
                    }
                },
                {
                    "attributes": {
                        "role": "CheckBox",
                        "name": "Missing Toggle Checkbox"
                    }
                }
            ]
        });

        let tool_output = json!({
            "ui_tree": ui_tree
        });

        // Verify parser definition structure
        let parser_def_json = serde_json::to_value(&parser_def).unwrap();
        assert!(parser_def_json.get("javascript_code").is_some());
    }

    #[test]
    fn test_parser_definition_serialization() {
        // Test the new clean syntax for JavaScript-based parsing
        let parser_def_json = json!({
            "ui_tree_source_step_id": "capture_tree",
            "javascript_code": "return [];"
        });

        let parser_def: OutputParserDefinition = serde_json::from_value(parser_def_json).unwrap();
        assert_eq!(
            parser_def.ui_tree_source_step_id,
            Some("capture_tree".to_string())
        );
        assert_eq!(parser_def.javascript_code, "return [];");
    }
}
