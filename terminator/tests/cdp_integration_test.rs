use std::process::Command;
use terminator::platforms::windows::cdp_client::CdpClient;
use tokio;

#[tokio::test] 
async fn test_cdp_browser_connection() {
    println!("🧪 Testing LIGHTWEIGHT Chrome DevTools Protocol connection");
    
    // Create CDP client
    let cdp = CdpClient::edge();
    
    // Check if browser is available
    let available = cdp.is_available().await;
    println!("📊 Browser with CDP available: {}", available);
    
    if !available {
        println!("⚠️  To test this, launch Edge with debugging:");
        println!("   msedge.exe --remote-debugging-port=9222");
        println!("   Then open: https://example.com");
        return;
    }
    
    // Get list of tabs
    match cdp.get_tabs().await {
        Ok(tabs) => {
            println!("✅ Found {} open tabs", tabs.len());
            for (i, tab) in tabs.iter().enumerate() {
                println!("  Tab {}: {} - {}", i + 1, tab.title, tab.url);
            }
            
            // If we have tabs, try to execute a script
            if !tabs.is_empty() {
                let tab = &tabs[0];
                println!("🚀 Testing script execution in: {}", tab.title);
                
                // Test getting page title
                match cdp.get_page_title(&tab.id).await {
                    Ok(title) => println!("✅ Page title: {}", title),
                    Err(e) => println!("❌ Failed to get title: {}", e),
                }
                
                // Test custom script
                match cdp.execute_script(&tab.id, "document.location.href").await {
                    Ok(result) => println!("✅ Page URL via script: {}", result),
                    Err(e) => println!("❌ Failed to execute script: {}", e),
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to get tabs: {}", e);
        }
    }
}

#[tokio::test]
async fn test_cdp_element_extraction() {
    println!("🧪 Testing LIGHTWEIGHT element extraction");
    
    let cdp = CdpClient::edge();
    
    if !cdp.is_available().await {
        println!("⚠️  Browser not available - launch with debugging enabled");
        return;
    }
    
    // Try to find a tab and extract element by ID
    if let Ok(Some(tab)) = cdp.find_tab_by_url("example.com").await {
        println!("✅ Found example.com tab: {}", tab.title);
        
        // Test element extraction
        match cdp.execute_script(&tab.id, "document.querySelector('h1')?.textContent || 'No h1 found'").await {
            Ok(result) => println!("✅ H1 content: {}", result),
            Err(e) => println!("❌ Failed to get H1: {}", e),
        }
        
        // Test getting element by ID (if exists)
        match cdp.execute_script(&tab.id, "document.getElementById('my-element')?.innerHTML || 'Element not found'").await {
            Ok(result) => println!("✅ Element by ID: {}", result),
            Err(e) => println!("❌ Failed to get element: {}", e),
        }
    } else {
        println!("⚠️  No example.com tab found - open https://example.com to test");
    }
}

// Helper test to demonstrate browser launching
#[test]
fn test_launch_edge_with_debugging() {
    println!("🚀 How to launch Edge with Chrome DevTools Protocol:");
    println!("   1. Open Command Prompt");
    println!("   2. Run: msedge.exe --remote-debugging-port=9222");
    println!("   3. Navigate to any website");
    println!("   4. Run this test again");
    println!();
    println!("🔧 Alternative - Launch programmatically:");
    
    // Example of launching Edge with debugging (commented out to avoid actually launching)
    /*
    let output = Command::new("msedge.exe")
        .args(&["--remote-debugging-port=9222", "https://example.com"])
        .spawn();
    
    match output {
        Ok(_) => println!("✅ Edge launched with debugging"),
        Err(e) => println!("❌ Failed to launch Edge: {}", e),
    }
    */
    
    println!("📝 Note: This is just a demonstration - no browser actually launched");
}