use terminator_mcp_agent::utils::{RunJavascriptArgs};
use terminator_mcp_agent::utils::DesktopWrapper;
use rmcp::handler::server::tool::Parameters;
use serde_json::json;

#[tokio::test]
async fn test_basic_js_math() {
    let server = DesktopWrapper::new().await.unwrap();
    let args = RunJavascriptArgs { script: "1 + 1".to_string(), engine: None, timeout_ms: None };
    let result = server.run_javascript(Parameters(args)).await.unwrap();
    let content = &result.content[0];
    let res_json = content.as_json().unwrap();
    assert_eq!(res_json["result"], json!(2));
}