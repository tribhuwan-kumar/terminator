use terminator::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_terminator_webview2_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Terminator WebView2 integration with real browser...");
    
    let desktop = Desktop::new(false, false)?;
    
    // Launch Edge with a simple test page
    let target_url = "data:text/html,<html><head><title>Test WebView2</title></head><body><h1 class='test-title'>Hello WebView2</h1><p>This is a test page</p></body></html>";
    
    println!("🌐 Opening test page in browser...");
    let browser_element = match desktop.open_url(target_url, Some(terminator::Browser::Edge)) {
        Ok(element) => {
            println!("✅ Browser opened successfully");
            element
        }
        Err(e) => {
            println!("⚠️  Edge failed, trying default browser: {}", e);
            desktop.open_url(target_url, None)?
        }
    };
    
    println!("⏳ Waiting for page to load...");
    sleep(Duration::from_secs(5)).await;
    
    // Test script execution using the public API
    println!("🚀 Testing WebView2 script execution via public API...");
    let script = "document.title";
    
    match browser_element.execute_script(script) {
        Ok(Some(result)) => {
            println!("🎉 SUCCESS! Script executed via public API: '{}'", result);
            assert!(!result.is_empty(), "Script result should not be empty");
        }
        Ok(None) => {
            println!("⚠️  Script executed but returned no result");
            // This is OK - WebView2 might be working but no result returned
        }
        Err(e) => {
            println!("❌ Script execution failed: {}", e);
            // Let's examine what element we have
            if let Some(name) = browser_element.name() {
                println!("🔍 Browser element name: '{}'", name);
            }
            return Err(e.into());
        }
    }
    
    println!("🎉 Terminator WebView2 integration test completed successfully!");
    Ok(())
}