# Simplified MCP Accuracy Testing Framework

## What I Actually Built

A clean, focused accuracy testing framework that:
1. Tests real MCP tools with actual Terminator SDK selectors
2. Measures success rates of workflows
3. Validates outcomes without noise

## Core Components

### 1. `workflow_accuracy_tests.rs`
- Main test framework
- Connects to MCP server
- Executes workflows step by step
- Measures accuracy and timing
- Handles retries and validation

### 2. `simple_accuracy_test.rs`
Two realistic test workflows:

#### Calculator Test
- Opens Windows Calculator
- Performs simple math (5 + 3 = 8)
- Uses real selectors: `button|Five`, `Name:Plus`, `nativeid:CalculatorResults`
- Validates the result

#### Notepad Test
- Opens Notepad
- Types text
- Saves file with Ctrl+S
- Uses real selectors: `document|Text Editor`, `edit|File name:`

### 3. `clean_accuracy_runner.rs`
- Simple test runner
- Individual tests for each workflow
- Combined test for overall accuracy
- JSON report generation

## How It Works

1. **Start MCP Server**: Spawns the terminator-mcp-agent process
2. **Execute Workflows**: Runs each step through MCP tools
3. **Measure Success**: Tracks which steps succeed/fail
4. **Calculate Accuracy**: Percentage of successful steps
5. **Generate Reports**: JSON files with detailed results

## Real Selectors Used

Based on actual Terminator SDK patterns:
- `role|name` format: `button|Save`, `document|Text Editor`
- Name selectors: `Name:Five`, `Name:Plus`
- Native IDs: `nativeid:CalculatorResults`
- ID fallbacks: `#12345` when name is empty

## Running Tests

```bash
# Test calculator workflow
cargo test test_calculator_accuracy -- --nocapture

# Test notepad workflow  
cargo test test_notepad_accuracy -- --nocapture

# Test both
cargo test test_basic_workflows -- --nocapture
```

## What Makes This Better

1. **Real Selectors**: Uses actual selector formats from the SDK
2. **Simple Workflows**: Calculator and Notepad - apps that actually exist
3. **Measurable Results**: Can verify 5+3=8, file gets saved
4. **No Fantasy**: No made-up websites or imaginary selectors
5. **Focused**: Just measures if MCP tools work correctly

## Next Steps

1. Add more basic app tests (Paint, File Explorer)
2. Test error cases (element not found, timeout)
3. Measure performance (response times)
4. Test cross-platform (Linux/macOS equivalents)

The framework is now lean, realistic, and actually useful for measuring MCP accuracy.