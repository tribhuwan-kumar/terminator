use std::sync::mpsc;
use webview2_com::Microsoft::Web::WebView2::Win32::CreateCoreWebView2Environment;
use webview2_com::CreateCoreWebView2EnvironmentCompletedHandler;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED, CoUninitialize};

#[test]
fn test_webview2_real_environment_creation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing REAL WebView2 environment creation...");
    
    unsafe {
        let _com_init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        
        // Try the exact pattern from the working example
        let (tx, rx) = mpsc::channel();

        match CreateCoreWebView2EnvironmentCompletedHandler::wait_for_async_operation(
            Box::new(|environmentcreatedhandler| unsafe {
                CreateCoreWebView2Environment(&environmentcreatedhandler)
                    .map_err(webview2_com::Error::WindowsError)
            }),
            Box::new(move |error_code, environment| {
                if let Err(e) = error_code {
                    println!("❌ Environment creation failed: {:?}", e);
                    tx.send(Err(e)).expect("send over mpsc channel");
                } else {
                    tx.send(Ok(environment.ok_or_else(|| {
                        windows::core::Error::from(windows::Win32::Foundation::E_POINTER)
                    }))).expect("send over mpsc channel");
                }
                Ok(())
            }),
        ) {
            Ok(_) => {
                match rx.recv() {
                    Ok(Ok(env)) => {
                        println!("✅ WebView2 environment created successfully!");
                        println!("🎉 Real WebView2 SDK integration is WORKING!");
                    }
                    Ok(Err(e)) => {
                        println!("❌ Environment creation error: {:?}", e);
                        return Err(e.into());
                    }
                    Err(_) => {
                        println!("❌ Channel receive error");
                        return Err("Channel error".into());
                    }
                }
            }
            Err(e) => {
                println!("❌ Environment operation failed: {:?}", e);
                return Err(e.into());
            }
        }
        
        CoUninitialize();
    }
    
    Ok(())
}

#[test]
fn test_webview2_runtime_availability() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing WebView2 runtime availability...");
    
    // Check if WebView2 runtime is available
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
                    println!("📋 Runtime info: {}", version);
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