mod common;

use crate::common::get_result_json;
use rmcp::handler::server::tool::Parameters;
use serde_json::json;
use std::collections::HashMap;
use terminator_mcp_agent::utils::{DesktopWrapper, ExecuteSequenceArgs, SequenceStep};

#[tokio::test]
async fn test_execute_sequence_simple_and_error_handling() {
    let server = DesktopWrapper::new().await.unwrap();

    // --- Test: Empty sequence ---
    let args = ExecuteSequenceArgs {
        steps: vec![],
        ..Default::default()
    };
    let result = server.execute_sequence(Parameters(args)).await.unwrap();
    let result_json = get_result_json(result);
    assert_eq!(result_json["status"], "success");
    assert_eq!(result_json["executed_tools"], 0);

    // --- Test: Invalid tool with stop_on_error = true ---
    let args = ExecuteSequenceArgs {
        steps: vec![SequenceStep {
            tool_name: Some("non_existent_tool".to_string()),
            ..Default::default()
        }],
        stop_on_error: Some(true),
        ..Default::default()
    };
    let result = server.execute_sequence(Parameters(args)).await.unwrap();
    let result_json = get_result_json(result);
    assert_eq!(result_json["status"], "partial_success");
    assert_eq!(result_json["results"][0]["status"], "error");

    // --- Test: continue_on_error = true ---
    let args = ExecuteSequenceArgs {
        steps: vec![
            SequenceStep {
                tool_name: Some("invalid_tool".to_string()),
                continue_on_error: Some(true),
                ..Default::default()
            },
            SequenceStep {
                tool_name: Some("delay".to_string()),
                arguments: Some(json!({ "delay_ms": 1 })),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let result = server.execute_sequence(Parameters(args)).await.unwrap();
    let result_json = get_result_json(result);
    assert_eq!(result_json["status"], "completed_with_errors");
    assert_eq!(result_json["results"][0]["status"], "skipped");
    assert_eq!(result_json["results"][1]["status"], "success");
}

#[tokio::test]
async fn test_sequence_with_conditional_execution() {
    let server = DesktopWrapper::new().await.unwrap();

    // --- Test: Conditional steps ---
    let mut inputs = HashMap::new();
    inputs.insert("run_first_step".to_string(), json!(true));
    inputs.insert("run_second_step".to_string(), json!(false));

    let args = ExecuteSequenceArgs {
        steps: vec![
            SequenceStep {
                tool_name: Some("delay".to_string()),
                arguments: Some(json!({"delay_ms": 1})),
                r#if: Some("run_first_step".to_string()),
                ..Default::default()
            },
            SequenceStep {
                tool_name: Some("delay".to_string()),
                arguments: Some(json!({"delay_ms": 1})),
                r#if: Some("run_second_step".to_string()),
                ..Default::default()
            },
            SequenceStep {
                tool_name: Some("delay".to_string()),
                arguments: Some(json!({"delay_ms": 1})),
                r#if: Some("!run_second_step".to_string()), // Test negation
                ..Default::default()
            },
        ],
        inputs: Some(serde_json::to_value(inputs).unwrap()),
        ..Default::default()
    };

    let result = server.execute_sequence(Parameters(args)).await.unwrap();
    let result_json = get_result_json(result);

    assert_eq!(result_json["status"], "success");
    assert_eq!(result_json["results"][0]["status"], "skipped");
    assert_eq!(result_json["results"][1]["status"], "skipped");
    assert_eq!(result_json["results"][2]["status"], "skipped");
}

#[tokio::test]
async fn test_sequence_with_variable_substitution() {
    let server = DesktopWrapper::new().await.unwrap();

    // --- Test: Variable substitution ---
    let mut inputs = HashMap::new();
    inputs.insert("delay_amount".to_string(), json!(15));
    inputs.insert(
        "validation_selector_selector".to_string(),
        json!("#some-id"),
    );

    let args = ExecuteSequenceArgs {
        steps: vec![
            SequenceStep {
                tool_name: Some("delay".to_string()),
                arguments: Some(json!({"delay_ms": "{{delay_amount}}"})),
                ..Default::default()
            },
            SequenceStep {
                tool_name: Some("validate_element".to_string()),
                arguments: Some(json!({"selector": "{{validation_selector_selector}}"})),
                continue_on_error: Some(true),
                ..Default::default()
            },
        ],
        inputs: Some(serde_json::to_value(inputs).unwrap()),
        ..Default::default()
    };

    let result = server.execute_sequence(Parameters(args)).await.unwrap();
    let result_json = get_result_json(result);
    assert_eq!(result_json["status"].as_str(), Some("success"));
}
