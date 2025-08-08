use terminator_workflow_recorder::{
    events::{ClickEvent, UIElementMetadata, WorkflowEvent},
    mcp_converter::McpConverter,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("🧪 Testing Chrome-Specific MCP Converter Fix");
    info!("===========================================");

    let converter = McpConverter::new();

    // Create a synthetic Chrome click event to test our fix
    let chrome_metadata = UIElementMetadata {
        application_name: "Google Chrome".to_string(),
        window_title: "I-94/I-95 Website - Google Chrome".to_string(),
        element_name: "Search".to_string(),
        element_role: "Text".to_string(),
        ui_element: None, // We don't need the actual UI element for this test
    };

    let chrome_click = ClickEvent {
        x: 100,
        y: 200,
        button: terminator_workflow_recorder::events::MouseButton::Left,
        timestamp: std::time::SystemTime::now(),
        metadata: chrome_metadata,
    };

    // Test our Chrome fix
    info!("🔍 Testing Chrome application click conversion...");

    let workflow_event = WorkflowEvent::Click(chrome_click);
    let mcp_sequence = converter.convert_to_mcp_sequence(&[workflow_event]).await?;

    info!("📊 Generated MCP Sequence:");
    for (i, step) in mcp_sequence.iter().enumerate() {
        info!(
            "  Step {}: {} with selector: '{}'",
            i + 1,
            step.tool_name,
            step.arguments
                .get("selector")
                .unwrap_or(&serde_json::Value::String("N/A".to_string()))
        );

        // Check if our Chrome fix worked
        if let Some(selector) = step.arguments.get("selector").and_then(|s| s.as_str()) {
            if selector.contains("role:Pane") {
                info!("  ✅ SUCCESS! Chrome fix worked - using role:Pane");
            } else if selector.contains("role:Window") {
                info!("  ❌ FAILED! Still using role:Window for Chrome");
            }
        }
    }

    // Test a non-Chrome application for comparison
    info!("");
    info!("🔍 Testing non-Chrome application click conversion (for comparison)...");

    let notepad_metadata = UIElementMetadata {
        application_name: "Notepad".to_string(),
        window_title: "Untitled - Notepad".to_string(),
        element_name: "Text Editor".to_string(),
        element_role: "Document".to_string(),
        ui_element: None,
    };

    let notepad_click = ClickEvent {
        x: 100,
        y: 200,
        button: terminator_workflow_recorder::events::MouseButton::Left,
        timestamp: std::time::SystemTime::now(),
        metadata: notepad_metadata,
    };

    let notepad_event = WorkflowEvent::Click(notepad_click);
    let notepad_sequence = converter.convert_to_mcp_sequence(&[notepad_event]).await?;

    info!("📊 Generated MCP Sequence for Notepad:");
    for (i, step) in notepad_sequence.iter().enumerate() {
        info!(
            "  Step {}: {} with selector: '{}'",
            i + 1,
            step.tool_name,
            step.arguments
                .get("selector")
                .unwrap_or(&serde_json::Value::String("N/A".to_string()))
        );

        // This should still use Window for non-Chrome apps
        if let Some(selector) = step.arguments.get("selector").and_then(|s| s.as_str()) {
            if selector.contains("role:Window") {
                info!("  ✅ CORRECT! Non-Chrome app using role:Window as expected");
            } else if selector.contains("role:Pane") {
                info!("  ❌ UNEXPECTED! Non-Chrome app using role:Pane");
            }
        }
    }

    info!("");
    info!("🏁 Chrome Fix Test Complete!");

    Ok(())
}
