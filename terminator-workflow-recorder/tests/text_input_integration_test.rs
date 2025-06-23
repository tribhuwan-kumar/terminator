use std::time::Duration;
use terminator::Desktop;
use terminator_workflow_recorder::{WorkflowEvent, WorkflowRecorder, WorkflowRecorderConfig};
use tokio::time::sleep;
use tokio_stream::StreamExt;

/// Test text input completion events with proper keystroke counting and timing
#[tokio::test]
#[ignore] // Run manually with: cargo test test_text_input_completion_comprehensive -- --ignored
async fn test_text_input_completion_comprehensive() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Starting comprehensive text input completion test...");

    // Create recorder with text input completion enabled
    let config = WorkflowRecorderConfig {
        record_text_input_completion: true,
        capture_ui_elements: true,
        record_application_switches: true,
        record_keyboard: true,
        record_mouse: true,
        record_clipboard: false, // Reduce noise
        record_hotkeys: false,   // Reduce noise
        ..Default::default()
    };

    let mut recorder = WorkflowRecorder::new("Text Input Test".to_string(), config);
    let mut event_stream = recorder.event_stream();

    // Start recording
    println!("🎬 Starting recorder...");
    match recorder.start().await {
        Ok(_) => println!("✅ Recording started successfully"),
        Err(e) => {
            println!("❌ Failed to start recording: {:?}", e);
            return Err(e.into());
        }
    }

    // Give the recorder a moment to start
    sleep(Duration::from_millis(1000)).await;
    println!("⏳ Recorder should be ready now");

    // Open Notepad for testing
    let desktop = Desktop::new(false, false)?;
    let _notepad_app = desktop.open_application("notepad.exe")?;
    sleep(Duration::from_secs(2)).await;

    println!("📝 Notepad opened");

    // Find the text editor element
    let locator = desktop.locator("role:Edit");
    let text_editor = locator.first(Some(Duration::from_secs(10))).await?;

    // Test 1: Type text and press Enter (should trigger completion)
    println!("\n📝 Test 1: Typing text + Enter key");
    text_editor.type_text("Hello World", false)?;
    sleep(Duration::from_millis(500)).await;
    text_editor.press_key("{Enter}")?; // Should trigger text input completion
    sleep(Duration::from_millis(1000)).await;

    // Test 2: Type more text and press Tab (should trigger completion)
    println!("📝 Test 2: Typing text + Tab key");
    text_editor.type_text("This is a test", false)?;
    sleep(Duration::from_millis(500)).await;
    text_editor.press_key("{Tab}")?; // Should trigger text input completion
    sleep(Duration::from_millis(1000)).await;

    // Test 3: Type text and click elsewhere (focus change)
    println!("📝 Test 3: Typing text + focus change");
    text_editor.type_text("Final test line", false)?;
    sleep(Duration::from_millis(500)).await;

    // Click on the window title bar to change focus
    text_editor.press_key("{Ctrl}a")?; // Select all to trigger completion
    sleep(Duration::from_millis(1000)).await;

    // Test 4: Type a short burst and wait for timeout
    println!("📝 Test 4: Typing text + natural timeout");
    text_editor.press_key("{End}")?; // Go to end
    text_editor.type_text(" - timeout test", false)?;
    sleep(Duration::from_millis(3000)).await; // Wait longer than timeout to trigger completion

    // Stop recording and collect events
    println!("⏹️ Stopping recorder...");
    match recorder.stop().await {
        Ok(_) => println!("✅ Recording stopped successfully"),
        Err(e) => {
            println!("❌ Failed to stop recording: {:?}", e);
            return Err(e.into());
        }
    }

    // Give more time for events to be processed
    sleep(Duration::from_millis(2000)).await;
    println!("⏹️ Recording stopped, collecting events...");

    // Collect all events
    let mut all_events = Vec::new();
    let timeout = Duration::from_secs(5);
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < timeout {
        match tokio::time::timeout(Duration::from_millis(100), event_stream.next()).await {
            Ok(Some(event)) => {
                println!("📦 Received event: {:?}", std::mem::discriminant(&event));
                all_events.push(event)
            }
            Ok(None) => {
                println!("📦 Event stream ended");
                break;
            }
            Err(_) => {
                println!("📦 Timeout waiting for events");
                break;
            }
        }
    }

    println!("📊 Collected {} total events", all_events.len());

    // Show event breakdown
    let mut event_counts = std::collections::HashMap::new();
    for event in &all_events {
        let event_type = match event {
            WorkflowEvent::Mouse(_) => "Mouse",
            WorkflowEvent::Keyboard(_) => "Keyboard",
            WorkflowEvent::TextInputCompleted(_) => "TextInputCompleted",
            WorkflowEvent::ApplicationSwitch(_) => "ApplicationSwitch",
            WorkflowEvent::BrowserTabNavigation(_) => "BrowserTabNavigation",
            WorkflowEvent::ButtonClick(_) => "ButtonClick",
            WorkflowEvent::Hotkey(_) => "Hotkey",
            _ => "Other",
        };
        *event_counts.entry(event_type).or_insert(0) += 1;
    }

    println!("📊 Event breakdown:");
    for (event_type, count) in &event_counts {
        println!("   - {}: {}", event_type, count);
    }

    // Analyze text input completion events
    let text_input_events: Vec<_> = all_events
        .iter()
        .filter_map(|event| {
            if let WorkflowEvent::TextInputCompleted(text_event) = event {
                Some(text_event)
            } else {
                None
            }
        })
        .collect();

    println!("\n🔍 TEXT INPUT COMPLETION ANALYSIS:");
    println!(
        "Found {} text input completion events",
        text_input_events.len()
    );

    for (i, event) in text_input_events.iter().enumerate() {
        println!(
            "🔥 TEXT INPUT COMPLETED {}: \"{}\" ({} keystrokes in {}ms)",
            i + 1,
            event.text_value,
            event.keystroke_count,
            event.typing_duration_ms
        );
        println!(
            "     └─ Field: \"{:?}\" ({})",
            event.field_name, event.field_type
        );
        println!("     └─ Method: {:?}", event.input_method);
    }

    // Assertions
    assert!(
        text_input_events.len() >= 2,
        "Expected at least 2 text input completion events, got {}",
        text_input_events.len()
    );

    // Check that we have proper keystroke counts (should be > 0 for typed text)
    let typed_events: Vec<_> = text_input_events
        .iter()
        .filter(|event| event.keystroke_count > 0)
        .collect();

    assert!(
        typed_events.len() >= 1,
        "Expected at least 1 event with keystroke counts > 0, got {}",
        typed_events.len()
    );

    // Check for specific text content
    let hello_world_events: Vec<_> = text_input_events
        .iter()
        .filter(|event| event.text_value.contains("Hello World"))
        .collect();

    let test_events: Vec<_> = text_input_events
        .iter()
        .filter(|event| event.text_value.contains("test"))
        .collect();

    assert!(
        !hello_world_events.is_empty() || !test_events.is_empty(),
        "Expected to find events with our test text content"
    );

    // Check timing - events should have reasonable durations
    for event in &text_input_events {
        assert!(
            event.typing_duration_ms > 0,
            "Text input completion should have positive duration, got {}",
            event.typing_duration_ms
        );
        assert!(
            event.typing_duration_ms < 30000,
            "Text input completion duration seems too long: {}ms",
            event.typing_duration_ms
        );
    }

    // Check that field type is appropriate for Notepad
    for event in &text_input_events {
        let field_type_lower = event.field_type.to_lowercase();
        assert!(
            field_type_lower.contains("edit")
                || field_type_lower.contains("document")
                || field_type_lower.contains("text"),
            "Unexpected field type for Notepad: {}",
            event.field_type
        );
    }

    println!("\n✅ TEXT INPUT COMPLETION TEST PASSED!");
    println!(
        "   ✓ {} completion events captured",
        text_input_events.len()
    );
    println!("   ✓ {} events with keystroke tracking", typed_events.len());
    println!("   ✓ Text content properly captured");
    println!("   ✓ All events have reasonable timing");
    println!("   ✓ Field types are appropriate for text input");

    Ok(())
}

