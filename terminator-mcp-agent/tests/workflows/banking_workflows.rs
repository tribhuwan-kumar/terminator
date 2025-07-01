use crate::workflow_accuracy_tests::*;
use rmcp::object;
use std::collections::HashMap;

/// Create online banking transfer workflow
pub fn create_bank_transfer_workflow() -> ComplexWorkflow {
    ComplexWorkflow {
        name: "Online Banking Wire Transfer".to_string(),
        description: "Log into bank account, set up and execute wire transfer with verification"
            .to_string(),
        category: WorkflowCategory::FormFilling,
        steps: vec![
            WorkflowStep {
                name: "Navigate to Bank Website".to_string(),
                tool_name: "navigate_browser".to_string(),
                arguments: object!({
                    "url": "https://www.chase.com",
                    "wait_for_load": true
                }),
                expected_outcome: ExpectedOutcome::NavigationCompleted {
                    url: "https://www.chase.com".to_string(),
                },
                validation_criteria: vec![ValidationCriterion::ElementExists {
                    selector: "selector:button:contains('Sign in')".to_string(),
                }],
                timeout_ms: 8000,
                retry_count: 2,
            },
            WorkflowStep {
                name: "Click Sign In".to_string(),
                tool_name: "click_element".to_string(),
                arguments: object!({
                    "selector": "selector:button:contains('Sign in')",
                    "wait_for_navigation": false
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "signin_clicked": true }),
                },
                validation_criteria: vec![ValidationCriterion::ElementExists {
                    selector: "id:userId".to_string(),
                }],
                timeout_ms: 5000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Enter Login Credentials".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:userId",
                                "text_to_type": "demo_user_12345",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:password",
                                "text_to_type": "SecurePass123!",
                                "clear_before_typing": true,
                                "mask_input": true
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "id:signin-button"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 300
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "login_submitted": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 10000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Handle 2FA if Present".to_string(),
                tool_name: "conditional_execute".to_string(),
                arguments: object!({
                    "condition": {
                        "tool_name": "element_exists",
                        "arguments": {
                            "selector": "selector:h1:contains('Verify your identity')"
                        }
                    },
                    "if_true": [
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:button:contains('Text me')"
                            }
                        },
                        {
                            "tool_name": "wait_for_element",
                            "arguments": {
                                "selector": "id:otp-input",
                                "timeout_ms": 5000
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:otp-input",
                                "text_to_type": "123456"
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:button:contains('Continue')"
                            }
                        }
                    ],
                    "timeout_ms": 15000
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "2fa_handled": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 20000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Navigate to Transfers".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "wait_for_element",
                            "arguments": {
                                "selector": "selector:nav",
                                "timeout_ms": 5000
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:a:contains('Pay & transfer')"
                            }
                        },
                        {
                            "tool_name": "wait_for_element",
                            "arguments": {
                                "selector": "selector:a:contains('Wire money')",
                                "timeout_ms": 3000
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:a:contains('Wire money')"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 500
                }),
                expected_outcome: ExpectedOutcome::NavigationCompleted {
                    url: "wire_transfer".to_string(),
                },
                validation_criteria: vec![ValidationCriterion::ElementExists {
                    selector: "selector:h1:contains('Wire')".to_string(),
                }],
                timeout_ms: 10000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Select Account and Enter Amount".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "select_option",
                            "arguments": {
                                "selector": "id:from-account",
                                "option_name": "Checking (...4567)"
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:amount",
                                "text_to_type": "1000.00",
                                "clear_before_typing": true
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
                    expected_data: object!({ "amount_entered": true }),
                },
                validation_criteria: vec![ValidationCriterion::ElementHasText {
                    selector: "id:amount".to_string(),
                    text: "1000.00".to_string(),
                }],
                timeout_ms: 8000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Enter Recipient Information".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:recipient-name",
                                "text_to_type": "ABC Corporation",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:recipient-account",
                                "text_to_type": "987654321",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:routing-number",
                                "text_to_type": "021000021",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:recipient-address",
                                "text_to_type": "123 Business Ave, New York, NY 10001",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:wire-purpose",
                                "text_to_type": "Invoice payment #INV-2024-001",
                                "clear_before_typing": true
                            }
                        }
                    ],
                    "delay_between_tools_ms": 200
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "recipient_info_complete": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 12000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Review and Confirm".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:button:contains('Review')"
                            }
                        },
                        {
                            "tool_name": "wait_for_element",
                            "arguments": {
                                "selector": "selector:h2:contains('Review your wire')",
                                "timeout_ms": 3000
                            }
                        },
                        {
                            "tool_name": "extract_element_text",
                            "arguments": {
                                "selector": "selector:.confirmation-amount",
                                "variable_name": "confirmed_amount"
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "id:confirm-checkbox"
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:button:contains('Send wire')"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 500
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "wire_sent": true }),
                },
                validation_criteria: vec![ValidationCriterion::ExactMatch {
                    field: "confirmed_amount".to_string(),
                    expected: "$1,000.00".to_string(),
                }],
                timeout_ms: 10000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Capture Confirmation".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "wait_for_element",
                            "arguments": {
                                "selector": "selector:.confirmation-message",
                                "timeout_ms": 5000
                            }
                        },
                        {
                            "tool_name": "extract_element_text",
                            "arguments": {
                                "selector": "selector:.confirmation-number",
                                "variable_name": "confirmation_number"
                            }
                        },
                        {
                            "tool_name": "take_screenshot",
                            "arguments": {
                                "filename": "wire_confirmation.png",
                                "selector": "selector:.confirmation-container"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 300
                }),
                expected_outcome: ExpectedOutcome::DataExtracted {
                    fields: HashMap::from([(
                        "confirmation_number".to_string(),
                        "WT123456789".to_string(),
                    )]),
                },
                validation_criteria: vec![ValidationCriterion::RegexMatch {
                    field: "confirmation_number".to_string(),
                    pattern: r"WT\d{9}".to_string(),
                }],
                timeout_ms: 8000,
                retry_count: 1,
            },
        ],
        test_data: TestData {
            input_files: vec![],
            expected_outputs: HashMap::from([
                ("wire_completed".to_string(), object!(true)),
                ("amount_transferred".to_string(), object!("1000.00")),
            ]),
            mock_data: HashMap::from([
                ("account_number".to_string(), "****4567".to_string()),
                ("routing_number".to_string(), "021000021".to_string()),
            ]),
        },
        accuracy_threshold: 90.0,
    }
}

