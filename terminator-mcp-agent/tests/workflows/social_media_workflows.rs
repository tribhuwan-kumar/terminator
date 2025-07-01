use crate::workflow_accuracy_tests::*;
use rmcp::object;
use std::collections::HashMap;

/// Create LinkedIn job application workflow
pub fn create_linkedin_job_application_workflow() -> ComplexWorkflow {
    ComplexWorkflow {
        name: "LinkedIn Easy Apply Job Application".to_string(),
        description: "Search for jobs and apply using LinkedIn Easy Apply feature".to_string(),
        category: WorkflowCategory::FormFilling,
        steps: vec![
            WorkflowStep {
                name: "Navigate to LinkedIn".to_string(),
                tool_name: "navigate_browser".to_string(),
                arguments: object!({
                    "url": "https://www.linkedin.com/jobs",
                    "wait_for_load": true
                }),
                expected_outcome: ExpectedOutcome::NavigationCompleted {
                    url: "https://www.linkedin.com/jobs".to_string(),
                },
                validation_criteria: vec![ValidationCriterion::ElementExists {
                    selector: "selector:input[placeholder*='Search']".to_string(),
                }],
                timeout_ms: 8000,
                retry_count: 2,
            },
            WorkflowStep {
                name: "Search for Jobs".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "selector:input[placeholder*='job title']",
                                "text_to_type": "Software Engineer",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "selector:input[placeholder*='location']",
                                "text_to_type": "San Francisco, CA",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "send_keys",
                            "arguments": {
                                "keys": "Enter"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 300
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "search_performed": true }),
                },
                validation_criteria: vec![ValidationCriterion::ElementExists {
                    selector: "selector:.jobs-search-results".to_string(),
                }],
                timeout_ms: 10000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Filter Easy Apply Jobs".to_string(),
                tool_name: "click_element".to_string(),
                arguments: object!({
                    "selector": "selector:button[aria-label*='Easy Apply']",
                    "wait_for_update": true
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "filter_applied": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 5000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Select First Job".to_string(),
                tool_name: "click_element".to_string(),
                arguments: object!({
                    "selector": "selector:.job-card-container:first-child",
                    "wait_for_update": true
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "job_selected": true }),
                },
                validation_criteria: vec![ValidationCriterion::ElementExists {
                    selector: "selector:button:contains('Easy Apply')".to_string(),
                }],
                timeout_ms: 5000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Click Easy Apply".to_string(),
                tool_name: "click_element".to_string(),
                arguments: object!({
                    "selector": "selector:button:contains('Easy Apply')",
                    "wait_for_modal": true
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "application_started": true }),
                },
                validation_criteria: vec![ValidationCriterion::ElementExists {
                    selector: "selector:.jobs-easy-apply-modal".to_string(),
                }],
                timeout_ms: 5000,
                retry_count: 2,
            },
            WorkflowStep {
                name: "Fill Application Form".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:easyApplyFormElement-phoneNumber",
                                "text_to_type": "555-123-4567",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:button[aria-label='Continue to next step']"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 500
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "form_filled": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 8000,
                retry_count: 1,
            },
        ],
        test_data: TestData {
            input_files: vec![],
            expected_outputs: HashMap::from([("application_submitted".to_string(), object!(true))]),
            mock_data: HashMap::new(),
        },
        accuracy_threshold: 80.0,
    }
}

/// Create Twitter/X posting workflow
pub fn create_twitter_posting_workflow() -> ComplexWorkflow {
    ComplexWorkflow {
        name: "Twitter/X Thread Creation".to_string(),
        description: "Create and post a multi-tweet thread with media".to_string(),
        category: WorkflowCategory::DataEntry,
        steps: vec![
            WorkflowStep {
                name: "Navigate to Twitter".to_string(),
                tool_name: "navigate_browser".to_string(),
                arguments: object!({
                    "url": "https://twitter.com/compose/tweet",
                    "wait_for_load": true
                }),
                expected_outcome: ExpectedOutcome::NavigationCompleted {
                    url: "https://twitter.com".to_string(),
                },
                validation_criteria: vec![ValidationCriterion::ElementExists {
                    selector: "selector:[data-testid='tweetTextarea_0']".to_string(),
                }],
                timeout_ms: 8000,
                retry_count: 2,
            },
            WorkflowStep {
                name: "Compose First Tweet".to_string(),
                tool_name: "type_into_element".to_string(),
                arguments: object!({
                    "selector": "selector:[data-testid='tweetTextarea_0']",
                    "text_to_type": "🚀 Excited to share our latest product update! Here's what's new in version 2.0:\n\n1/5",
                    "clear_before_typing": true
                }),
                expected_outcome: ExpectedOutcome::TextEntered {
                    field: "tweet_text".to_string(),
                    value: "tweet_content".to_string(),
                },
                validation_criteria: vec![],
                timeout_ms: 5000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Add Thread".to_string(),
                tool_name: "click_element".to_string(),
                arguments: object!({
                    "selector": "selector:[data-testid='addButton']",
                    "wait_for_update": true
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "thread_extended": true }),
                },
                validation_criteria: vec![ValidationCriterion::ElementExists {
                    selector: "selector:[data-testid='tweetTextarea_1']".to_string(),
                }],
                timeout_ms: 3000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Compose Second Tweet".to_string(),
                tool_name: "type_into_element".to_string(),
                arguments: object!({
                    "selector": "selector:[data-testid='tweetTextarea_1']",
                    "text_to_type": "✨ New Features:\n- Real-time collaboration\n- Advanced analytics dashboard\n- API v2 support\n- Mobile app improvements\n\n2/5",
                    "clear_before_typing": true
                }),
                expected_outcome: ExpectedOutcome::TextEntered {
                    field: "tweet_text_2".to_string(),
                    value: "features_list".to_string(),
                },
                validation_criteria: vec![],
                timeout_ms: 5000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Post Thread".to_string(),
                tool_name: "click_element".to_string(),
                arguments: object!({
                    "selector": "selector:[data-testid='tweetButtonInline']",
                    "wait_for_navigation": true
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "thread_posted": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 5000,
                retry_count: 2,
            },
        ],
        test_data: TestData {
            input_files: vec![],
            expected_outputs: HashMap::from([("thread_created".to_string(), object!(true))]),
            mock_data: HashMap::new(),
        },
        accuracy_threshold: 85.0,
    }
}
