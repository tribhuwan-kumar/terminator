use terminator::Desktop;
use terminator::platforms::windows::cdp_client::CdpClient;
use anyhow::Result;

/// LIGHTWEIGHT Browser Automation Demo
/// 
/// This example shows how to use the new lightweight Chrome DevTools Protocol (CDP)
/// approach for browser automation. This is much simpler than WebView2 COM interfaces
/// and works with any existing Edge/Chrome browser.
/// 
/// Requirements:
/// 1. Launch Edge/Chrome with debugging: msedge.exe --remote-debugging-port=9222
/// 2. Open any website in the browser
/// 3. Run this example
/// 
/// Benefits:
/// - ✅ Ultra lightweight (just HTTP requests)
/// - ✅ Works on any Windows machine 
/// - ✅ No external dependencies
/// - ✅ Connects to existing browsers
/// - ✅ No crashes or COM complexity

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 LIGHTWEIGHT Browser Automation Demo");
    println!("=====================================");
    
    // Step 1: Create CDP client
    println!("\n1️⃣ Creating Chrome DevTools Protocol client...");
    let cdp = CdpClient::edge();
    
    // Step 2: Check if browser is available with debugging
    println!("\n2️⃣ Checking if browser is available with debugging...");
    if !cdp.is_available().await {
        println!("❌ No browser available with Chrome DevTools Protocol");
        println!("\n🔧 To enable browser debugging:");
        println!("   msedge.exe --remote-debugging-port=9222");
        println!("   chrome.exe --remote-debugging-port=9222");
        println!("\n📝 Then open any website and run this example again");
        return Ok(());
    }
    
    println!("✅ Browser with CDP is available!");
    
    // Step 3: Get list of open tabs
    println!("\n3️⃣ Getting list of open tabs...");
    let tabs = cdp.get_tabs().await?;
    println!("✅ Found {} open tabs:", tabs.len());
    
    for (i, tab) in tabs.iter().enumerate() {
        println!("   Tab {}: {} - {}", i + 1, tab.title, tab.url);
    }
    
    if tabs.is_empty() {
        println!("❌ No tabs found. Please open a website in the browser.");
        return Ok(());
    }
    
    // Step 4: Demonstrate script execution
    println!("\n4️⃣ Demonstrating JavaScript execution...");
    let tab = &tabs[0];
    println!("🎯 Using tab: {}", tab.title);
    
    // Test 1: Get page title
    println!("\n📄 Getting page title...");
    match cdp.get_page_title(&tab.id).await {
        Ok(title) => println!("✅ Page title: '{}'", title),
        Err(e) => println!("❌ Failed to get title: {}", e),
    }
    
    // Test 2: Get page URL
    println!("\n🌐 Getting page URL...");
    match cdp.execute_script(&tab.id, "document.location.href").await {
        Ok(result) => println!("✅ Page URL: '{}'", result.as_str().unwrap_or("")),
        Err(e) => println!("❌ Failed to get URL: {}", e),
    }
    
    // Test 3: Count elements
    println!("\n🔢 Counting page elements...");
    match cdp.execute_script(&tab.id, "document.querySelectorAll('*').length").await {
        Ok(result) => println!("✅ Total elements on page: {}", result.as_str().unwrap_or("0")),
        Err(e) => println!("❌ Failed to count elements: {}", e),
    }
    
    // Test 4: Find specific element by ID (example)
    println!("\n🎯 Looking for elements by ID...");
    let test_ids = ["my-element", "content", "main", "header", "footer"];
    
    for id in &test_ids {
        let script = format!("document.getElementById('{}')?.textContent || 'Not found'", id);
        match cdp.execute_script(&tab.id, &script).await {
            Ok(result) => {
                let text = result.as_str().unwrap_or("");
                if text != "Not found" && !text.is_empty() {
                    println!("✅ Element '{}': '{}'", id, text.chars().take(50).collect::<String>());
                }
            }
            Err(_) => {} // Ignore errors for this demo
        }
    }
    
    // Test 5: Get page HTML snippet
    println!("\n📝 Getting page HTML snippet...");
    match cdp.execute_script(&tab.id, "document.documentElement.outerHTML.substring(0, 200)").await {
        Ok(result) => {
            let html = result.as_str().unwrap_or("");
            println!("✅ HTML snippet: {}...", html);
        }
        Err(e) => println!("❌ Failed to get HTML: {}", e),
    }
    
    // Step 5: Demonstrate Terminator integration
    println!("\n5️⃣ Demonstrating Terminator + CDP integration...");
    
    // You can also use CDP from within Terminator elements
    println!("💡 You can now use element.execute_script_cdp() in your automation:");
    println!("   let element = desktop.find_element(\"role:Document\")?;");
    println!("   let result = element.execute_script_cdp(\"document.title\").await?;");
    
    println!("\n🎉 Lightweight browser automation demo completed!");
    println!("✅ This approach is much simpler than complex WebView2 COM interfaces");
    println!("✅ Works reliably with existing Edge/Chrome browsers");
    println!("✅ Zero external dependencies beyond HTTP requests");
    
    Ok(())
}

/// Alternative example showing how to use CDP with URL targeting
#[allow(dead_code)]
async fn example_url_targeting() -> Result<()> {
    let cdp = CdpClient::edge();
    
    // Find tab with specific URL pattern and execute script
    match cdp.execute_on_page("example.com", "document.title").await {
        Ok(result) => println!("Title from example.com: {}", result.as_str().unwrap_or("")),
        Err(e) => println!("Failed: {}", e),
    }
    
    Ok(())
}

/// Example showing how to launch browser with debugging enabled
#[allow(dead_code)]
fn example_launch_browser_with_debugging() {
    use std::process::Command;
    
    println!("🚀 Launching Edge with debugging enabled...");
    
    // Launch Edge with debugging port
    match Command::new("msedge.exe")
        .args(&["--remote-debugging-port=9222", "https://example.com"])
        .spawn()
    {
        Ok(_) => println!("✅ Edge launched with debugging enabled"),
        Err(e) => println!("❌ Failed to launch Edge: {}", e),
    }
    
    // Or launch Chrome
    match Command::new("chrome.exe")
        .args(&["--remote-debugging-port=9222", "https://example.com"])
        .spawn()
    {
        Ok(_) => println!("✅ Chrome launched with debugging enabled"),
        Err(e) => println!("❌ Failed to launch Chrome: {}", e),
    }
}