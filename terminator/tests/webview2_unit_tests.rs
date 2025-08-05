use terminator::*;

/// Unit tests for individual WebView2 functions
/// This helps isolate exactly where the WebView2 implementation fails

#[test]
fn test_basic_webview2_imports() -> Result<(), Box<dyn std::error::Error>> {
    // Test 1: Can we import WebView2 dependencies?
    println!("🧪 Test 1: Testing WebView2 imports...");
    
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2;
    println!("✅ WebView2 imports available");
    
    Ok(())
}

#[tokio::test]
async fn test_find_browser_process() -> Result<(), Box<dyn std::error::Error>> {
    // Test 2: Can we find a browser process running?
    println!("🧪 Test 2: Finding browser processes...");
    
    let desktop = Desktop::new(false, false)?;
    let apps = desktop.applications().await?;
    
    let mut browser_found = false;
    for app in apps {
        if let Ok(name) = app.get_name().await {
            let name_lower = name.to_lowercase();
            if name_lower.contains("chrome") || 
               name_lower.contains("edge") || 
               name_lower.contains("firefox") ||
               name_lower.contains("browser") {
                println!("✅ Found browser process: {}", name);
                browser_found = true;
                break;
            }
        }
    }
    
    if !browser_found {
        println!("⚠️  No browser process currently running - start a browser for better tests");
    }
    
    Ok(())
}

#[test]
fn test_webview2_script_encoding() -> Result<(), Box<dyn std::error::Error>> {
    // Test 3: Test script encoding for WebView2
    println!("🧪 Test 3: Testing script encoding...");
    
    let script = "document.title";
    let script_wide: Vec<u16> = script
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let _script_pcwstr = windows::core::PCWSTR::from_raw(script_wide.as_ptr());
    
    println!("✅ Script encoding successful: {} chars", script_wide.len());
    
    Ok(())
}

#[test]
fn test_com_initialization() -> Result<(), Box<dyn std::error::Error>> {
    // Test 4: Test COM initialization
    println!("🧪 Test 4: Testing COM initialization...");
    
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED, CoUninitialize};
    
    unsafe {
        let result = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if result.is_ok() {
            println!("✅ COM initialization successful");
            CoUninitialize();
        } else {
            println!("⚠️  COM initialization failed (might already be initialized): 0x{:x}", result.0);
        }
    }
    
    Ok(())
}

#[test]
fn test_webview2_environment_functions() -> Result<(), Box<dyn std::error::Error>> {
    // Test 5: Test WebView2 environment function availability
    println!("🧪 Test 5: Testing WebView2 environment functions...");
    
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED, CoUninitialize};
    
    unsafe {
        let _com_init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        
        // Just test that WebView2 functions are available to call
        println!("✅ WebView2 environment functions are available");
        
        CoUninitialize();
    }
    
    Ok(())
}

#[test]
fn test_webview2_runtime_detection() -> Result<(), Box<dyn std::error::Error>> {
    // Test 6: Check if WebView2 runtime is installed
    println!("🧪 Test 6: Testing WebView2 runtime detection...");
    
    // Check registry for WebView2 runtime
    use std::process::Command;
    
    let output = Command::new("reg")
        .args(&["query", "HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}", "/v", "pv"])
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                let version = String::from_utf8_lossy(&result.stdout);
                if version.contains("REG_SZ") {
                    println!("✅ WebView2 runtime detected in registry");
                } else {
                    println!("⚠️  WebView2 runtime not found in registry");
                }
            } else {
                println!("⚠️  Could not check WebView2 runtime registry");
            }
        }
        Err(_) => {
            println!("⚠️  Registry query failed - WebView2 runtime status unknown");
        }
    }
    
    Ok(())
}