use std::time::Duration;
use terminator::Desktop;
use terminator_workflow_recorder::{WorkflowEvent, WorkflowRecorder, WorkflowRecorderConfig};
use tokio_stream::StreamExt;

/// Integration test for browser navigation events through keyboard shortcuts
/// Tests that browser shortcuts and navigation are properly captured
#[tokio::test]
#[ignore] // Run with: cargo test test_browser_navigation_shortcuts -- --ignored --nocapture
async fn test_browser_navigation_shortcuts() {
    println!("\n🌐 Starting Browser Navigation Shortcuts Integration Test");
    println!("========================================================");

    // Step 1: Configure recorder for browser events
    let config = WorkflowRecorderConfig {
        record_mouse: true,
        record_keyboard: true,
        capture_ui_elements: true,
        record_browser_tab_navigation: true,
        record_text_input_completion: false, // Focus on navigation events
        record_clipboard: false,
        record_hotkeys: true,
        track_modifier_states: true,
        ..Default::default()
    };

    let mut recorder = WorkflowRecorder::new("Browser Navigation Test".to_string(), config);
    let mut event_stream = recorder.event_stream();

    println!("✅ Workflow recorder configured for browser navigation");

    // Step 2: Start recording
    println!("🎬 Starting recording...");
    recorder.start().await.expect("Failed to start recording");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Step 3: Initialize desktop automation
    println!("🖥️  Initializing Terminator Desktop...");
    let desktop = Desktop::new(false, false).expect("Failed to create Desktop");

    // Step 4: Open browser and navigate to test page
    println!("🌍 Opening browser and navigating to test page...");
    let browser = desktop
        .open_url("https://httpbin.org", None)
        .expect("Failed to open browser");

    // Wait for page to load
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Step 5: Test various browser shortcuts
    println!("⌨️  Testing browser keyboard shortcuts...");

    let shortcuts = vec![
        ("{Ctrl}{T}", "New Tab"),
        ("{Ctrl}{L}", "Focus Address Bar"),
        ("{Ctrl}{R}", "Refresh Page"),
        ("{Alt}{Left}", "Back Navigation"),
        ("{Alt}{Right}", "Forward Navigation"),
        ("{F5}", "Refresh"),
    ];

    for (shortcut, description) in shortcuts {
        println!("   Testing {}: {}", description, shortcut);
        let _ = browser.press_key(shortcut);
        tokio::time::sleep(Duration::from_millis(2000)).await;
    }

    // Step 6: Test tab navigation
    println!("🔄 Testing tab navigation...");
    let _ = browser.press_key("{Ctrl}{1}"); // Switch to first tab
    tokio::time::sleep(Duration::from_millis(1000)).await;
    let _ = browser.press_key("{Ctrl}{T}"); // New tab
    tokio::time::sleep(Duration::from_millis(1000)).await;
    let _ = browser.press_key("{Ctrl}{2}"); // Switch to second tab
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Step 7: Test address bar typing
    println!("🌐 Testing address bar navigation...");
    let _ = browser.press_key("{Ctrl}{L}"); // Focus address bar
    tokio::time::sleep(Duration::from_millis(500)).await;
    let _ = browser.type_text("https://example.com", false);
    let _ = browser.press_key("{Enter}");
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Step 8: Stop recording
    println!("⏹️  Stopping recording...");
    recorder.stop().await.expect("Failed to stop recording");
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Step 9: Collect and analyze events
    println!("📊 Collecting and analyzing captured events...");
    let mut captured_events = Vec::new();
    let mut timeout_count = 0;
    const MAX_TIMEOUTS: usize = 15;

    while timeout_count < MAX_TIMEOUTS {
        tokio::select! {
            event = event_stream.next() => {
                if let Some(event) = event {
                    captured_events.push(event);
                    timeout_count = 0;
                } else {
                    break;
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                timeout_count += 1;
            }
        }
    }

    println!("📈 Total events captured: {}", captured_events.len());

    // Step 10: Analyze browser navigation events
    let mut button_click_events = Vec::new();
    let mut browser_nav_events = Vec::new();
    let mut keyboard_events = 0;
    let mut hotkey_events = 0;

    for event in &captured_events {
        match event {
            WorkflowEvent::ButtonClick(button_event) => {
                button_click_events.push(button_event);
            }
            WorkflowEvent::BrowserTabNavigation(nav_event) => {
                browser_nav_events.push(nav_event);
            }
            WorkflowEvent::Keyboard(kb_event) => {
                keyboard_events += 1;
                if kb_event.ctrl_pressed || kb_event.alt_pressed || kb_event.shift_pressed {
                    hotkey_events += 1;
                }
            }
            _ => {}
        }
    }

    println!("📊 Browser Navigation Event Analysis:");
    println!("   - Button clicks: {}", button_click_events.len());
    println!("   - Browser tab navigation: {}", browser_nav_events.len());
    println!("   - Keyboard events: {}", keyboard_events);
    println!("   - Hotkey combinations: {}", hotkey_events);

    // Step 11: Verify browser navigation events
    println!("\n🔍 Verifying Browser Navigation Events:");

    // Verify we captured some events
    assert!(
        keyboard_events > 0,
        "❌ No keyboard events captured! Expected browser shortcuts."
    );

    assert!(
        hotkey_events > 0,
        "❌ No hotkey combinations captured! Expected browser shortcuts."
    );

    // Verify browser navigation events contain meaningful data
    for (i, event) in browser_nav_events.iter().enumerate() {
        println!(
            "   Browser Nav {}: Action={:?}, Method={:?}",
            i + 1,
            event.action,
            event.method
        );

        if let Some(ref to_url) = event.to_url {
            println!("     - To URL: '{}'", to_url);
        }
        if let Some(ref to_title) = event.to_title {
            println!("     - To Title: '{}'", to_title);
        }
        println!("     - Browser: '{}'", event.browser);
    }

    // Step 12: Clean up - Close browser
    println!("🧹 Cleaning up...");
    let _ = browser.close();

    println!("\n✅ Browser Navigation Shortcuts Test PASSED!");
    println!("   - Keyboard events: {}", keyboard_events);
    println!("   - Hotkey combinations: {}", hotkey_events);
    println!("   - Browser navigation: {}", browser_nav_events.len());
    println!("   - Button interactions: {}", button_click_events.len());
}

/// Integration test for browser form interactions
/// Tests form filling and text input completion events
#[tokio::test]
#[ignore] // Run with: cargo test test_browser_form_interactions -- --ignored --nocapture
async fn test_browser_form_interactions() {
    println!("\n📝 Starting Browser Form Interactions Integration Test");
    println!("====================================================");

    let config = WorkflowRecorderConfig {
        record_mouse: true,
        record_keyboard: true,
        capture_ui_elements: true,
        record_text_input_completion: true,
        text_input_completion_timeout_ms: 1500,
        record_browser_tab_navigation: true,
        record_clipboard: false,
        record_hotkeys: false,
        ..Default::default()
    };

    let mut recorder = WorkflowRecorder::new("Browser Form Test".to_string(), config);
    let mut event_stream = recorder.event_stream();

    println!("✅ Workflow recorder configured for form interactions");

    // Start recording
    recorder.start().await.expect("Failed to start recording");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Initialize desktop
    let desktop = Desktop::new(false, false).expect("Failed to create Desktop");

    // Open a test form page
    println!("📄 Opening test form page...");
    let browser = desktop
        .open_url("https://httpbin.org/forms/post", None)
        .expect("Failed to open form page");

    tokio::time::sleep(Duration::from_secs(3)).await;

    // Test form interactions
    println!("📝 Testing form field interactions...");

    // Try to find and fill form fields using the correct pattern
    if let Ok(inputs) = browser
        .locator("role:Edit")
        .unwrap()
        .all(Some(Duration::from_secs(5)), None)
        .await
    {
        for (i, input) in inputs.iter().enumerate().take(3) {
            let test_value = format!("test_value_{}", i + 1);
            println!("   Filling input {}: '{}'", i + 1, test_value);

            let _ = input.click();
            tokio::time::sleep(Duration::from_millis(300)).await;
            let _ = input.type_text(&test_value, true);
            tokio::time::sleep(Duration::from_millis(1500)).await; // Longer wait for text completion
            let _ = input.press_key("{Tab}");
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    // Try to find and click submit button
    println!("🚀 Testing form submission...");
    if let Ok(submit_button) = browser
        .locator("role:button")
        .unwrap()
        .first(Some(Duration::from_secs(3)))
        .await
    {
        let _ = submit_button.click();
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    // Stop recording
    recorder.stop().await.expect("Failed to stop recording");
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Collect events
    let mut captured_events = Vec::new();
    let mut timeout_count = 0;
    const MAX_TIMEOUTS: usize = 15;

    while timeout_count < MAX_TIMEOUTS {
        tokio::select! {
            event = event_stream.next() => {
                if let Some(event) = event {
                    captured_events.push(event);
                    timeout_count = 0;
                } else {
                    break;
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                timeout_count += 1;
            }
        }
    }

    // Analyze form interaction events
    let mut text_input_events = Vec::new();
    let mut button_click_events = Vec::new();
    let mut keyboard_events = 0;

    for event in &captured_events {
        match event {
            WorkflowEvent::TextInputCompleted(text_event) => {
                text_input_events.push(text_event);
            }
            WorkflowEvent::ButtonClick(button_event) => {
                button_click_events.push(button_event);
            }
            WorkflowEvent::Keyboard(_) => keyboard_events += 1,
            _ => {}
        }
    }

    println!("📊 Form Interaction Analysis:");
    println!("   - Text input completions: {}", text_input_events.len());
    println!("   - Button clicks: {}", button_click_events.len());
    println!("   - Keyboard events: {}", keyboard_events);

    // Verify form events
    println!("\n🔍 Verifying Form Interaction Events:");

    // Should have captured some interactions
    let total_interactions = text_input_events.len() + button_click_events.len();
    assert!(
        total_interactions > 0 || keyboard_events > 0,
        "❌ No form interaction events captured!"
    );

    // Verify text input events if any were captured
    if !text_input_events.is_empty() {
        println!("✅ Text input events captured:");
        for (i, event) in text_input_events.iter().enumerate() {
            println!(
                "   Input {}: '{}' (method: {:?})",
                i + 1,
                event.text_value,
                event.input_method
            );
            assert!(
                !event.text_value.trim().is_empty(),
                "❌ Text input event {} has empty value",
                i + 1
            );
        }
    }

    // Verify button click events if any were captured
    if !button_click_events.is_empty() {
        println!("✅ Button click events captured:");
        for (i, event) in button_click_events.iter().enumerate() {
            println!(
                "   Button {}: '{}' (type: {:?})",
                i + 1,
                event.button_text,
                event.interaction_type
            );
        }
    }

    // Clean up
    let _ = browser.close();

    println!("\n✅ Browser Form Interactions Test PASSED!");
    println!("   - Text inputs: {}", text_input_events.len());
    println!("   - Button clicks: {}", button_click_events.len());
    println!("   - Keyboard events: {}", keyboard_events);
}

/// Integration test for mouse click events in browser
/// Tests button clicks and UI interactions
#[tokio::test]
#[ignore] // Run with: cargo test test_browser_mouse_interactions -- --ignored --nocapture
async fn test_browser_mouse_interactions() {
    println!("\n🖱️  Starting Browser Mouse Interactions Integration Test");
    println!("========================================================");

    let config = WorkflowRecorderConfig {
        record_mouse: true,
        record_keyboard: false, // Focus on mouse only
        capture_ui_elements: true,
        record_browser_tab_navigation: true,
        record_text_input_completion: false,
        record_clipboard: false,
        record_hotkeys: false,
        ..Default::default()
    };

    let mut recorder = WorkflowRecorder::new("Browser Mouse Test".to_string(), config);
    let mut event_stream = recorder.event_stream();

    // Start recording
    recorder.start().await.expect("Failed to start recording");
    tokio::time::sleep(Duration::from_millis(500)).await;

    let desktop = Desktop::new(false, false).expect("Failed to create Desktop");

    // Open browser
    println!("🌐 Opening browser...");
    let browser = desktop
        .open_url("https://httpbin.org", None)
        .expect("Failed to open browser");

    tokio::time::sleep(Duration::from_secs(3)).await;

    // Test various mouse interactions
    println!("🖱️  Testing mouse interactions...");

    // Try to click on elements
    if let Ok(elements) = browser
        .locator("role:Hyperlink")
        .unwrap()
        .all(Some(Duration::from_secs(5)), None)
        .await
    {
        if !elements.is_empty() {
            println!(
                "   Found {} clickable elements, clicking first few",
                elements.len()
            );
            for (i, element) in elements.iter().enumerate().take(2) {
                println!(
                    "   Clicking element {} ({})",
                    i + 1,
                    element.name().unwrap_or_default()
                );
                let _ = element.click();
                tokio::time::sleep(Duration::from_millis(1500)).await;
            }
        }
    }

    // Test browser navigation with mouse (back button simulation)
    println!("🔙 Testing browser back navigation...");
    let _ = browser.press_key("{Alt}{Left}"); // Back
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Stop recording
    recorder.stop().await.expect("Failed to stop recording");
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Collect events
    let mut captured_events = Vec::new();
    let mut timeout_count = 0;
    const MAX_TIMEOUTS: usize = 10;

    while timeout_count < MAX_TIMEOUTS {
        tokio::select! {
            event = event_stream.next() => {
                if let Some(event) = event {
                    captured_events.push(event);
                    timeout_count = 0;
                } else {
                    break;
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                timeout_count += 1;
            }
        }
    }

    // Analyze mouse events
    let mut mouse_events = 0;
    let mut button_click_events = Vec::new();
    let mut browser_nav_events = Vec::new();

    for event in &captured_events {
        match event {
            WorkflowEvent::Mouse(_) => mouse_events += 1,
            WorkflowEvent::ButtonClick(button_event) => {
                button_click_events.push(button_event);
            }
            WorkflowEvent::BrowserTabNavigation(nav_event) => {
                browser_nav_events.push(nav_event);
            }
            _ => {}
        }
    }

    println!("📊 Mouse Interaction Analysis:");
    println!("   - Mouse events: {}", mouse_events);
    println!("   - Button clicks: {}", button_click_events.len());
    println!("   - Browser navigation: {}", browser_nav_events.len());

    // Verify we captured mouse interactions
    assert!(
        mouse_events > 0 || button_click_events.len() > 0,
        "❌ No mouse interaction events captured!"
    );

    // Clean up
    let _ = browser.close();

    println!("\n✅ Browser Mouse Interactions Test PASSED!");
    println!("   - Mouse events: {}", mouse_events);
    println!("   - Button clicks: {}", button_click_events.len());
    println!("   - Browser navigation: {}", browser_nav_events.len());
}
