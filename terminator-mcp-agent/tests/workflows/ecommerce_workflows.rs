use crate::workflow_accuracy_tests::*;
use rmcp::object;
use std::collections::HashMap;

/// Create an Amazon product search and comparison workflow
pub fn create_amazon_shopping_workflow() -> ComplexWorkflow {
    ComplexWorkflow {
        name: "Amazon Product Search and Price Comparison".to_string(),
        description: "Search for a product, compare prices, read reviews, and add to cart".to_string(),
        category: WorkflowCategory::WebNavigation,
        steps: vec![
            WorkflowStep {
                name: "Open Chrome Browser".to_string(),
                tool_name: "open_application".to_string(),
                arguments: object!({
                    "application_name": "Google Chrome",
                    "wait_for_ready": true,
                    "timeout_ms": 5000
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "status": "browser_opened" })
                },
                validation_criteria: vec![
                    ValidationCriterion::ElementExists {
                        selector: "name:Address and search bar".to_string(),
                    },
                ],
                timeout_ms: 5000,
                retry_count: 2,
            },
            WorkflowStep {
                name: "Navigate to Amazon".to_string(),
                tool_name: "navigate_browser".to_string(),
                arguments: object!({
                    "url": "https://www.amazon.com",
                    "wait_for_load": true
                }),
                expected_outcome: ExpectedOutcome::NavigationCompleted {
                    url: "https://www.amazon.com".to_string()
                },
                validation_criteria: vec![
                    ValidationCriterion::ElementExists {
                        selector: "id:twotabsearchtextbox".to_string(),
                    },
                    ValidationCriterion::PartialMatch {
                        field: "title".to_string(),
                        contains: "Amazon".to_string(),
                    },
                ],
                timeout_ms: 8000,
                retry_count: 2,
            },
            WorkflowStep {
                name: "Search for Product".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "id:twotabsearchtextbox"
                            }
                        },
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:twotabsearchtextbox",
                                "text_to_type": "wireless noise cancelling headphones",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "id:nav-search-submit-button"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 500
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "search_completed": true })
                },
                validation_criteria: vec![
                    ValidationCriterion::ElementExists {
                        selector: "selector:[data-component-type='s-search-result']".to_string(),
                    },
                ],
                timeout_ms: 10000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Apply Filters".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:span:contains('4 Stars & Up')"
                            }
                        },
                        {
                            "tool_name": "wait_for_element",
                            "arguments": {
                                "selector": "selector:.s-breadcrumb",
                                "timeout_ms": 3000
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:span:contains('$50 to $100')"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 1000,
                    "stop_on_error": false
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "filters_applied": 2 })
                },
                validation_criteria: vec![
                    ValidationCriterion::ElementExists {
                        selector: "selector:.s-breadcrumb".to_string(),
                    },
                ],
                timeout_ms: 8000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Extract Product Information".to_string(),
                tool_name: "extract_elements_data".to_string(),
                arguments: object!({
                    "elements": [
                        {
                            "selector": "selector:[data-component-type='s-search-result']:nth-child(1) h2",
                            "attribute": "text",
                            "name": "product_1_name"
                        },
                        {
                            "selector": "selector:[data-component-type='s-search-result']:nth-child(1) .a-price-whole",
                            "attribute": "text",
                            "name": "product_1_price"
                        },
                        {
                            "selector": "selector:[data-component-type='s-search-result']:nth-child(1) .a-icon-alt",
                            "attribute": "text",
                            "name": "product_1_rating"
                        },
                        {
                            "selector": "selector:[data-component-type='s-search-result']:nth-child(2) h2",
                            "attribute": "text",
                            "name": "product_2_name"
                        },
                        {
                            "selector": "selector:[data-component-type='s-search-result']:nth-child(2) .a-price-whole",
                            "attribute": "text",
                            "name": "product_2_price"
                        }
                    ],
                    "wait_for_elements": true
                }),
                expected_outcome: ExpectedOutcome::DataExtracted {
                    fields: HashMap::from([
                        ("product_count".to_string(), "5".to_string()),
                    ])
                },
                validation_criteria: vec![
                    ValidationCriterion::RegexMatch {
                        field: "product_1_price".to_string(),
                        pattern: r"\d+".to_string(),
                    },
                    ValidationCriterion::PartialMatch {
                        field: "product_1_rating".to_string(),
                        contains: "stars".to_string(),
                    },
                ],
                timeout_ms: 5000,
                retry_count: 2,
            },
            WorkflowStep {
                name: "Click First Product".to_string(),
                tool_name: "click_element".to_string(),
                arguments: object!({
                    "selector": "selector:[data-component-type='s-search-result']:nth-child(1) h2 a",
                    "wait_for_navigation": true
                }),
                expected_outcome: ExpectedOutcome::NavigationCompleted {
                    url: "product_page".to_string()
                },
                validation_criteria: vec![
                    ValidationCriterion::ElementExists {
                        selector: "id:add-to-cart-button".to_string(),
                    },
                ],
                timeout_ms: 8000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Check Product Details".to_string(),
                tool_name: "extract_elements_data".to_string(),
                arguments: object!({
                    "elements": [
                        {
                            "selector": "id:productTitle",
                            "attribute": "text",
                            "name": "product_title"
                        },
                        {
                            "selector": "selector:.a-price-whole",
                            "attribute": "text",
                            "name": "product_price"
                        },
                        {
                            "selector": "id:availability span",
                            "attribute": "text",
                            "name": "availability"
                        },
                        {
                            "selector": "selector:#acrCustomerReviewText",
                            "attribute": "text",
                            "name": "review_count"
                        }
                    ]
                }),
                expected_outcome: ExpectedOutcome::DataExtracted {
                    fields: HashMap::from([
                        ("availability".to_string(), "In Stock".to_string()),
                    ])
                },
                validation_criteria: vec![
                    ValidationCriterion::PartialMatch {
                        field: "availability".to_string(),
                        contains: "Stock".to_string(),
                    },
                    ValidationCriterion::RegexMatch {
                        field: "review_count".to_string(),
                        pattern: r"\d+ ratings?".to_string(),
                    },
                ],
                timeout_ms: 5000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Read Reviews Section".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "scroll_to_element",
                            "arguments": {
                                "selector": "id:reviewsMedley"
                            }
                        },
                        {
                            "tool_name": "extract_elements_data",
                            "arguments": {
                                "elements": [
                                    {
                                        "selector": "selector:.review-rating:nth-child(1)",
                                        "attribute": "text",
                                        "name": "top_review_rating"
                                    },
                                    {
                                        "selector": "selector:.review-text-content:nth-child(1)",
                                        "attribute": "text",
                                        "name": "top_review_text"
                                    }
                                ]
                            }
                        }
                    ],
                    "delay_between_tools_ms": 500
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "reviews_extracted": true })
                },
                validation_criteria: vec![],
                timeout_ms: 5000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Add to Cart".to_string(),
                tool_name: "click_element".to_string(),
                arguments: object!({
                    "selector": "id:add-to-cart-button",
                    "wait_for_response": true
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "added_to_cart": true })
                },
                validation_criteria: vec![
                    ValidationCriterion::ElementExists {
                        selector: "selector:span:contains('Added to Cart')".to_string(),
                    },
                ],
                timeout_ms: 5000,
                retry_count: 2,
            },
        ],
        test_data: TestData {
            input_files: vec![],
            expected_outputs: HashMap::from([
                ("products_found".to_string(), object!(true)),
                ("cart_updated".to_string(), object!(true)),
            ]),
            mock_data: HashMap::new(),
        },
        accuracy_threshold: 75.0,
    }
}

