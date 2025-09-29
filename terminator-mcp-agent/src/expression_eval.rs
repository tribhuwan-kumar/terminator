use serde_json::Value;
use tracing::warn;

/// Normalizes an expression by replacing smart quotes and other Unicode characters
/// with their ASCII equivalents to handle copy-paste from various sources.
fn normalize_expression(expr: &str) -> String {
    expr
        // Normalize smart quotes to straight quotes
        .replace(['\u{2018}', '\u{2019}'], "'") // Smart single quotes
        .replace(['\u{201C}', '\u{201D}'], "\"") // Smart double quotes
        .replace('`', "'") // Backticks to single quotes
        // Normalize Unicode spaces
        .replace(['\u{00A0}', '\u{2009}', '\u{202F}'], " ") // Various Unicode spaces
        .trim()
        .to_string()
}

// Helper to get a value from the variables JSON.
pub fn get_value<'a>(path: &str, variables: &'a Value) -> Option<&'a Value> {
    // Support dot notation for nested access
    if !path.contains('.') {
        return variables.get(path); // Fast path for simple keys
    }

    let mut current = variables;
    for part in path.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

// Main evaluation function.
pub fn evaluate(expression: &str, variables: &Value) -> bool {
    // Normalize the expression to handle smart quotes and other Unicode characters
    let normalized = normalize_expression(expression);
    evaluate_internal(&normalized, variables)
}

// Internal evaluation function that works with normalized expressions
fn evaluate_internal(expression: &str, variables: &Value) -> bool {
    // Trim whitespace
    let expr = expression.trim();

    // Handle negation operator
    if let Some(inner_expr) = expr.strip_prefix('!') {
        let inner_expr = inner_expr.trim();
        return !evaluate_internal(inner_expr, variables);
    }

    // Handle logical operators (&&, ||) with proper precedence
    if let Some(pos) = expr.find("&&") {
        let left = &expr[..pos].trim();
        let right = &expr[pos + 2..].trim();
        return evaluate_internal(left, variables) && evaluate_internal(right, variables);
    }

    if let Some(pos) = expr.find("||") {
        let left = &expr[..pos].trim();
        let right = &expr[pos + 2..].trim();
        return evaluate_internal(left, variables) || evaluate_internal(right, variables);
    }

    // Try parsing function-based expressions first, e.g., contains(vars, 'value')
    if let Some(result) = parse_and_evaluate_function(expr, variables) {
        return result;
    }

    // Fallback to simple binary expressions, e.g., vars == 'value'
    if let Some(result) = parse_and_evaluate_binary_expression(expr, variables) {
        return result;
    }

    // Handle literal boolean values
    if expr == "true" {
        return true;
    }
    if expr == "false" {
        return false;
    }

    // Handle simple variable references (evaluate to their boolean truthiness)
    // This allows expressions like "env.troubleshooting" or "!env.troubleshooting"
    // where troubleshooting is a boolean
    if let Some(value) = get_value(expr, variables) {
        return match value {
            Value::Bool(b) => *b,
            Value::String(s) => !s.is_empty() && s != "false" && s != "0",
            Value::Number(n) => n.as_i64().unwrap_or(0) != 0,
            Value::Null => false,
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
        };
    }

    warn!(
        "Could not parse expression: '{}'. Defaulting to false.",
        expression
    );
    false
}

// Parses expressions like "contains(policy.product_types, 'FEX')"
fn parse_and_evaluate_function(expr: &str, variables: &Value) -> Option<bool> {
    let (func_name, args_str) = expr.split_once('(')?;
    if !args_str.ends_with(')') {
        return None;
    }
    let args_str = &args_str[..args_str.len() - 1]; // Remove trailing ')'

    match func_name.trim() {
        "always" => {
            // always() function takes no arguments and always returns true
            if args_str.trim().is_empty() {
                Some(true)
            } else {
                None // always() should not have arguments
            }
        }
        _ => {
            // For other functions, we need exactly 2 arguments
            let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
            if args.len() != 2 {
                return None;
            }

            let val1 = get_value(args[0], variables)?;
            let val2_str = args[1].trim_matches('\''); // Remove single quotes

            match func_name.trim() {
                "contains" => Some(evaluate_contains(val1, val2_str)),
                "startsWith" => Some(val1.as_str()?.starts_with(val2_str)),
                "endsWith" => Some(val1.as_str()?.ends_with(val2_str)),
                _ => None,
            }
        }
    }
}

// Evaluates the 'contains' function for different types.
fn evaluate_contains(collection: &Value, item: &str) -> bool {
    match collection {
        Value::Array(arr) => arr.iter().any(|v| v.as_str() == Some(item)),
        Value::String(s) => s.contains(item),
        _ => false,
    }
}

// Parses simple expressions like "variable == 'value'" or "variable == true"
fn parse_and_evaluate_binary_expression(expr: &str, variables: &Value) -> Option<bool> {
    let (var_path, op, raw_rhs) = if let Some(pos) = expr.find("==") {
        (&expr[..pos], "==", &expr[pos + 2..])
    } else if let Some(pos) = expr.find("!=") {
        (&expr[..pos], "!=", &expr[pos + 2..])
    } else {
        return None;
    };

    let var_path = var_path.trim();
    let raw_rhs = raw_rhs.trim();

    let lhs = get_value(var_path, variables)?;

    let are_equal = match raw_rhs {
        "true" => lhs.as_bool() == Some(true),
        "false" => lhs.as_bool() == Some(false),
        _ if raw_rhs.starts_with('\'') && raw_rhs.ends_with('\'') => {
            let rhs_str = raw_rhs.trim_matches('\'');
            compare_values_smart(lhs, rhs_str)
        }
        _ if raw_rhs.starts_with('"') && raw_rhs.ends_with('"') => {
            let rhs_str = raw_rhs.trim_matches('"');
            compare_values_smart(lhs, rhs_str)
        }
        _ => return None, // Invalid RHS
    };

    match op {
        "==" => Some(are_equal),
        "!=" => Some(!are_equal),
        _ => None, // Should be unreachable
    }
}

// Smart comparison that handles type coercion between strings and booleans
fn compare_values_smart(lhs: &Value, rhs_str: &str) -> bool {
    match lhs {
        Value::String(s) => s == rhs_str,
        Value::Bool(true) => rhs_str == "true" || rhs_str == "1",
        Value::Bool(false) => rhs_str == "false" || rhs_str == "0",
        Value::Number(n) => rhs_str.parse::<f64>().ok() == Some(n.as_f64().unwrap_or(0.0)),
        _ => false,
    }
}
