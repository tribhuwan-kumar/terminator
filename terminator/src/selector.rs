use std::collections::BTreeMap;

/// Represents ways to locate a UI element
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Selector {
    /// Select by role and optional name
    Role { role: String, name: Option<String> },
    /// Select by accessibility ID
    Id(String),
    /// Select by name/label
    Name(String),
    /// Select by text content
    Text(String),
    /// Select using XPath-like query
    Path(String),
    /// Select by using Native Automation id, (eg: `AutomationID` for windows) and for linux it is Id value in Attributes
    NativeId(String),
    /// Select by multiple attributes (key-value pairs)
    Attributes(BTreeMap<String, String>),
    /// Filter current elements by a predicate
    Filter(usize), // Uses an ID to reference a filter predicate stored separately
    /// Chain multiple selectors
    Chain(Vec<Selector>),
    /// Select by class name
    ClassName(String),
    /// Filter by visibility on screen
    Visible(bool),
    /// Select by localized role
    LocalizedRole(String),
    /// Select by position (x,y) on screen
    Position(i32, i32),
}

impl std::fmt::Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&str> for Selector {
    fn from(s: &str) -> Self {
        // Handle chained selectors first
        let parts: Vec<&str> = s.split(">>").map(|p| p.trim()).collect();
        if parts.len() > 1 {
            return Selector::Chain(parts.into_iter().map(Selector::from).collect());
        }

        // if using pipe, use it for the role plus name (preferred precise format)
        if s.contains('|') {
            let parts: Vec<&str> = s.split('|').collect();
            if parts.len() >= 2 {
                return Selector::Role {
                    role: parts[0].trim().to_string(),
                    name: Some(parts[1].trim().to_string()),
                };
            }
        }

        // Make common UI roles like "window", "button", etc. default to Role selectors
        // instead of Name selectors
        match s {
            // if role:button
            _ if s.starts_with("role:") => Selector::Role {
                role: s[5..].to_string(),
                name: None,
            },
            "app" | "application" | "window" | "button" | "checkbox" | "menu" | "menuitem"
            | "menubar" | "textfield" | "input" => {
                let parts: Vec<&str> = s.splitn(2, ':').collect();
                Selector::Role {
                    role: parts.first().unwrap_or(&"").to_string(),
                    name: parts.get(1).map(|name| name.to_string()), // optional
                }
            }
            // starts with AX
            _ if s.starts_with("AX") => Selector::Role {
                role: s.to_string(),
                name: None,
            },
            _ if s.starts_with("Name:") || s.starts_with("name:") => {
                let parts: Vec<&str> = s.splitn(2, ':').collect();
                Selector::Name(parts[1].to_string())
            }
            _ if s.to_lowercase().starts_with("classname:") => {
                let parts: Vec<&str> = s.splitn(2, ':').collect();
                Selector::ClassName(parts[1].to_string())
            }
            _ if s.to_lowercase().starts_with("nativeid:") => {
                let parts: Vec<&str> = s.splitn(2, ':').collect();
                Selector::NativeId(parts[1].trim().to_string())
            }
            _ if s.to_lowercase().starts_with("visible:") => {
                let value = s[8..].trim().to_lowercase();
                Selector::Visible(value == "true")
            }
            _ if s.to_lowercase().starts_with("pos:") => {
                let parts: Vec<&str> = s[4..].split(',').map(|p| p.trim()).collect();
                if parts.len() == 2 {
                    if let (Ok(x), Ok(y)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                        return Selector::Position(x, y);
                    }
                }
                // Fallback to name if format is wrong
                Selector::Name(s.to_string())
            }
            _ if s.starts_with("id:") => Selector::Id(s[3..].to_string()),
            _ if s.starts_with("text:") => Selector::Text(s[5..].to_string()),
            _ if s.contains(':') => {
                let parts: Vec<&str> = s.splitn(2, ':').collect();
                Selector::Role {
                    role: parts[0].to_string(),
                    name: Some(parts[1].to_string()),
                }
            }
            _ if s.starts_with('#') => Selector::Id(s[1..].to_string()),
            _ if s.starts_with('/') => Selector::Path(s.to_string()),
            _ if s.to_lowercase().starts_with("text:") => Selector::Text(s[5..].to_string()),
            _ => Selector::Name(s.to_string()),
        }
    }
}
