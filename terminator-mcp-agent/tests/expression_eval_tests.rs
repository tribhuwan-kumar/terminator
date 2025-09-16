use serde_json::json;
use terminator_mcp_agent::expression_eval::evaluate;

#[test]
fn test_evaluate_binary_expressions() {
    let vars = json!({
        "use_max_budget": false,
        "coverage_type": "Graded"
    });

    assert!(evaluate("use_max_budget == false", &vars));
    assert!(!evaluate("use_max_budget == true", &vars));
    assert!(evaluate("coverage_type == 'Graded'", &vars));
    assert!(evaluate("coverage_type != 'Standard'", &vars));
}

#[test]
fn test_evaluate_contains() {
    let vars = json!({
        "product_types": ["FEX", "Term"],
        "description": "Final Expense"
    });

    assert!(evaluate("contains(product_types, 'FEX')", &vars));
    assert!(!evaluate("contains(product_types, 'MedSup')", &vars));
    assert!(evaluate("contains(description, 'Expense')", &vars));
}

#[test]
fn test_evaluate_starts_with() {
    let vars = json!({ "name": "John Doe" });
    assert!(evaluate("startsWith(name, 'John')", &vars));
    assert!(!evaluate("startsWith(name, 'Doe')", &vars));
}

#[test]
fn test_evaluate_ends_with() {
    let vars = json!({ "name": "John Doe" });
    assert!(evaluate("endsWith(name, 'Doe')", &vars));
    assert!(!evaluate("endsWith(name, 'John')", &vars));
}

#[test]
fn test_string_with_spaces() {
    let vars = json!({
        "quote_type": "Face Amount"
    });

    assert!(evaluate("quote_type == 'Face Amount'", &vars));
    assert!(!evaluate("quote_type == 'Monthly Amount'", &vars));
}

#[test]
fn test_invalid_expressions() {
    let vars = json!({});
    assert!(!evaluate("invalid expression", &vars)); // Invalid format
    assert!(!evaluate("unsupported(a, b)", &vars)); // Unsupported function
    assert!(!evaluate("var.not.found == true", &vars)); // Variable not found
}

#[test]
fn test_negation_contains() {
    let vars = json!({
        "product_types": ["FEX", "Term"],
        "description": "Final Expense"
    });

    assert!(!evaluate("!contains(product_types, 'FEX')", &vars));
    assert!(evaluate("!contains(product_types, 'MedSup')", &vars));
    assert!(!evaluate("!contains(description, 'Expense')", &vars));
    assert!(evaluate("!contains(description, 'Medical')", &vars));
}

#[test]
fn test_negation_starts_with() {
    let vars = json!({ "name": "John Doe" });
    assert!(!evaluate("!startsWith(name, 'John')", &vars));
    assert!(evaluate("!startsWith(name, 'Doe')", &vars));
}

#[test]
fn test_negation_ends_with() {
    let vars = json!({ "name": "John Doe" });
    assert!(!evaluate("!endsWith(name, 'Doe')", &vars));
    assert!(evaluate("!endsWith(name, 'John')", &vars));
}

#[test]
fn test_negation_binary_expressions() {
    let vars = json!({
        "use_max_budget": false,
        "coverage_type": "Graded",
        "enabled": true
    });

    assert!(!evaluate("!use_max_budget == false", &vars));
    assert!(evaluate("!use_max_budget == true", &vars));
    assert!(!evaluate("!coverage_type == 'Graded'", &vars));
    assert!(evaluate("!coverage_type == 'Standard'", &vars));
    assert!(evaluate("!coverage_type != 'Graded'", &vars));
    assert!(!evaluate("!coverage_type != 'Standard'", &vars));
    assert!(!evaluate("!enabled == true", &vars));
    assert!(evaluate("!enabled == false", &vars));
}

#[test]
fn test_negation_with_whitespace() {
    let vars = json!({
        "product_types": ["FEX", "Term"]
    });

    assert!(evaluate("! contains(product_types, 'MedSup')", &vars));
    assert!(evaluate("!  contains(product_types, 'MedSup')", &vars));
    assert!(evaluate("  !contains(product_types, 'MedSup')", &vars));
    assert!(evaluate("  ! contains(product_types, 'MedSup')", &vars));
    assert!(evaluate("  !  contains(product_types, 'MedSup')  ", &vars));
}

#[test]
fn test_double_negation() {
    let vars = json!({
        "product_types": ["FEX", "Term"]
    });

    assert!(evaluate("!!contains(product_types, 'FEX')", &vars));
    assert!(!evaluate("!!contains(product_types, 'MedSup')", &vars));
}

#[test]
fn test_triple_negation() {
    let vars = json!({
        "product_types": ["FEX", "Term"]
    });

    assert!(!evaluate("!!!contains(product_types, 'FEX')", &vars));
    assert!(evaluate("!!!contains(product_types, 'MedSup')", &vars));
}

#[test]
fn test_negation_with_missing_variables() {
    let vars = json!({});
    assert!(evaluate("!contains(missing_var, 'value')", &vars));
    assert!(evaluate("!missing_var == 'value'", &vars));
    assert!(evaluate("!startsWith(missing_var, 'test')", &vars));
}

