use crate::Selector;

#[test]
fn test_window_and_name_notepad() {
    // Test the exact selector pattern we're using
    let sel = Selector::from("role:Window && name:Notepad");
    match sel {
        Selector::And(v) => {
            assert_eq!(v.len(), 2);
            match &v[0] {
                Selector::Role { role, name } => {
                    assert_eq!(role, "Window");
                    assert_eq!(*name, None);
                }
                _ => panic!("Expected Role selector for Window, got: {:?}", v[0]),
            }
            match &v[1] {
                Selector::Name(n) => {
                    assert_eq!(n, "Notepad");
                }
                _ => panic!("Expected Name selector, got: {:?}", v[1]),
            }
        }
        _ => panic!("Expected And selector, got: {:?}", sel),
    }
}

#[test]
fn test_document_or_edit() {
    let sel = Selector::from("role:Document || role:Edit");
    match sel {
        Selector::Or(v) => {
            assert_eq!(v.len(), 2);
            match &v[0] {
                Selector::Role { role, name } => {
                    assert_eq!(role, "Document");
                    assert_eq!(*name, None);
                }
                _ => panic!("Expected Role selector for Document, got: {:?}", v[0]),
            }
            match &v[1] {
                Selector::Role { role, name } => {
                    assert_eq!(role, "Edit");
                    assert_eq!(*name, None);
                }
                _ => panic!("Expected Role selector for Edit, got: {:?}", v[1]),
            }
        }
        _ => panic!("Expected Or selector, got: {:?}", sel),
    }
}

#[test]
fn test_chain_with_window_selector() {
    // Test the chain selector pattern: window >> element
    let sel = Selector::from("role:Window && name:Notepad >> role:Document");
    match sel {
        Selector::Chain(parts) => {
            assert_eq!(parts.len(), 2);
            // First part should be AND (Window && name:Notepad)
            match &parts[0] {
                Selector::And(v) => {
                    assert_eq!(v.len(), 2);
                }
                _ => panic!("Expected And selector in chain first part, got: {:?}", parts[0]),
            }
            // Second part should be Role (Document)
            match &parts[1] {
                Selector::Role { role, name } => {
                    assert_eq!(role, "Document");
                    assert_eq!(*name, None);
                }
                _ => panic!("Expected Role selector in chain second part, got: {:?}", parts[1]),
            }
        }
        _ => panic!("Expected Chain selector, got: {:?}", sel),
    }
}