/// Create an eBay bidding workflow
pub fn create_ebay_auction_workflow() -> ComplexWorkflow {
    ComplexWorkflow {
        name: "eBay Auction Monitoring and Bidding".to_string(),
        description: "Search for items, monitor auction, place strategic bid".to_string(),
        category: WorkflowCategory::WebNavigation,
        steps: vec![
            WorkflowStep {
                name: "Navigate to eBay".to_string(),
                tool_name: "navigate_browser".to_string(),
                arguments: object!({
                    "url": "https://www.ebay.com",
                    "wait_for_load": true
                }),
                expected_outcome: ExpectedOutcome::NavigationCompleted {
                    url: "https://www.ebay.com".to_string()
                },
                validation_criteria: vec![
                    ValidationCriterion::ElementExists {
                        selector: "id:gh-ac".to_string(),
                    },
                ],
                timeout_ms: 8000,
                retry_count: 2,
            },
            WorkflowStep {
                name: "Search for Collectible".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "type_into_element",
                            "arguments": {
                                "selector": "id:gh-ac",
                                "text_to_type": "vintage camera lens",
                                "clear_before_typing": true
                            }
                        },
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "id:gh-btn"
                            }
                        }
                    ],
                    "delay_between_tools_ms": 300
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "search_completed": true })
                },
                validation_criteria: vec![
                    ValidationCriterion::ElementExists {
                        selector: "selector:.srp-results".to_string(),
                    },
                ],
                timeout_ms: 8000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Filter to Auctions Only".to_string(),
                tool_name: "execute_sequence".to_string(),
                arguments: object!({
                    "tools": [
                        {
                            "tool_name": "click_element",
                            "arguments": {
                                "selector": "selector:input[aria-label='Auction']"
                            }
                        },
                        {
                            "tool_name": "wait_for_element",
                            "arguments": {
                                "selector": "selector:.s-item__time-left",
                                "timeout_ms": 3000
                            }
                        }
                    ],
                    "delay_between_tools_ms": 500
                }),
                expected_outcome: ExpectedOutcome::Success {
                    expected_data: object!({ "filter_applied": true })
                },
                validation_criteria: vec![
                    ValidationCriterion::ElementExists {
                        selector: "selector:.s-item__time-left".to_string(),
                    },
                ],
                timeout_ms: 5000,
                retry_count: 1,
            },
            WorkflowStep {
                name: "Extract Auction Details".to_string(),
                tool_name: "extract_elements_data".to_string(),
                arguments: object!({
                    "elements": [
                        {
                            "selector": "selector:.s-item:nth-child(1) .s-item__title",
                            "attribute": "text",
                            "name": "item_title"
                        },
                        {
                            "selector": "selector:.s-item:nth-child(1) .s-item__price",
                            "attribute": "text",
                            "name": "current_bid"
                        },
                        {
                            "selector": "selector:.s-item:nth-child(1) .s-item__time-left",
                            "attribute": "text",
                            "name": "time_left"
                        },
                        {
                            "selector": "selector:.s-item:nth-child(1) .s-item__bids",
                            "attribute": "text",
                            "name": "bid_count"
                        }
                    ]
                }),
                expected_outcome: ExpectedOutcome::DataExtracted {
                    fields: HashMap::from([
                        ("auction_found".to_string(), "true".to_string()),
                    ])
                },
                validation_criteria: vec![
                    ValidationCriterion::RegexMatch {
                        field: "current_bid".to_string(),
                        pattern: r"\$[\d,]+\.\d{2}".to_string(),
                    },
                    ValidationCriterion::PartialMatch {
                        field: "time_left".to_string(),
                        contains: "left".to_string(),
                    },
                ],
                timeout_ms: 5000,
                retry_count: 1,
            },
        ],
        test_data: TestData {
            input_files: vec![],
            expected_outputs: HashMap::from([
                ("auction_monitored".to_string(), object!(true)),
            ]),
            mock_data: HashMap::new(),
        },
        accuracy_threshold: 80.0,
    }
}