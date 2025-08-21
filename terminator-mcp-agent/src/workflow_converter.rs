//! Workflow conversion module for converting raw recorded workflows to MCP sequences

use crate::mcp_converter::{ConversionConfig, McpConverter};
use crate::workflow_events::{McpToolStep, RecordedWorkflow, WorkflowEvent};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// MCP-converted workflow with tool sequences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpWorkflow {
    /// The workflow steps as MCP tool calls
    pub steps: Vec<McpToolStep>,

    /// Metadata from the original workflow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<WorkflowMetadata>,
}

/// Metadata about the workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub event_count: usize,
    pub conversion_notes: Vec<String>,
}

/// Convert a raw recorded workflow to MCP tool sequences
pub async fn convert_workflow_to_mcp(workflow: RecordedWorkflow) -> Result<McpWorkflow> {
    info!("Converting workflow to MCP sequences: {}", &workflow.name);

    let converter = McpConverter::new();
    let mut mcp_steps = Vec::new();
    let mut conversion_notes = Vec::new();

    // Process each event in the workflow
    for (index, event) in workflow.events.iter().enumerate() {
        debug!(
            "Converting event {} of {}",
            index + 1,
            workflow.events.len()
        );

        match converter.convert_event(&event.event, None).await {
            Ok(conversion_result) => {
                // Add the primary sequence
                mcp_steps.extend(conversion_result.primary_sequence);

                // Log semantic action for debugging
                debug!("Event {}: {}", index + 1, conversion_result.semantic_action);

                // Collect any conversion notes
                if !conversion_result.conversion_notes.is_empty() {
                    conversion_notes.extend(conversion_result.conversion_notes);
                }
            }
            Err(e) => {
                warn!("Failed to convert event {}: {}", index + 1, e);
                conversion_notes.push(format!("Event {} conversion failed: {}", index + 1, e));

                // Continue with next event instead of failing the entire workflow
                continue;
            }
        }
    }

    info!(
        "Conversion complete: {} events -> {} MCP steps",
        workflow.events.len(),
        mcp_steps.len()
    );

    Ok(McpWorkflow {
        steps: mcp_steps,
        metadata: Some(WorkflowMetadata {
            name: Some(workflow.name.clone()),
            description: None,
            created_at: Some(format!("{}", workflow.start_time)),
            event_count: workflow.events.len(),
            conversion_notes,
        }),
    })
}

/// Load a workflow from a JSON file and convert it to MCP
pub async fn load_and_convert_workflow(file_path: &str) -> Result<McpWorkflow> {
    info!("Loading workflow from: {}", file_path);

    // Read the JSON file
    let content = std::fs::read_to_string(file_path)?;

    // Parse as RecordedWorkflow (from recorder crate)
    let workflow: RecordedWorkflow = serde_json::from_str(&content)?;

    // Convert to MCP sequences
    convert_workflow_to_mcp(workflow).await
}

/// Convert a workflow with custom configuration
pub async fn convert_workflow_with_config(
    workflow: RecordedWorkflow,
    config: ConversionConfig,
) -> Result<McpWorkflow> {
    info!("Converting workflow with custom config");

    let converter = McpConverter::with_config(config);
    let mut mcp_steps = Vec::new();
    let mut conversion_notes = Vec::new();

    for (index, event) in workflow.events.iter().enumerate() {
        match converter.convert_event(&event.event, None).await {
            Ok(conversion_result) => {
                mcp_steps.extend(conversion_result.primary_sequence);

                if !conversion_result.conversion_notes.is_empty() {
                    conversion_notes.extend(conversion_result.conversion_notes);
                }
            }
            Err(e) => {
                warn!("Failed to convert event {}: {}", index + 1, e);
                conversion_notes.push(format!("Event {} conversion failed: {}", index + 1, e));
                continue;
            }
        }
    }

    Ok(McpWorkflow {
        steps: mcp_steps,
        metadata: Some(WorkflowMetadata {
            name: Some(workflow.name.clone()),
            description: None,
            created_at: Some(format!("{}", workflow.start_time)),
            event_count: workflow.events.len(),
            conversion_notes,
        }),
    })
}

/// Validate that a workflow can be converted successfully
pub async fn validate_workflow_conversion(workflow: &RecordedWorkflow) -> Result<Vec<String>> {
    let converter = McpConverter::new();
    let mut validation_notes = Vec::new();

    for (index, event) in workflow.events.iter().enumerate() {
        match converter.convert_event(&event.event, None).await {
            Ok(result) => {
                if result.primary_sequence.is_empty() {
                    validation_notes.push(format!(
                        "Event {}: No MCP sequence generated for {:?}",
                        index + 1,
                        event_type_name(&event.event)
                    ));
                }
            }
            Err(e) => {
                validation_notes.push(format!("Event {}: Conversion error - {}", index + 1, e));
            }
        }
    }

    Ok(validation_notes)
}

fn event_type_name(event: &WorkflowEvent) -> &'static str {
    match event {
        WorkflowEvent::Click(_) => "Click",
        WorkflowEvent::Keyboard(_) => "Keyboard",
        WorkflowEvent::Mouse(_) => "Mouse",
        WorkflowEvent::Clipboard(_) => "Clipboard",
        WorkflowEvent::TextSelection(_) => "TextSelection",
        WorkflowEvent::DragDrop(_) => "DragDrop",
        WorkflowEvent::Hotkey(_) => "Hotkey",
        WorkflowEvent::TextInputCompleted(_) => "TextInputCompleted",
        WorkflowEvent::ApplicationSwitch(_) => "ApplicationSwitch",
        WorkflowEvent::BrowserTabNavigation(_) => "BrowserTabNavigation",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow_events::{
        ButtonInteractionType, ClickEvent, EventMetadata, Position, RecordedEvent,
    };

    #[tokio::test]
    async fn test_simple_workflow_conversion() {
        // Create a simple workflow with a click event
        let click_event = WorkflowEvent::Click(ClickEvent {
            element_text: "Test Button".to_string(),
            interaction_type: ButtonInteractionType::Click,
            element_role: "Button".to_string(),
            was_enabled: true,
            click_position: Some(Position { x: 100, y: 200 }),
            element_description: Some("Test button description".to_string()),
            child_text_content: vec![],
            metadata: EventMetadata::empty(),
        });

        let workflow = RecordedWorkflow {
            name: "Test Workflow".to_string(),
            start_time: 1000,
            end_time: Some(2000),
            events: vec![RecordedEvent {
                timestamp: 1000,
                event: click_event,
                metadata: None,
            }],
        };

        let result = convert_workflow_to_mcp(workflow).await.unwrap();

        assert!(!result.steps.is_empty());
        assert_eq!(result.metadata.as_ref().unwrap().event_count, 1);
    }
}
