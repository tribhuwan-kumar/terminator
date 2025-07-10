use rmcp::model::CallToolResult;
use serde_json::Value;

pub fn get_result_json(result: CallToolResult) -> Value {
    let content = result.content.first().expect("Result content is empty");
    let serialized_content =
        serde_json::to_value(content).expect("Failed to serialize content to JSON");

    if let Some(text) = serialized_content.get("text").and_then(|v| v.as_str()) {
        serde_json::from_str(text).unwrap_or_else(|e| {
            panic!("Failed to parse inner JSON string from text field. Error: {e}. Text: {text}")
        })
    } else {
        panic!("Expected serialized content to have a 'text' field, but got: {serialized_content}")
    }
}
