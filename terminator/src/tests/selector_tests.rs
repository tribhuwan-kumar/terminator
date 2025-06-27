//! Tests for UI element selectors

use crate::{Desktop, Selector, UIElement};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// A test fixture that manages the lifecycle of an application for testing.
/// It ensures the application is opened before the test runs and closed after,
/// even if the test panics.
struct AppFixture {
    desktop: Arc<Desktop>,
    app: Option<UIElement>,
    rt: Runtime,
}

impl AppFixture {
    /// Creates a new fixture and launches the specified application.
    fn new(app_name: &str) -> Self {
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        let desktop = Arc::new(Desktop::new(false, false).expect("Failed to create Desktop"));
        let desktop_clone = desktop.clone();

        let app = rt.block_on(async {
            match desktop_clone.open_application(app_name) {
                Ok(app) => {
                    // Wait a bit for the app to be fully ready
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    Some(app)
                }
                Err(e) => {
                    panic!(
                        "Failed to open application '{}' for testing: {}",
                        app_name, e
                    );
                }
            }
        });

        AppFixture { desktop, app, rt }
    }
}

impl Drop for AppFixture {
    /// Ensures the application is closed when the fixture goes out of scope.
    fn drop(&mut self) {
        if let Some(app) = self.app.take() {
            let app_name = app.name().unwrap_or_else(|| "unknown".to_string());
            println!("--- Tearing down test, closing '{}' ---", app_name);
            if let Err(e) = app.close() {
                // It might already be closed, so just log the error.
                eprintln!(
                    "Error closing application '{}' during test teardown: {}",
                    app_name, e
                );
            }
            // Give a moment for the process to terminate
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

#[test]
fn test_id_selector_finds_element() {
    let fixture = AppFixture::new("notepad");
    let desktop = fixture.desktop.clone();
    let rt = &fixture.rt;

    rt.block_on(async {
        // 1. Find the main text area (the "Edit" control in Notepad) to get a target element.
        let edit_selector = Selector::Role {
            role: "edit".to_string(),
            name: None,
        };
        let text_area = desktop
            .locator(edit_selector)
            .first(None)
            .await
            .expect("Could not find the text area in Notepad.");

        // 2. Get the ID of the target element.
        let target_id = text_area.id().expect("Text area should have an ID.");
        println!("Found text area with ID: {}", target_id);
        assert!(!target_id.is_empty(), "Element ID should not be empty.");

        // 3. Create an ID selector from the retrieved ID.
        let id_selector_str = format!("#{}", target_id);
        let id_selector = Selector::from(id_selector_str.as_str());

        // 4. Use the ID selector to find the element again.
        let found_element = desktop
            .locator(id_selector)
            .first(None)
            .await
            .expect("Failed to find element using ID selector.");

        // 5. Verify that the found element is the same as the original.
        let found_id = found_element
            .id()
            .expect("Found element should have an ID.");
        assert_eq!(
            target_id, found_id,
            "The ID of the found element must match the target ID."
        );

        println!("✅ Successfully found element by ID: {}", target_id);
    });
}

#[test]
fn test_click_by_position_in_settings() {
    // Using a more specific app name for settings
    let fixture = AppFixture::new("ms-settings:");
    let desktop = fixture.desktop.clone();
    let rt = &fixture.rt;

    rt.block_on(async {
        // 1. Find the "System" button on the main settings page.
        // We give it a few seconds to appear.
        let system_button_selector = Selector::Role {
            role: "listitem".to_string(),
            name: Some("System".to_string()),
        };
        let system_button = desktop
            .locator(system_button_selector)
            .first(Some(std::time::Duration::from_secs(5)))
            .await
            .expect("Could not find the 'System' button in Settings.");

        // 2. Get its bounds and calculate the center point.
        let (x, y, width, height) = system_button
            .bounds()
            .expect("System button should have bounds.");
        let center_x = (x + width / 2.0) as i32;
        let center_y = (y + height / 2.0) as i32;
        println!(
            "Found 'System' button at ({}, {}). Clicking center at ({}, {})",
            x, y, center_x, center_y
        );

        // 3. Create a position selector for the center point.
        let pos_selector = Selector::from(format!("pos:{},{}", center_x, center_y).as_str());

        // 4. Use the position selector to find the element and click it.
        let element_at_pos = desktop
            .locator(pos_selector)
            .first(None)
            .await
            .expect("Failed to find element using position selector.");

        element_at_pos
            .click()
            .expect("Clicking by position failed.");

        // 5. Verify the click was successful by checking for navigation.
        // On the "System" page, there should be a "Display" option.
        println!("Clicked by position. Verifying navigation to System page...");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await; // Wait for page to load

        let display_selector = Selector::Name("Display".to_string());
        let display_item = desktop
            .locator(display_selector)
            .wait(Some(std::time::Duration::from_secs(5)))
            .await;

        assert!(
            display_item.is_ok(),
            "Failed to find 'Display' item after click. Navigation to System page likely failed."
        );

        println!("✅ Successfully clicked element by position and verified navigation.");

        // now find the close button and click it
        let close_button_selector = Selector::Role {
            role: "button".to_string(),
            name: Some("Close".to_string()),
        };
        let close_button = desktop
            .locator(close_button_selector)
            .first(None)
            .await
            .expect("Could not find the 'Close' button in Settings.");
        close_button
            .click()
            .expect("Clicking the 'Close' button failed.");
    });
}
