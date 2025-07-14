use terminator_mcp_agent::utils::{RunJavascriptArgs};
use terminator_mcp_agent::utils::DesktopWrapper;
use rmcp::handler::server::tool::Parameters;
use serde_json::json;

#[tokio::test]
async fn test_basic_js_math() {
    let server = DesktopWrapper::new().await.unwrap();
    let args = RunJavascriptArgs { script: Some("1 + 1".to_string()), workflow_yaml: None, engine: None, timeout_ms: None };
    let result = server.run_javascript(Parameters(args)).await.unwrap();
    let content = &result.content[0];
    let res_json = content.as_json().unwrap();
    assert_eq!(res_json["result"], json!(2));
}

#[tokio::test]
async fn test_workflow_yaml() {
    let server = DesktopWrapper::new().await.unwrap();
    let yaml = "steps:\n  - run: 2 + 3".to_string();
    let args = RunJavascriptArgs { script: None, workflow_yaml: Some(yaml), engine: None, timeout_ms: None };
    let result = server.run_javascript(Parameters(args)).await.unwrap();
    let content = &result.content[0];
    let res_json = content.as_json().unwrap();
    assert_eq!(res_json["result"]["workflow_steps"][0], json!(5));
}