/// Test that empty or unchanged text inputs don't generate spurious events
#[tokio::test]
#[ignore] // Run manually with: cargo test test_text_input_no_spurious_events -- --ignored
async fn test_text_input_no_spurious_events() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing that empty/unchanged inputs don't generate spurious events...");

    let config = WorkflowRecorderConfig {
        record_text_input_completion: true,
        capture_ui_elements: true,
        record_application_switches: false, // Reduce noise
        record_keyboard: false,             // Reduce noise
        record_mouse: false,                // Reduce noise
        record_clipboard: false,            // Reduce noise
        record_hotkeys: false,              // Reduce noise
        ..Default::default()
    };

    let mut recorder = WorkflowRecorder::new("No Spurious Events Test".to_string(), config);
    let mut event_stream = recorder.event_stream();
    recorder.start().await?;

    let desktop = Desktop::new(false, false)?;
    let _notepad_app = desktop.open_application("notepad.exe")?;
    sleep(Duration::from_secs(2)).await;

    // Test clicking in and out of the text area without typing
    println!("🖱️ Clicking in text area without typing...");

    let locator = desktop.locator("role:Edit");
    let text_editor = locator.first(Some(Duration::from_secs(10))).await?;

    // Click multiple times without typing
    text_editor.click()?;
    sleep(Duration::from_millis(500)).await;

    text_editor.press_key("{Home}")?; // Navigate without typing
    sleep(Duration::from_millis(500)).await;

    text_editor.press_key("{End}")?; // Navigate without typing
    sleep(Duration::from_millis(500)).await;

    // Press Tab without any text content
    text_editor.press_key("{Tab}")?;
    sleep(Duration::from_millis(1000)).await;

    recorder.stop().await?;

    // Collect events
    let mut all_events = Vec::new();
    let timeout = Duration::from_secs(2);
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < timeout {
        match tokio::time::timeout(Duration::from_millis(100), event_stream.next()).await {
            Ok(Some(event)) => all_events.push(event),
            Ok(None) => break,
            Err(_) => break,
        }
    }

    // Check for text input completion events
    let text_input_events: Vec<_> = all_events
        .iter()
        .filter_map(|event| {
            if let WorkflowEvent::TextInputCompleted(text_event) = event {
                Some(text_event)
            } else {
                None
            }
        })
        .collect();

    println!(
        "📊 Found {} text input completion events (should be 0 or very few)",
        text_input_events.len()
    );

    for event in &text_input_events {
        println!(
            "⚠️ Unexpected event: \"{}\" in field \"{:?}\"",
            event.text_value, event.field_name
        );
    }

    // Should have 0 or very few spurious events
    assert!(
        text_input_events.len() <= 1,
        "Too many spurious text input completion events: {}",
        text_input_events.len()
    );

    println!("✅ NO SPURIOUS EVENTS TEST PASSED!");

    Ok(())
}

