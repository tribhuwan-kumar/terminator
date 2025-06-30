# MCP Workflow Accuracy Testing Framework

## Overview

This framework provides a comprehensive system for measuring the accuracy of the MCP (Model Context Protocol) server in executing complex, real-world workflows. It's designed to simulate and test workflows that AI agents typically perform, such as:

- PDF data extraction and form filling
- Multi-step insurance quote generation
- Cross-application data collection and compilation
- Complex web navigation and form submission

## Architecture

### Core Components

1. **WorkflowAccuracyTester** (`workflow_accuracy_tests.rs`)
   - Main test runner that connects to the MCP server
   - Executes workflows and measures accuracy
   - Generates comprehensive reports

2. **Workflow Definitions** (`workflow_definitions.rs`)
   - Pre-defined complex workflows that simulate real-world tasks
   - Each workflow contains multiple steps with validation criteria
   - Covers different categories: DataEntry, FormFilling, DocumentProcessing, etc.

3. **Mock Implementation** (`mock_workflow_runner.rs`)
   - Allows testing the framework without real UI interactions
   - Simulates realistic success rates and response times
   - Useful for development and CI/CD pipelines

4. **Test Runners** (`accuracy_test_runner.rs`, `mock_accuracy_test.rs`)
   - Individual test cases for different workflow scenarios
   - Generates JSON and Markdown reports

## Key Concepts

### Workflow Structure

Each workflow consists of:
- **Name & Description**: Clear identification of the workflow
- **Category**: Type of workflow (DataEntry, FormFilling, etc.)
- **Steps**: Sequential actions to perform
- **Test Data**: Input files and expected outputs
- **Accuracy Threshold**: Minimum acceptable accuracy percentage

### Workflow Steps

Each step includes:
- **Tool Name**: MCP tool to execute (e.g., `click_element`, `type_into_element`)
- **Arguments**: Parameters for the tool
- **Expected Outcome**: What should happen if successful
- **Validation Criteria**: How to verify success
- **Timeout & Retry**: Resilience parameters

### Validation Criteria

Multiple validation types are supported:
- **ExactMatch**: Field value must exactly match expected
- **PartialMatch**: Field value must contain expected substring
- **RegexMatch**: Field value must match regex pattern
- **NumericRange**: Numeric value must be within range
- **ElementExists**: UI element must be present
- **ElementHasText**: UI element must contain specific text
- **ResponseTime**: Operation must complete within time limit

## Example Workflows

### 1. PDF Invoice to Accounting Form
Simulates extracting data from a PDF invoice and entering it into accounting software:
- Opens PDF reader
- Loads invoice file
- Extracts key fields (invoice number, date, amount, vendor)
- Opens accounting software
- Navigates to bill entry
- Fills form with extracted data
- Saves the entry

### 2. Insurance Quote Generation
Simulates getting an insurance quote from a website:
- Opens browser
- Navigates to insurance site
- Fills personal information
- Fills health information
- Selects coverage options
- Submits for quote
- Extracts quote details

### 3. Research Data Collection
Simulates collecting data from multiple sources:
- Opens spreadsheet application
- Creates headers
- Opens browser
- Searches for company data
- Extracts stock information
- Switches back to spreadsheet
- Enters collected data
- Saves spreadsheet

## Running Tests

### With Real MCP Server
```bash
# Build the MCP agent first
cargo build --bin terminator-mcp-agent

# Run specific workflow test
cargo test test_pdf_data_entry_accuracy -- --nocapture

# Run all workflow tests
cargo test test_all_workflows_accuracy -- --nocapture
```

### With Mock Implementation
```bash
# Run mock tests (no UI required)
cargo test test_mock_workflow_accuracy -- --nocapture
```

## Accuracy Metrics

The framework measures:
- **Overall Accuracy**: Percentage of successful steps across all workflows
- **Per-Workflow Accuracy**: Success rate for individual workflows
- **Step-Level Success**: Detailed success/failure for each step
- **Execution Time**: Performance metrics at workflow and step level
- **Validation Results**: Specific validation criteria pass/fail

## Report Generation

Reports are generated in multiple formats:

### JSON Report
- Complete structured data
- Located in `target/accuracy_reports/`
- Includes all metrics and detailed results

### Markdown Report
- Human-readable summary
- Tables showing workflow and step results
- Error summaries and validation details

## Extending the Framework

### Adding New Workflows

1. Create a new function in `workflow_definitions.rs`:
```rust
pub fn create_my_workflow() -> ComplexWorkflow {
    ComplexWorkflow {
        name: "My Workflow".to_string(),
        // ... define steps, validation, etc.
    }
}
```

2. Add test case in `accuracy_test_runner.rs`:
```rust
#[tokio::test]
async fn test_my_workflow_accuracy() -> Result<()> {
    let mut tester = WorkflowAccuracyTester::new().await?;
    tester.add_workflow(create_my_workflow());
    // ... run and validate
}
```

### Adding New Validation Types

1. Add variant to `ValidationCriterion` enum
2. Implement validation logic in `validate_step_outcome`
3. Update mock implementation if needed

## Best Practices

1. **Realistic Workflows**: Design workflows that mirror actual user tasks
2. **Comprehensive Validation**: Use multiple validation criteria per step
3. **Error Handling**: Include retry logic and handle expected failures
4. **Performance Targets**: Set realistic timeout values
5. **Mock Testing**: Use mocks for rapid iteration during development

## Future Enhancements

- [ ] Support for parallel workflow execution
- [ ] Integration with AI agents for adaptive testing
- [ ] Visual workflow designer
- [ ] Real-time accuracy monitoring dashboard
- [ ] Workflow recording and playback
- [ ] Cross-platform workflow compatibility testing
- [ ] Machine learning-based accuracy prediction
- [ ] Automated workflow generation from user recordings

## Troubleshooting

### Common Issues

1. **MCP Agent Not Found**
   - Ensure `cargo build --bin terminator-mcp-agent` has been run
   - Check binary path in `get_agent_binary_path()`

2. **Low Accuracy Scores**
   - Review validation criteria - may be too strict
   - Check element selectors match actual UI
   - Increase timeout values for slow operations

3. **Test Timeouts**
   - Adjust workflow step timeouts
   - Check MCP server responsiveness
   - Use mock tests for faster feedback

## Contributing

When adding new workflows or improving the framework:
1. Ensure workflows represent real use cases
2. Add comprehensive validation criteria
3. Test with both real MCP and mock implementations
4. Update documentation with new features
5. Include performance benchmarks