#[test]
fn test_negation_edge_cases() {
    let vars = json!({
        "empty_array": [],
        "empty_string": "",
        "null_value": null
    });

    assert!(evaluate("!contains(empty_array, 'anything')", &vars));
    assert!(evaluate("!contains(empty_string, 'anything')", &vars));
    assert!(evaluate("!startsWith(empty_string, 'test')", &vars));
    assert!(evaluate("!endsWith(empty_string, 'test')", &vars));
    assert!(evaluate("!null_value == 'test'", &vars));
}

#[test]
fn test_complex_negation_scenarios() {
    let vars = json!({
        "product_types": ["FEX", "Term", "MedSup"],
        "quote_type": "Face Amount",
        "enabled": true,
        "user_name": "John Smith"
    });

    assert!(!evaluate("!contains(product_types, 'FEX')", &vars));
    assert!(evaluate("!contains(product_types, 'Preneed')", &vars));
    assert!(!evaluate("!quote_type == 'Face Amount'", &vars));
    assert!(evaluate("!quote_type == 'Monthly Amount'", &vars));
    assert!(evaluate("!startsWith(user_name, 'Jane')", &vars));
    assert!(!evaluate("!endsWith(user_name, 'Smith')", &vars));
}

#[test]
fn test_negation_preserves_original_functionality() {
    let vars = json!({
        "product_types": ["FEX", "Term"],
        "quote_type": "Face Amount",
        "enabled": true
    });

    assert!(evaluate("contains(product_types, 'FEX')", &vars));
    assert!(!evaluate("contains(product_types, 'MedSup')", &vars));
    assert!(evaluate("quote_type == 'Face Amount'", &vars));
    assert!(!evaluate("quote_type == 'Monthly Amount'", &vars));
    assert!(evaluate("enabled == true", &vars));
    assert!(!evaluate("enabled == false", &vars));

    assert!(!evaluate("!contains(product_types, 'FEX')", &vars));
    assert!(evaluate("!contains(product_types, 'MedSup')", &vars));
    assert!(!evaluate("!quote_type == 'Face Amount'", &vars));
    assert!(evaluate("!quote_type == 'Monthly Amount'", &vars));
    assert!(!evaluate("!enabled == true", &vars));
    assert!(evaluate("!enabled == false", &vars));
}

#[test]
fn test_always_function() {
    let vars = json!({
        "some_var": "some_value"
    });

    assert!(evaluate("always()", &vars));

    let empty_vars = json!({});
    assert!(evaluate("always()", &empty_vars));

    let null_vars = json!(null);
    assert!(evaluate("always()", &null_vars));
}

#[test]
fn test_always_function_with_whitespace() {
    let vars = json!({});

    assert!(evaluate("always()", &vars));
    assert!(evaluate("always( )", &vars));
    assert!(evaluate("always(  )", &vars));
    assert!(evaluate(" always() ", &vars));
    assert!(evaluate("  always()  ", &vars));
}

#[test]
fn test_always_function_with_arguments_should_fail() {
    let vars = json!({});

    assert!(!evaluate("always(arg)", &vars));
    assert!(!evaluate("always('test')", &vars));
    assert!(!evaluate("always(var1, var2)", &vars));
}

#[test]
fn test_negation_of_always() {
    let vars = json!({});

    assert!(!evaluate("!always()", &vars));
    assert!(!evaluate("! always()", &vars));
    assert!(!evaluate("  !always()  ", &vars));
    assert!(evaluate("!!always()", &vars));
}

#[test]
fn test_simple_boolean_variables() {
    let vars = json!({
        "enabled": true,
        "disabled": false,
        "env": {
            "troubleshooting": false,
            "needs_login": "true"
        }
    });

    assert!(evaluate("enabled", &vars));
    assert!(!evaluate("disabled", &vars));
    assert!(!evaluate("!enabled", &vars));
    assert!(evaluate("!disabled", &vars));
    assert!(!evaluate("env.troubleshooting", &vars));
    assert!(evaluate("!env.troubleshooting", &vars));
    assert!(evaluate("env.needs_login == 'true' && !env.troubleshooting", &vars));
    assert!(!evaluate("env.needs_login == 'true' && env.troubleshooting", &vars));
}

#[test]
fn test_truthiness_of_different_types() {
    let vars = json!({
        "empty_string": "",
        "non_empty_string": "hello",
        "false_string": "false",
        "zero_string": "0",
        "zero_number": 0,
        "positive_number": 42,
        "negative_number": -1,
        "empty_array": [],
        "non_empty_array": [1, 2, 3],
        "empty_object": {},
        "non_empty_object": {"key": "value"},
        "null_value": null
    });

    assert!(!evaluate("empty_string", &vars));
    assert!(evaluate("non_empty_string", &vars));
    assert!(!evaluate("false_string", &vars));
    assert!(!evaluate("zero_string", &vars));
    assert!(!evaluate("zero_number", &vars));
    assert!(evaluate("positive_number", &vars));
    assert!(evaluate("negative_number", &vars));
    assert!(!evaluate("empty_array", &vars));
    assert!(evaluate("non_empty_array", &vars));
    assert!(!evaluate("empty_object", &vars));
    assert!(evaluate("non_empty_object", &vars));
    assert!(!evaluate("null_value", &vars));
}