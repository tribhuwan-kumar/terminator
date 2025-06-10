use terminator_workflow_recorder::{
    TextInputCompletedEvent, TextInputMethod, EventMetadata, WorkflowEvent
};

/// Unit tests for text input completion functionality
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_completed_event_creation() {
        let event = TextInputCompletedEvent {
            text_value: "Hello World".to_string(),
            field_name: Some("Test Field".to_string()),
            field_type: "Edit".to_string(),
            input_method: TextInputMethod::Typed,
            typing_duration_ms: 1500,
            keystroke_count: 11,
            metadata: EventMetadata { ui_element: None },
        };

        assert_eq!(event.text_value, "Hello World");
        assert_eq!(event.field_name, Some("Test Field".to_string()));
        assert_eq!(event.field_type, "Edit");
        assert_eq!(event.input_method, TextInputMethod::Typed);
        assert_eq!(event.typing_duration_ms, 1500);
        assert_eq!(event.keystroke_count, 11);
    }

    #[test]
    fn test_text_input_method_variants() {
        let typed = TextInputMethod::Typed;
        let pasted = TextInputMethod::Pasted;
        let auto_filled = TextInputMethod::AutoFilled;
        let mixed = TextInputMethod::Mixed;

        // Verify all variants are different
        assert_ne!(typed, pasted);
        assert_ne!(typed, auto_filled);
        assert_ne!(typed, mixed);
        assert_ne!(pasted, auto_filled);
        assert_ne!(pasted, mixed);
        assert_ne!(auto_filled, mixed);
    }

    #[test]
    fn test_workflow_event_text_input_completion_variant() {
        let text_event = TextInputCompletedEvent {
            text_value: "user@example.com".to_string(),
            field_name: Some("Email".to_string()),
            field_type: "Edit".to_string(),
            input_method: TextInputMethod::Pasted,
            typing_duration_ms: 100,
            keystroke_count: 2,
            metadata: EventMetadata { ui_element: None },
        };

        let workflow_event = WorkflowEvent::TextInputCompleted(text_event);
        
        // Verify it's the correct variant
        match workflow_event {
            WorkflowEvent::TextInputCompleted(event) => {
                assert_eq!(event.text_value, "user@example.com");
                assert_eq!(event.input_method, TextInputMethod::Pasted);
            }
            _ => panic!("Expected TextInputCompleted variant"),
        }
    }

    #[test]
    fn test_text_input_serialization() {
        let event = TextInputCompletedEvent {
            text_value: "test content".to_string(),
            field_name: Some("Username".to_string()),
            field_type: "Edit".to_string(),
            input_method: TextInputMethod::Typed,
            typing_duration_ms: 2000,
            keystroke_count: 12,
            metadata: EventMetadata { ui_element: None },
        };

        let workflow_event = WorkflowEvent::TextInputCompleted(event);
        
        // Test serialization
        let json = serde_json::to_string(&workflow_event).expect("Failed to serialize");
        assert!(json.contains("TextInputCompleted"));
        assert!(json.contains("test content"));
        assert!(json.contains("Username"));
        assert!(json.contains("Typed"));

        // Test deserialization
        let deserialized: WorkflowEvent = serde_json::from_str(&json)
            .expect("Failed to deserialize");
        
        match deserialized {
            WorkflowEvent::TextInputCompleted(event) => {
                assert_eq!(event.text_value, "test content");
                assert_eq!(event.field_name, Some("Username".to_string()));
                assert_eq!(event.input_method, TextInputMethod::Typed);
            }
            _ => panic!("Expected TextInputCompleted variant after deserialization"),
        }
    }

    #[test]
    fn test_empty_text_handling() {
        let event = TextInputCompletedEvent {
            text_value: "".to_string(),
            field_name: None,
            field_type: "Edit".to_string(),
            input_method: TextInputMethod::Typed,
            typing_duration_ms: 0,
            keystroke_count: 0,
            metadata: EventMetadata { ui_element: None },
        };

        assert!(event.text_value.is_empty());
        assert_eq!(event.field_name, None);
        assert_eq!(event.keystroke_count, 0);
    }

    #[test]
    fn test_long_text_handling() {
        let long_text = "a".repeat(1000);
        let event = TextInputCompletedEvent {
            text_value: long_text.clone(),
            field_name: Some("Large Text Field".to_string()),
            field_type: "TextArea".to_string(),
            input_method: TextInputMethod::Pasted,
            typing_duration_ms: 50,
            keystroke_count: 1,
            metadata: EventMetadata { ui_element: None },
        };

        assert_eq!(event.text_value.len(), 1000);
        assert_eq!(event.text_value, long_text);
        assert_eq!(event.input_method, TextInputMethod::Pasted);
        assert!(event.typing_duration_ms < 100); // Fast for pasted content
    }
} 