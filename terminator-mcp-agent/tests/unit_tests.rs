use serde_json::json;
use terminator_mcp_agent::utils::{ExecuteSequenceArgs, ToolCall};

#[test]
fn test_execute_sequence_args_serialization() {
    let args = ExecuteSequenceArgs {
        tools: vec![ToolCall {
            tool_name: "test_tool".to_string(),
            arguments: json!({"key": "value"}),
            continue_on_error: Some(true),
            delay_ms: Some(100),
        }],
        stop_on_error: Some(false),
        delay_between_tools_ms: Some(50),
        include_detailed_results: Some(true),
    };

    // Test serialization
    let serialized = serde_json::to_value(&args).unwrap();
    assert_eq!(serialized["tools"][0]["tool_name"], "test_tool");
    assert_eq!(serialized["tools"][0]["arguments"]["key"], "value");
    assert_eq!(serialized["tools"][0]["continue_on_error"], true);
    assert_eq!(serialized["tools"][0]["delay_ms"], 100);
    assert_eq!(serialized["stop_on_error"], false);
    assert_eq!(serialized["delay_between_tools_ms"], 50);
    assert_eq!(serialized["include_detailed_results"], true);

    // Test deserialization
    let json_str = r#"{
        "tools": [
            {
                "tool_name": "another_tool",
                "arguments": {"foo": "bar"},
                "continue_on_error": false,
                "delay_ms": 200
            }
        ],
        "stop_on_error": true,
        "delay_between_tools_ms": 100,
        "include_detailed_results": false
    }"#;

    let deserialized: ExecuteSequenceArgs = serde_json::from_str(json_str).unwrap();
    assert_eq!(deserialized.tools.len(), 1);
    assert_eq!(deserialized.tools[0].tool_name, "another_tool");
    assert_eq!(deserialized.tools[0].arguments["foo"], "bar");
    assert_eq!(deserialized.tools[0].continue_on_error, Some(false));
    assert_eq!(deserialized.tools[0].delay_ms, Some(200));
    assert_eq!(deserialized.stop_on_error, Some(true));
    assert_eq!(deserialized.delay_between_tools_ms, Some(100));
    assert_eq!(deserialized.include_detailed_results, Some(false));
}

#[test]
fn test_tool_call_defaults() {
    // Test that optional fields can be omitted
    let json_str = r#"{
        "tool_name": "minimal_tool",
        "arguments": {}
    }"#;

    let tool_call: ToolCall = serde_json::from_str(json_str).unwrap();
    assert_eq!(tool_call.tool_name, "minimal_tool");
    assert_eq!(tool_call.arguments, json!({}));
    assert_eq!(tool_call.continue_on_error, None);
    assert_eq!(tool_call.delay_ms, None);
}

#[test]
fn test_execute_sequence_minimal() {
    // Test minimal valid execute sequence args
    let json_str = r#"{
        "tools": []
    }"#;

    let args: ExecuteSequenceArgs = serde_json::from_str(json_str).unwrap();
    assert_eq!(args.tools.len(), 0);
    assert_eq!(args.stop_on_error, None);
    assert_eq!(args.delay_between_tools_ms, None);
    assert_eq!(args.include_detailed_results, None);
}

#[test]
fn test_complex_arguments_preservation() {
    let complex_args = json!({
        "nested": {
            "array": [1, 2, 3],
            "object": {
                "key": "value"
            }
        },
        "boolean": true,
        "number": 42.5,
        "null_value": null
    });

    let tool_call = ToolCall {
        tool_name: "complex_tool".to_string(),
        arguments: complex_args.clone(),
        continue_on_error: None,
        delay_ms: None,
    };

    let serialized = serde_json::to_value(&tool_call).unwrap();
    assert_eq!(serialized["arguments"], complex_args);
}
