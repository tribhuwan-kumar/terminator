use crate::workflow_accuracy_tests::*;
use rmcp::object;
use std::collections::HashMap;

/// Create a DMV appointment scheduling workflow
pub fn create_dmv_appointment_workflow() -> ComplexWorkflow {
    ComplexWorkflow {
        name: "DMV Online Appointment Scheduling".to_string(),
        description: "Schedule a DMV appointment for license renewal through official website"
            .to_string(),
        category: WorkflowCategory::FormFilling,
        steps: vec![
            WorkflowStep {
                name: "Navigate to DMV Website".to_string(),
                tool_name: "navigate_browser".to_string(),
                arguments: object!({
                    "url": "https://www.dmv.ca.gov",
                    "wait_for_load": true
                }),
                expected_outcome: ExpectedOutcome::NavigationCompleted {
                    url: "https://www.dmv.ca.gov".to_string(),
                },
                validation_criteria: vec![ValidationCriterion::PartialMatch {
                    field: "title".to_string(),
                    contains: "DMV".to_string(),
                }],
                timeout_ms: 8000,
                retry_count: 2,
            },
            WorkflowStep {
                name: "Navigate to Appointments".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:a:contains('Appointments')",
                                "alternative_selectors": "selector:button:contains('Schedule'),name:Appointments"
                            }
                        },
                        {
                            "tool_name": "wait_for_element",
                            "arguments": {
                                "selector": "selector:h1:contains('Appointment')",
                                "timeout_ms": 5000
                            }
                        }
                    ],
                    "delay_between_tools_ms": 500
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "appointments_page_loaded": true }),
                },
                validation_criteria: vec![ValidationCriterion::ElementExists {
                    selector: "selector:button:contains('Schedule')".to_string(),
                }],
                timeout_ms: 10000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Select Service Type".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:input[value='DL']",
                                "element_type": "radio"
                            }
                        },
                        {
                            "tool_name": "select_option",
                            "arguments": {
                                "selector": "id:task_select",
                                "option_name": "Renew Driver License"
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:button:contains('Continue')"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 300
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "service_selected": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 8000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Enter Personal Information".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:first_name",
                                "text_to_type": "John",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:last_name",
                                "text_to_type": "Smith",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:dl_number",
                                "text_to_type": "D1234567",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:phone_number",
                                "text_to_type": "555-123-4567",
                                "clear_before_typing": true
                            }
                        }
                    ],
                    "delay_between_tools_ms": 200
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "info_entered": true }),
                },
                validation_criteria: vec![ValidationCriterion::ElementHasText {
                    selector: "id:first_name".to_string(),
                    text: "John".to_string(),
                }],
                timeout_ms: 10000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Select Office Location".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:zip_code",
                                "text_to_type": "90210",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:button:contains('Find Offices')"
                            }
                        },
                        {
                            "tool_name": "wait_for_element",
                            "arguments": {
                                "selector": "selector:.office-list",
                                "timeout_ms": 3000
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:.office-item:first-child input[type='radio']"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 500
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "office_selected": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 10000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Select Date and Time".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:.calendar-day.available:first"
                            }
                        },
                        {
                            "tool_name": "wait_for_element",
                            "arguments": {
                                "selector": "selector:.time-slot",
                                "timeout_ms": 2000
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:.time-slot:contains('10:00 AM')"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 300
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "appointment_time_selected": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 8000,
                retry_count: 1,
            },
        ],
        test_data: TestData {
            input_files: vec![],
            expected_outputs: HashMap::from([("appointment_scheduled".to_string(), object!(true))]),
            mock_data: HashMap::from([
                ("dl_number".to_string(), "D1234567".to_string()),
                ("zip_code".to_string(), "90210".to_string()),
            ]),
        },
        accuracy_threshold: 85.0,
    }
}

/// Create IRS tax form submission workflow
pub fn create_irs_tax_form_workflow() -> ComplexWorkflow {
    ComplexWorkflow {
        name: "IRS Tax Form 1040 E-Filing".to_string(),
        description: "Complete and submit tax form through IRS Free File system".to_string(),
        category: WorkflowCategory::FormFilling,
        steps: vec![
            WorkflowStep {
                name: "Navigate to IRS Website".to_string(),
                tool_name: "navigate_browser".to_string(),
                arguments: object!({
                    "url": "https://www.irs.gov/filing/free-file-do-your-federal-taxes-for-free",
                    "wait_for_load": true
                }),
                expected_outcome: ExpectedOutcome::NavigationCompleted {
                    url: "https://www.irs.gov".to_string(),
                },
                validation_criteria: vec![ValidationCriterion::PartialMatch {
                    field: "title".to_string(),
                    contains: "IRS".to_string(),
                }],
                timeout_ms: 8000,
                retry_count: 2,
            },
            WorkflowStep {
                name: "Start Free File Process".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:a:contains('Start Free File')"
                            }
                        },
                        {
                            "tool_name": "wait_for_element",
                            "arguments": {
                                "selector": "selector:h1:contains('Choose')",
                                "timeout_ms": 5000
                            }
                        }
                    ],
                    "delay_between_tools_ms": 500
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "free_file_started": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 10000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Answer Qualification Questions".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "select_option",
                            "arguments": {
                                "selector": "id:income_range",
                                "option_name": "$50,000 - $75,000"
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:input[value='single']",
                                "element_type": "radio"
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:age",
                                "text_to_type": "35",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "select_option",
                            "arguments": {
                                "selector": "id:state",
                                "option_name": "California"
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:button:contains('Find My Options')"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 300
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "qualification_complete": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 12000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Select Tax Software Provider".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "wait_for_element",
                            "arguments": {
                                "selector": "selector:.provider-list",
                                "timeout_ms": 3000
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:.provider-card:first-child button:contains('Select')"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 500
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "provider_selected": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 8000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Begin Tax Return".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "wait_for_navigation",
                            "arguments": {
                                "timeout_ms": 5000
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:button:contains('Start My Return')"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 1000
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "return_started": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 10000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Enter Personal Information".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:ssn",
                                "text_to_type": "123-45-6789",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:first_name",
                                "text_to_type": "John",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:last_name",
                                "text_to_type": "Taxpayer",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:dob",
                                "text_to_type": "01/15/1988",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:address",
                                "text_to_type": "123 Main Street",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:city",
                                "text_to_type": "Los Angeles",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:zip",
                                "text_to_type": "90001",
                                "clear_before_typing": true
                            }
                        }
                    ],
                    "delay_between_tools_ms": 200
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "personal_info_complete": true }),
                },
                validation_criteria: vec![ValidationCriterion::ElementHasText {
                    selector: "id:first_name".to_string(),
                    text: "John".to_string(),
                }],
                timeout_ms: 15000,
                retry_count: 1,
            },
        ],
        test_data: TestData {
            input_files: vec![],
            expected_outputs: HashMap::from([("tax_form_started".to_string(), object!(true))]),
            mock_data: HashMap::from([
                ("ssn".to_string(), "123-45-6789".to_string()),
                ("income".to_string(), "65000".to_string()),
            ]),
        },
        accuracy_threshold: 80.0,
    }
}
