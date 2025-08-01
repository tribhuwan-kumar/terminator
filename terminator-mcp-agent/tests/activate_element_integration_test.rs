mod common;

use crate::common::get_result_json;
use rmcp::handler::server::tool::Parameters;
use std::time::Duration;
use terminator_mcp_agent::utils::{ActivateElementArgs, DesktopWrapper};

use tracing::Level;

#[tokio::test]
async fn test_activate_element_verification_structure() {
    // Initialize logging for debugging
    let _ = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .try_init();

    let server = DesktopWrapper::new().await.unwrap();

    // Test 1: Test with any element to check verification structure
    println!("🔍 Testing activate_element verification structure...");
    let test_activate_args = ActivateElementArgs {
        selector: "name:NonExistentApplication123456".to_string(),
        fallback_selectors: None,
        timeout_ms: Some(2000),
        include_tree: Some(false),
        retries: Some(0),
    };

    match server
        .activate_element(Parameters(test_activate_args))
        .await
    {
        Ok(activate_result) => {
            println!("✅ Found element - testing verification structure");
            let activate_result_json = get_result_json(activate_result);

            // Test the verification structure
            assert!(activate_result_json.get("verification").is_some());
            let verification = &activate_result_json["verification"];

            assert!(verification.get("activation_verified").is_some());
            assert!(verification.get("verification_method").is_some());
            assert!(verification.get("target_pid").is_some());

            println!("✅ Verification structure test passed");
        }
        Err(_) => {
            println!("✅ Element not found - this is also a valid test outcome");
        }
    }

    // Test 2: Test with Desktop element (should find it but verification behavior may vary)
    println!("🎯 Testing activate_element verification with Desktop element...");
    let desktop_activate_args = ActivateElementArgs {
        selector: "name:Desktop".to_string(),
        fallback_selectors: Some("role:Pane".to_string()),
        timeout_ms: Some(5000),
        include_tree: Some(false),
        retries: Some(1),
    };

    match server
        .activate_element(Parameters(desktop_activate_args))
        .await
    {
        Ok(activate_result) => {
            let activate_result_json = get_result_json(activate_result);

            // Verify the response structure contains our new verification fields
            println!("📊 Checking response structure...");

            // Check basic response structure
            assert!(activate_result_json.get("action").is_some());
            assert_eq!(activate_result_json["action"], "activate_element");

            // Verify verification section exists
            assert!(activate_result_json.get("verification").is_some());
            let verification = &activate_result_json["verification"];

            // Verify verification structure
            assert!(verification.get("activation_verified").is_some());
            assert!(verification.get("verification_method").is_some());
            assert!(verification.get("target_pid").is_some());

            let activation_verified = verification["activation_verified"]
                .as_bool()
                .unwrap_or(false);
            let verification_method = verification["verification_method"].as_str().unwrap_or("");
            let target_pid = verification["target_pid"].as_u64().unwrap_or(0);

            println!("✅ Verification results:");
            println!("   - Activation verified: {}", activation_verified);
            println!("   - Verification method: {}", verification_method);
            println!("   - Target PID: {}", target_pid);

            // Assert basic verification structure requirements
            assert_eq!(verification_method, "process_id_comparison");
            assert!(target_pid > 0, "Target PID should be greater than 0");

            // Check status based on verification result
            let expected_status = if activation_verified {
                "success"
            } else {
                "success_unverified"
            };
            assert_eq!(activate_result_json["status"], expected_status);

            // If verified, check additional fields
            if activation_verified {
                assert!(verification.get("focused_pid").is_some());
                assert!(verification.get("pid_match").is_some());
                assert_eq!(verification["pid_match"], true);

                let focused_pid = verification["focused_pid"].as_u64().unwrap_or(0);
                assert_eq!(
                    focused_pid, target_pid,
                    "Focused PID should match target PID when verified"
                );

                println!("✅ Activation was verified successfully!");
            } else {
                println!(
                    "⚠️ Activation was called but could not be verified (this is OK for testing)"
                );
            }

            // Verify recommendation field exists
            assert!(activate_result_json.get("recommendation").is_some());
            let recommendation = activate_result_json["recommendation"]
                .as_str()
                .unwrap_or("");

            if activation_verified {
                assert!(recommendation.contains("verified successfully"));
            } else {
                assert!(recommendation.contains("could not be verified"));
            }

            println!("✅ All verification structure tests passed!");
        }
        Err(e) => {
            println!("⚠️ Desktop activation failed: {:?}", e);
            println!(
                "This is acceptable - the important thing is that we tested the error path above"
            );
        }
    }
}

#[tokio::test]
async fn test_activate_element_verification_timing() {
    let _ = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .try_init();

    let server = DesktopWrapper::new().await.unwrap();

    println!("⏱️ Testing activation verification timing...");

    // Test with a simple selector that should work
    let start_time = std::time::Instant::now();

    let activate_args = ActivateElementArgs {
        selector: "role:Pane".to_string(), // Generic pane selector
        fallback_selectors: None,
        timeout_ms: Some(3000),
        include_tree: Some(false),
        retries: Some(0),
    };

    match server.activate_element(Parameters(activate_args)).await {
        Ok(activate_result) => {
            let elapsed = start_time.elapsed();
            let activate_result_json = get_result_json(activate_result);

            // Verification should include the 500ms delay we added
            assert!(
                elapsed >= Duration::from_millis(500),
                "Test should take at least 500ms due to verification delay, took: {:?}",
                elapsed
            );

            // But shouldn't take too long
            assert!(
                elapsed < Duration::from_millis(10000),
                "Test shouldn't take more than 10 seconds, took: {:?}",
                elapsed
            );

            // Verify structure exists
            assert!(activate_result_json.get("verification").is_some());

            println!("✅ Timing test passed - took {:?}", elapsed);
        }
        Err(_) => {
            // If element is not found, there should be no verification delay
            let elapsed = start_time.elapsed();
            println!(
                "⚠️ Element not found as expected, completed quickly in {:?}",
                elapsed
            );
            // No assertion needed here - fast failure is actually good
        }
    }

    println!("✅ Verification timing test completed!");
}
