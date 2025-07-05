use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OutputParserDefinition {
    /// A JSONPath string to locate the UI tree within the tool's output JSON.
    /// e.g., "results[-1].result.content[?(@.ui_tree)].ui_tree"
    pub ui_tree_json_path: String,
    /// Defines how to identify a single item's container within the UI tree.
    pub item_container_definition: ItemContainerDefinition,
    /// Defines which fields to extract from each identified container.
    pub fields_to_extract: std::collections::HashMap<String, FieldExtractor>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemContainerDefinition {
    /// Conditions the container node itself must match.
    pub node_conditions: Vec<PropertyCondition>,
    /// Conditions that must be met by the container's direct children.
    pub child_conditions: LogicalCondition,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldExtractor {
    #[serde(default)]
    pub from_child: Option<FromChild>,
    #[serde(default)]
    pub from_children: Option<FromChildren>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FromChild {
    pub conditions: Vec<PropertyCondition>,
    pub extract_property: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FromChildren {
    pub conditions: Vec<PropertyCondition>,
    pub extract_property: String,
    /// If a value is provided, joins the results into a single string.
    pub join_with: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogicalCondition {
    pub logic: Logic, // "AND" or "OR"
    pub conditions: Vec<ChildCondition>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Logic {
    And,
    Or,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChildCondition {
    /// Asserts that at least one child matches the given property conditions.
    pub exists_child: Option<ExistsChild>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExistsChild {
    pub conditions: Vec<PropertyCondition>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PropertyCondition {
    pub property: String,
    #[serde(flatten)]
    pub operator: ConditionOperator,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "op", content = "value")]
pub enum ConditionOperator {
    Equals(String),
    StartsWith(String),
    Contains(String),
    IsOneOf(Vec<String>),
}

/// The main entry point for parsing tool output.
pub fn run_output_parser(parser_def_val: &Value, tool_output: &Value) -> Result<Option<Value>> {
    let parser_def: OutputParserDefinition = serde_json::from_value(parser_def_val.clone())?;

    let ui_tree = find_tree_in_json_result(tool_output, &parser_def.ui_tree_json_path)?;

    match ui_tree {
        Some(tree) => {
            let mut extracted_items = Vec::new();
            find_items_in_node(tree, &parser_def, &mut extracted_items);
            Ok(Some(json!(extracted_items)))
        }
        None => Ok(None),
    }
}

/// Finds the UI tree in a given JSON result using JSONPath.
fn find_tree_in_json_result<'a>(json_result: &'a Value, path: &str) -> Result<Option<&'a Value>> {
    let paths = jsonpath_lib::select(json_result, path)?;
    Ok(paths.first().copied())
}

/// Recursively traverses the UI tree to find nodes that match the container definition.
fn find_items_in_node(
    node: &Value,
    parser_def: &OutputParserDefinition,
    extracted_items: &mut Vec<Value>,
) {
    if check_node_as_container(node, &parser_def.item_container_definition) {
        if let Some(item) = extract_fields_from_container(node, &parser_def.fields_to_extract) {
            extracted_items.push(item);
        }
    }

    if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
        for child in children {
            find_items_in_node(child, parser_def, extracted_items);
        }
    }
}

/// Checks if a given node meets all the criteria to be an item container.
fn check_node_as_container(node: &Value, container_def: &ItemContainerDefinition) -> bool {
    // 1. Check if the node itself matches the conditions
    for condition in &container_def.node_conditions {
        if !check_property_condition(node, condition) {
            return false;
        }
    }

    // 2. Check if the node's children meet the logical conditions
    let children = match node.get("children").and_then(|c| c.as_array()) {
        Some(c) => c,
        None => return false, // No children, so can't meet child conditions
    };

    let logic_ok = match container_def.child_conditions.logic {
        Logic::And => container_def
            .child_conditions
            .conditions
            .iter()
            .all(|cond| check_child_condition(children, cond)),
        Logic::Or => container_def
            .child_conditions
            .conditions
            .iter()
            .any(|cond| check_child_condition(children, cond)),
    };

    if !logic_ok {
        return false;
    }

    true
}

/// Extracts the defined fields from a container node.
fn extract_fields_from_container(
    container: &Value,
    fields_to_extract: &std::collections::HashMap<String, FieldExtractor>,
) -> Option<Value> {
    let mut extracted_object = serde_json::Map::new();
    let children = container.get("children")?.as_array()?;

    for (field_name, extractor) in fields_to_extract {
        if let Some(from_child) = &extractor.from_child {
            for child in children {
                if from_child
                    .conditions
                    .iter()
                    .all(|cond| check_property_condition(child, cond))
                {
                    if let Some(value) = child.get(&from_child.extract_property).cloned() {
                        extracted_object.insert(field_name.clone(), value);
                        break;
                    }
                }
            }
        } else if let Some(from_children) = &extractor.from_children {
            let mut found_values: Vec<Value> = Vec::new();
            for child in children {
                if from_children
                    .conditions
                    .iter()
                    .all(|cond| check_property_condition(child, cond))
                {
                    if let Some(value) = child.get(&from_children.extract_property).cloned() {
                        found_values.push(value);
                    }
                }
            }
            if !found_values.is_empty() {
                if let Some(separator) = &from_children.join_with {
                    let joined = found_values
                        .iter()
                        .map(|v| v.as_str().unwrap_or(""))
                        .collect::<Vec<_>>()
                        .join(separator);
                    extracted_object.insert(field_name.clone(), json!(joined));
                } else {
                    extracted_object.insert(field_name.clone(), json!(found_values));
                }
            }
        }
    }

    if extracted_object.is_empty() {
        None
    } else {
        Some(Value::Object(extracted_object))
    }
}

/// Checks if a set of children nodes satisfies a given condition.
fn check_child_condition(children: &[Value], condition: &ChildCondition) -> bool {
    if let Some(exists_child) = &condition.exists_child {
        return children.iter().any(|child| {
            exists_child
                .conditions
                .iter()
                .all(|cond| check_property_condition(child, cond))
        });
    }
    false
}

/// Checks if a node's property matches a specific condition.
fn check_property_condition(node: &Value, condition: &PropertyCondition) -> bool {
    let prop_value = match node.get(&condition.property).and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return false,
    };

    match &condition.operator {
        ConditionOperator::Equals(val) => prop_value == val,
        ConditionOperator::StartsWith(val) => prop_value.starts_with(val),
        ConditionOperator::Contains(val) => prop_value.contains(val),
        ConditionOperator::IsOneOf(vals) => vals.iter().any(|v| v == prop_value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn get_test_ui_tree() -> Value {
        json!({
            "name": "Root",
            "role": "Document",
            "children": [
                {
                    "name": "Some Header",
                    "role": "Header"
                },
                {
                    "name": "Quote List Container",
                    "role": "Group",
                    "children": [
                        // Valid Quote 1
                        {
                            "name": "Quote 1 Container",
                            "role": "Group",
                            "children": [
                                {"role": "Image", "name": "prosperity primeterm-to-100 logo"},
                                {"role": "Text", "name": "Prosperity PrimeTerm to 100: 20 YEAR TERM*"},
                                {"role": "Text", "name": "$358.56"},
                                {"role": "Text", "name": "Monthly Price"},
                                {"role": "Text", "name": "Graded"},
                                {"role": "Text", "name": "Discontinued"},
                            ]
                        },
                        // Valid Quote 2
                        {
                            "name": "Quote 2 Container",
                            "role": "Group",
                            "children": [
                                {"role": "Image", "name": "acme-insurance-logo"},
                                {"role": "Text", "name": "ACME Insurance: Term Life"},
                                {"role": "Text", "name": "$120.00"},
                                {"role": "Text", "name": "Monthly Price"},
                                {"role": "Text", "name": "Standard"},
                            ]
                        },
                        // Invalid item (no price)
                        {
                            "name": "Invalid Container",
                            "role": "Group",
                            "children": [
                                {"role": "Text", "name": "Some other info"},
                                {"role": "Text", "name": "Monthly Price"},
                            ]
                        }
                    ]
                }
            ]
        })
    }

    fn get_quote_parser_definition() -> OutputParserDefinition {
        serde_json::from_value(json!({
            "uiTreeJsonPath": "$",
            "itemContainerDefinition": {
                "nodeConditions": [{"property": "role", "op": "equals", "value": "Group"}],
                "childConditions": {
                    "logic": "and",
                    "conditions": [
                        {"existsChild": {"conditions": [{"property": "name", "op": "startsWith", "value": "$"}]}},
                        {"existsChild": {"conditions": [{"property": "name", "op": "equals", "value": "Monthly Price"}]}}
                    ]
                }
            },
            "fieldsToExtract": {
                "carrierProduct": {
                    "fromChild": {
                        "conditions": [{"property": "name", "op": "contains", "value": ":"}],
                        "extractProperty": "name"
                    }
                },
                "monthlyPrice": {
                    "fromChild": {
                        "conditions": [{"property": "name", "op": "startsWith", "value": "$"}],
                        "extractProperty": "name"
                    }
                },
                "status": {
                    "fromChildren": {
                        "conditions": [
                            {"property": "role", "op": "equals", "value": "Text"},
                            {"property": "name", "op": "isOneOf", "value": ["Graded", "Discontinued", "Standard"]}
                        ],
                        "extractProperty": "name",
                        "joinWith": ", "
                    }
                }
            }
        }))
        .unwrap()
    }

    #[test]
    fn test_extract_quotes() {
        let ui_tree = get_test_ui_tree();
        let parser_def = get_quote_parser_definition();

        let mut extracted_items = Vec::new();
        find_items_in_node(&ui_tree, &parser_def, &mut extracted_items);

        assert_eq!(extracted_items.len(), 2);

        // Check first quote
        let quote1 = &extracted_items[0];
        assert_eq!(
            quote1["carrierProduct"],
            "Prosperity PrimeTerm to 100: 20 YEAR TERM*"
        );
        assert_eq!(quote1["monthlyPrice"], "$358.56");
        assert_eq!(quote1["status"], "Graded, Discontinued");

        // Check second quote
        let quote2 = &extracted_items[1];
        assert_eq!(quote2["carrierProduct"], "ACME Insurance: Term Life");
        assert_eq!(quote2["monthlyPrice"], "$120.00");
        assert_eq!(quote2["status"], "Standard");
    }

    #[test]
    fn test_run_parser_with_json_path() {
        let tool_output = json!({
            "results": [
                {},
                {
                    "result": {
                        "content": [
                            {
                                "ui_tree": get_test_ui_tree()
                            }
                        ]
                    }
                }
            ]
        });

        let parser_def_json = serde_json::to_value(get_quote_parser_definition()).unwrap();
        // Modify the path for this specific test case
        let mut parser_def_json_obj = parser_def_json.as_object().unwrap().clone();
        parser_def_json_obj.insert(
            "uiTreeJsonPath".to_string(),
            json!("$.results[-1].result.content[0].ui_tree"),
        );
        let parser_def_json = Value::Object(parser_def_json_obj);

        let result = run_output_parser(&parser_def_json, &tool_output)
            .unwrap()
            .unwrap();
        let items = result.as_array().unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[1]["monthlyPrice"], "$120.00");
    }
}
