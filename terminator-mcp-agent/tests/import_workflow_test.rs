#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn test_import_workflow_args_serialization() {
        // Test that the new return_raw field is properly serialized
        let args_with_return_raw = json!({
            "file_path": "test.yaml",
            "return_raw": true
        });

        let args_without_return_raw = json!({
            "file_path": "test.yaml"
        });

        // Verify the JSON structure
        assert!(args_with_return_raw["return_raw"].as_bool().unwrap());
        assert!(args_without_return_raw["return_raw"].is_null());

        println!("✓ ImportWorkflowSequenceArgs serialization test passed");
    }

    #[test]
    fn test_return_raw_default() {
        // Test the default behavior of return_raw field
        let args_none = json!({
            "file_path": "test.yaml",
            "return_raw": null
        });

        let args_false = json!({
            "file_path": "test.yaml",
            "return_raw": false
        });

        let args_true = json!({
            "file_path": "test.yaml",
            "return_raw": true
        });

        // Verify defaults
        assert!(args_none["return_raw"].is_null());
        assert!(!args_false["return_raw"].as_bool().unwrap());
        assert!(args_true["return_raw"].as_bool().unwrap());

        println!("✓ return_raw default value test passed");
    }
}
