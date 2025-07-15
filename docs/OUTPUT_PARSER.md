
# UI Tree Output Parser Documentation

This document describes how to use the `output_parser` module to extract structured data from a UI accessibility tree. The parser allows you to define rules for identifying "item containers" within the tree and then extract specific fields from those containers or their children.

## Core Concepts

The output parser operates on a `serde_json::Value` representing a UI tree, typically obtained from tools like `get_focused_window_tree` or `get_window_tree`. It uses a JSON-based definition to specify:

1.  **Item Container Definition**: How to identify the root node of each "item" you want to extract (e.g., a row in a table, a product card).
2.  **Field Extractors**: How to pull specific pieces of data (fields) from within each identified item container.

## Main Structures

### `OutputParserDefinition`

The top-level structure that defines the entire parsing logic.

```json
{
  "itemContainerDefinition": { /* ... */ },
  "fieldsToExtract": { /* ... */ }
}
```

-   `itemContainerDefinition`: An `ItemContainerDefinition` object that specifies how to find individual data items within the UI tree.
-   `fieldsToExtract`: A map where keys are the desired field names (e.g., `"productName"`, `"price"`) and values are `FieldExtractor` objects defining how to get that field's value.

### `ItemContainerDefinition`

Defines the criteria for a UI tree node to be considered an "item container".

```json
{
  "nodeConditions": [ /* ... */ ],
  "childConditions": { /* ... */ }
}
```

-   `nodeConditions`: An array of `PropertyCondition` objects. ALL of these conditions must be met by the container node itself.
-   `childConditions`: A `LogicalCondition` object. This defines conditions that must be met by the direct children of the container node. This is useful for identifying containers based on the presence or properties of their children.

### `LogicalCondition`

Combines multiple `ChildCondition`s using "AND" or "OR" logic.

```json
{
  "logic": "and" | "or",
  "conditions": [ /* ... */ ]
}
```

-   `logic`: Either `"and"` (all conditions must be true) or `"or"` (at least one condition must be true).
-   `conditions`: An array of `ChildCondition` objects.

### `ChildCondition`

Defines a condition that applies to the children of an item container.

```json
{
  "existsChild": { /* ... */ }
}
```

-   `existsChild`: An `ExistsChild` object. This condition is met if at least one child of the container matches the specified `PropertyCondition`s.

### `ExistsChild`

Specifies conditions for a child node to exist.

```json
{
  "conditions": [ /* ... */ ]
}
```

-   `conditions`: An array of `PropertyCondition` objects. ALL of these must be met by at least one child.

### `PropertyCondition`

Defines a single condition to check against a UI element's property.

```json
{
  "property": "name" | "role" | "value",
  "op": "equals" | "startsWith" | "contains" | "isOneOf",
  "value": "string" | ["string", "string"]
}
```

-   `property`: The name of the property to check (e.g., `"name"`, `"role"`, `"value"`). Properties can be top-level on the UI element or nested under an `"attributes"` object.
-   `op`: The operator to use for comparison:
    -   `"equals"`: Property value must exactly match `value`.
    -   `"startsWith"`: Property value must start with `value`.
    -   `"contains"`: Property value must contain `value`.
    -   `"isOneOf"`: Property value must be one of the strings in the `value` array.
-   `value`: The value(s) to compare against. For `"isOneOf"`, this should be an array of strings.

### `FieldExtractor`

Defines how to extract a single field's value from an item container.

```json
{
  "fromChild": { /* ... */ } |
  "fromChildren": { /* ... */ }
}
```

-   `fromChild`: An `FromChild` object. Extracts a single value from the first child that matches the conditions.
-   `fromChildren`: An `FromChildren` object. Extracts multiple values from all children that match the conditions.

### `FromChild`

Extracts a single property from a matching child.

```json
{
  "conditions": [ /* ... */ ],
  "extractProperty": "name"
}
```

-   `conditions`: An array of `PropertyCondition` objects. The first child matching ALL of these conditions will be used.
-   `extractProperty`: The name of the property to extract from the matched child.