/// Simple test to verify basic recording functionality works
#[tokio::test]
#[ignore] // Run manually with: cargo test test_basic_recording_works -- --ignored
async fn test_basic_recording_works() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing basic recording functionality...");

    // Create recorder with minimal features
    let config = WorkflowRecorderConfig {
        record_text_input_completion: false, // Disable our new feature
        capture_ui_elements: false,          // Disable UI capture
        record_application_switches: false,  // Disable
        record_keyboard: true,               // Keep basic keyboard
        record_mouse: false,                 // Disable mouse
        record_clipboard: false,             // Disable
        record_hotkeys: false,               // Disable
        ..Default::default()
    };

    let mut recorder = WorkflowRecorder::new("Basic Test".to_string(), config);
    let mut event_stream = recorder.event_stream();

    // Start recording
    println!("🎬 Starting basic recorder...");
    recorder.start().await?;
    println!("✅ Basic recording started");

    // Give it a moment
    sleep(Duration::from_millis(1000)).await;

    // Open Notepad and do some basic typing
    let desktop = Desktop::new(false, false)?;
    let _notepad_app = desktop.open_application("notepad.exe")?;
    sleep(Duration::from_secs(2)).await;

    // Find text editor and type something simple
    let locator = desktop.locator("role:Edit");
    let text_editor = locator.first(Some(Duration::from_secs(10))).await?;
    text_editor.type_text("Hello", false)?;

    sleep(Duration::from_millis(2000)).await;

    // Stop recording
    println!("⏹️ Stopping basic recorder...");
    recorder.stop().await?;
    println!("✅ Basic recording stopped");

    // Collect events
    let mut all_events = Vec::new();
    let timeout = Duration::from_secs(3);
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < timeout {
        match tokio::time::timeout(Duration::from_millis(100), event_stream.next()).await {
            Ok(Some(event)) => {
                println!("📦 Basic event: {:?}", std::mem::discriminant(&event));
                all_events.push(event);
            }
            Ok(None) => {
                println!("📦 Basic event stream ended");
                break;
            }
            Err(_) => {
                println!("📦 Basic timeout");
                continue;
            }
        }
    }

    println!("📊 Basic test collected {} events", all_events.len());

    // Just verify we got some events
    assert!(
        all_events.len() > 0,
        "Expected some events, got {}",
        all_events.len()
    );

    println!("✅ BASIC RECORDING TEST PASSED!");

    Ok(())
}