/// Create credit card application workflow
pub fn create_credit_card_application_workflow() -> ComplexWorkflow {
    ComplexWorkflow {
        name: "Credit Card Application Process".to_string(),
        description: "Complete full credit card application with income verification".to_string(),
        category: WorkflowCategory::FormFilling,
        steps: vec![
            WorkflowStep {
                name: "Navigate to Credit Cards Page".to_string(),
                tool_name: "navigate_browser".to_string(),
                arguments: object!({
                    "url": "https://www.capitalone.com/credit-cards/",
                    "wait_for_load": true
                }),
                expected_outcome: ExpectedOutcome::NavigationCompleted {
                    url: "https://www.capitalone.com/credit-cards/".to_string(),
                },
                validation_criteria: vec![ValidationCriterion::ElementExists {
                    selector: "selector:h1:contains('Credit Cards')".to_string(),
                }],
                timeout_ms: 8000,
                retry_count: 2,
            },
            WorkflowStep {
                name: "Select Card Type".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "scroll_to_element",
                            "arguments": {
                                "selector": "selector:.card-tile:contains('Venture')"
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:.card-tile:contains('Venture') button:contains('Apply')"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 500
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "card_selected": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 8000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Fill Personal Information".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:firstName",
                                "text_to_type": "Jane",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:lastName",
                                "text_to_type": "Smith",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:dateOfBirth",
                                "text_to_type": "03/15/1985",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:ssn",
                                "text_to_type": "123-45-6789",
                                "clear_before_typing": true,
                                "mask_input": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:mothersMaidenName",
                                "text_to_type": "Johnson",
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
                    selector: "id:firstName".to_string(),
                    text: "Jane".to_string(),
                }],
                timeout_ms: 12000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Fill Contact Information".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:email",
                                "text_to_type": "jane.smith@email.com",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:confirmEmail",
                                "text_to_type": "jane.smith@email.com",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:phoneNumber",
                                "text_to_type": "555-234-5678",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:streetAddress",
                                "text_to_type": "456 Oak Street",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:city",
                                "text_to_type": "San Francisco",
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
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:zipCode",
                                "text_to_type": "94102",
                                "clear_before_typing": true
                            }
                        }
                    ],
                    "delay_between_tools_ms": 200
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "contact_info_complete": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 15000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Fill Financial Information".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:annualIncome",
                                "text_to_type": "85000",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "select_option",
                            "arguments": {
                                "selector": "id:employmentStatus",
                                "option_name": "Employed"
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:employer",
                                "text_to_type": "Tech Solutions Inc",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:monthlyRent",
                                "text_to_type": "2000",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:input[name='bankAccount'][value='checking']",
                                "element_type": "checkbox"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 200
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "financial_info_complete": true }),
                },
                validation_criteria: vec![ValidationCriterion::NumericRange {
                    field: "annual_income".to_string(),
                    min: 50000.0,
                    max: 150000.0,
                }],
                timeout_ms: 10000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Review and Submit".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "id:agreeTerms",
                                "element_type": "checkbox"
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "id:electronicConsent",
                                "element_type": "checkbox"
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:button:contains('Submit Application')"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 300
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "application_submitted": true }),
                },
                validation_criteria: vec![],
                timeout_ms: 8000,
                retry_count: 2,
            },
            WorkflowStep {
                name: "Capture Decision".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "wait_for_element",
                            "arguments": {
                                "selector": "selector:.decision-container",
                                "timeout_ms": 30000
                            }
                        },
                        {
                            "tool_name": "extract_element_text",
                            "arguments": {
                                "selector": "selector:.decision-status",
                                "variable_name": "application_status"
                            }
                        },
                        {
                            "tool_name": "extract_element_text",
                            "arguments": {
                                "selector": "selector:.reference-number",
                                "variable_name": "reference_number"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 500
                }),
                expected_outcome: ExpectedOutcome::DataExtracted {
                    fields: HashMap::from([(
                        "application_status".to_string(),
                        "Approved".to_string(),
                    )]),
                },
                validation_criteria: vec![ValidationCriterion::PartialMatch {
                    field: "application_status".to_string(),
                    contains: "pproved".to_string(),
                }],
                timeout_ms: 35000,
                retry_count: 1,
            },
        ],
        test_data: TestData {
            input_files: vec![],
            expected_outputs: HashMap::from([
                ("application_completed".to_string(), object!(true)),
                ("decision_received".to_string(), object!(true)),
            ]),
            mock_data: HashMap::from([
                ("ssn".to_string(), "123-45-6789".to_string()),
                ("income".to_string(), "85000".to_string()),
            ]),
        },
        accuracy_threshold: 85.0,
    }
}