### `FromChildren`

Extracts properties from multiple matching children, optionally joining them.

```json
{
  "conditions": [ /* ... */ ],
  "extractProperty": "name",
  "joinWith": ", "
}
```

-   `conditions`: An array of `PropertyCondition` objects. ALL children matching ALL of these conditions will have their properties extracted.
-   `extractProperty`: The name of the property to extract from each matched child.
-   `joinWith`: An optional string. If provided, all extracted values will be joined into a single string using this separator (e.g., `", "`).

## Usage Example

Let's say you have a UI tree representing a list of insurance quotes, and you want to extract the "carrier product", "monthly price", and "status" for each valid quote.

Consider the following simplified UI tree structure:

```json
{
  "name": "Root",
  "role": "Document",
  "children": [
    {
      "name": "Quote List Container",
      "role": "Group",
      "children": [
        {
          "name": "Quote 1 Container",
          "role": "Group",
          "children": [
            {"role": "Image", "name": "logo"},
            {"role": "Text", "name": "Prosperity PrimeTerm to 100: 20 YEAR TERM*"},
            {"role": "Text", "name": "$358.56"},
            {"role": "Text", "name": "Monthly Price"},
            {"role": "Text", "name": "Graded"},
            {"role": "Text", "name": "Discontinued"}
          ]
        },
        {
          "name": "Quote 2 Container",
          "role": "Group",
          "children": [
            {"role": "Image", "name": "acme-insurance-logo"},
            {"role": "Text", "name": "ACME Insurance: Term Life"},
            {"role": "Text", "name": "$120.00"},
            {"role": "Text", "name": "Monthly Price"},
            {"role": "Text", "name": "Standard"}
          ]
        }
      ]
    }
  ]
}
```

### Parser Definition

To extract the desired information, you would define your `OutputParserDefinition` as follows:

```json
{
  "itemContainerDefinition": {
    "nodeConditions": [
      {"property": "role", "op": "equals", "value": "Group"}
    ],
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
        "conditions": [{"property": "name", "op": "contains", "value": ":"}
        ],
        "extractProperty": "name"
      }
    },
    "monthlyPrice": {
      "fromChild": {
        "conditions": [{"property": "name", "op": "startsWith", "value": "$"}
        ],
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
}
```

### How it works

1.  **`itemContainerDefinition`**: It looks for nodes with `role: "Group"` that have children where one child's name starts with `"$"` AND another child's name is exactly `"Monthly Price"`. This identifies each valid quote container.

2.  **`fieldsToExtract.carrierProduct`**: For each identified quote container, it finds the first child with `name` containing `":"` (e.g., `"Prosperity PrimeTerm to 100: 20 YEAR TERM*"`), and extracts its `name` property.

3.  **`fieldsToExtract.monthlyPrice`**: It finds the first child with `name` starting with `"$"` (e.g., `"$358.56"`), and extracts its `name` property.

4.  **`fieldsToExtract.status`**: It finds all children with `role: "Text"` and whose `name` is one of `"Graded"`, `"Discontinued"`, or `"Standard"`. It then extracts their `name` properties and `join`s them with `", "`.

### Executing the Parser

You would use the `run_output_parser` function, providing the UI tree and the parser definition:

```rust
use serde_json::{json, Value};
use crate::output_parser::{run_output_parser, OutputParserDefinition};

// Assuming ui_tree_value is your captured UI tree as a serde_json::Value
// and parser_definition_value is your parser definition JSON as a serde_json::Value

let result = run_output_parser(&parser_definition_value, &ui_tree_value)?;

// The result will be a JSON array of extracted items:
// [
//   {
//     "carrierProduct": "Prosperity PrimeTerm to 100: 20 YEAR TERM*",
//     "monthlyPrice": "$358.56",
//     "status": "Graded, Discontinued"
//   },
//   {
//     "carrierProduct": "ACME Insurance: Term Life",
//     "monthlyPrice": "$120.00",
//     "status": "Standard"
//   }
// ]
```

This parser will return a list of JSON objects, each representing a quote with its extracted fields. 