/// Test text input completion with Windows Run dialog (more reliable than Notepad)
#[tokio::test]
#[ignore] // Run manually with: cargo test test_text_input_run_dialog -- --ignored
async fn test_text_input_run_dialog() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing text input completion with Run dialog...");

    // Create recorder with text input completion enabled
    let config = WorkflowRecorderConfig {
        record_text_input_completion: true,
        capture_ui_elements: true,
        record_application_switches: false,
        record_keyboard: true,
        record_mouse: false,
        record_clipboard: false,
        record_hotkeys: false,
        ..Default::default()
    };

    let mut recorder = WorkflowRecorder::new("Text Input Run Dialog Test".to_string(), config);
    let mut event_stream = recorder.event_stream();

    // Start recording
    println!("🎬 Starting recorder...");
    recorder.start().await?;
    println!("✅ Recording started successfully");

    sleep(Duration::from_millis(1000)).await;

    // Use a web form for testing (more reliable than Run dialog)
    let desktop = Desktop::new(false, false)?;

    println!("📝 Opening test form page...");
    let browser = desktop.open_url("https://httpbin.org/forms/post", None)?;
    sleep(Duration::from_secs(3)).await;

    // Find text inputs and test typing
    println!("🔍 Finding text inputs...");
    let locator = browser.locator("role:Edit").unwrap();

    if let Ok(inputs) = locator.all(Some(Duration::from_secs(5)), None).await {
        for (i, input) in inputs.iter().enumerate().take(2) {
            let test_text = format!("test_text_{}", i + 1);
            println!(
                "📝 Test {}: Type '{}' and press Enter/Tab",
                i + 1,
                test_text
            );

            let _ = input.click();
            sleep(Duration::from_millis(300)).await;
            let _ = input.type_text(&test_text, true);
            sleep(Duration::from_millis(1500)).await; // Wait for text completion

            if i == 0 {
                let _ = input.press_key("{Enter}");
            } else {
                let _ = input.press_key("{Tab}");
            }
            sleep(Duration::from_millis(500)).await;
        }
    }

    // Test focus change by clicking somewhere else
    println!("📝 Test 3: Focus change");
    if let Ok(submit_btn) = browser
        .locator("role:button")
        .unwrap()
        .first(Some(Duration::from_secs(3)))
        .await
    {
        let _ = submit_btn.click();
        sleep(Duration::from_millis(1000)).await;
    }

    // Close browser
    println!("🧹 Closing browser...");
    let _ = browser.close();
    sleep(Duration::from_millis(1000)).await;

    // Stop recording
    println!("⏹️ Stopping recorder...");
    recorder.stop().await?;
    println!("✅ Recording stopped successfully");

    sleep(Duration::from_millis(2000)).await;
    println!("⏹️ Recording stopped, collecting events...");

    // Collect all events
    let mut all_events = Vec::new();
    let timeout = Duration::from_secs(5);
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < timeout {
        match tokio::time::timeout(Duration::from_millis(100), event_stream.next()).await {
            Ok(Some(event)) => {
                println!("📦 Received event: {:?}", std::mem::discriminant(&event));
                all_events.push(event)
            }
            Ok(None) => {
                println!("📦 Event stream ended");
                break;
            }
            Err(_) => {
                println!("📦 Timeout waiting for events");
                break;
            }
        }
    }

    println!("📊 Collected {} total events", all_events.len());

    // Show event breakdown
    let mut event_counts = std::collections::HashMap::new();
    for event in &all_events {
        let event_type = match event {
            WorkflowEvent::Mouse(_) => "Mouse",
            WorkflowEvent::Keyboard(_) => "Keyboard",
            WorkflowEvent::TextInputCompleted(_) => "TextInputCompleted",
            WorkflowEvent::ApplicationSwitch(_) => "ApplicationSwitch",
            WorkflowEvent::BrowserTabNavigation(_) => "BrowserTabNavigation",
            WorkflowEvent::ButtonClick(_) => "ButtonClick",
            WorkflowEvent::Hotkey(_) => "Hotkey",
            _ => "Other",
        };
        *event_counts.entry(event_type).or_insert(0) += 1;
    }

    println!("📊 Event breakdown:");
    for (event_type, count) in &event_counts {
        println!("   - {}: {}", event_type, count);
    }

    // Analyze text input completion events
    let text_input_events: Vec<_> = all_events
        .iter()
        .filter_map(|e| {
            if let WorkflowEvent::TextInputCompleted(event) = e {
                Some(event)
            } else {
                None
            }
        })
        .collect();

    println!("🔍 TEXT INPUT COMPLETION ANALYSIS:");
    println!(
        "Found {} text input completion events",
        text_input_events.len()
    );

    for (i, event) in text_input_events.iter().enumerate() {
        println!(
            "  {}. Field: {:?} ({})",
            i + 1,
            event.field_name,
            event.field_type
        );
        println!("     Text: '{}'", event.text_value);
        println!(
            "     Keystrokes: {}, Duration: {}ms",
            event.keystroke_count, event.typing_duration_ms
        );
        println!("     Method: {:?}", event.input_method);
    }

    // We should have at least 1 text input completion event
    assert!(
        text_input_events.len() >= 1,
        "Expected at least 1 text input completion event, got {}",
        text_input_events.len()
    );

    println!("✅ TEXT INPUT COMPLETION TEST PASSED!");

    Ok(())
}
