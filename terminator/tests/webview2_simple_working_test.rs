// This test follows the EXACT pattern from the working WebView2 example provided by the user

use std::sync::mpsc;
use webview2_com::Microsoft::Web::WebView2::Win32::CreateCoreWebView2Environment;
use webview2_com::{CreateCoreWebView2EnvironmentCompletedHandler, CreateCoreWebView2ControllerCompletedHandler, ExecuteScriptCompletedHandler};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED, CoUninitialize};
use windows::Win32::Foundation::{HWND, E_POINTER};

#[test] 
fn test_webview2_exact_working_pattern() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing WebView2 using EXACT working pattern from provided example...");
    
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        
        // Step 1: Create WebView2 Environment (EXACT pattern from working example)
        let environment = {
            let (tx, rx) = mpsc::channel();

            CreateCoreWebView2EnvironmentCompletedHandler::wait_for_async_operation(
                Box::new(|environmentcreatedhandler| unsafe {
                    CreateCoreWebView2Environment(&environmentcreatedhandler)
                        .map_err(webview2_com::Error::WindowsError)
                }),
                Box::new(move |error_code, environment| {
                    error_code?;
                    tx.send(environment.ok_or_else(|| windows::core::Error::from(E_POINTER)))
                        .expect("send over mpsc channel");
                    Ok(())
                }),
            )?;

            rx.recv()
                .map_err(|_| webview2_com::Error::SendError)?
        }?;
        
        println!("✅ WebView2 environment created using exact working pattern!");
        
        // Step 2: Create a simple test window for the controller
        let hwnd = create_simple_test_window();
        println!("✅ Test window created: {:?}", hwnd);
        
        // Step 3: Create WebView2 Controller (EXACT pattern from working example)
        let controller = {
            let (tx, rx) = mpsc::channel();

            CreateCoreWebView2ControllerCompletedHandler::wait_for_async_operation(
                Box::new(move |handler| unsafe {
                    environment
                        .CreateCoreWebView2Controller(hwnd, &handler)
                        .map_err(webview2_com::Error::WindowsError)
                }),
                Box::new(move |error_code, controller| {
                    error_code?;
                    tx.send(controller.ok_or_else(|| windows::core::Error::from(E_POINTER)))
                        .expect("send over mpsc channel");
                    Ok(())
                }),
            )?;

            rx.recv()
                .map_err(|_| webview2_com::Error::SendError)?
        }?;
        
        println!("✅ WebView2 controller created using exact working pattern!");
        
        // Step 4: Get CoreWebView2 interface (EXACT pattern from working example)
        let webview2 = unsafe { controller.CoreWebView2()? };
        println!("✅ CoreWebView2 interface obtained!");
        
        // Step 5: Execute script (EXACT pattern from working example)
        ExecuteScriptCompletedHandler::wait_for_async_operation(
            Box::new({
                let webview2 = webview2.clone();
                move |handler| unsafe {
                    let script = "document.title || 'Test WebView2'";
                    let script_wide: Vec<u16> = script
                        .encode_utf16()
                        .chain(std::iter::once(0))
                        .collect();
                    let script_pcwstr = windows::core::PCWSTR::from_raw(script_wide.as_ptr());
                    
                    webview2
                        .ExecuteScript(script_pcwstr, &handler)
                        .map_err(webview2_com::Error::WindowsError)
                }
            }),
            Box::new(|error_code, result| {
                error_code?;
                let result_str = unsafe { result.to_string() };
                println!("🎉 Script execution SUCCESS! Result: '{}'", result_str);
                Ok(())
            }),
        )?;
        
        println!("🎉 COMPLETE WebView2 integration using exact working pattern SUCCESSFUL!");
        
        CoUninitialize();
    }
    
    Ok(())
}

// Create a minimal test window
unsafe fn create_simple_test_window() -> HWND {
    use windows::Win32::UI::WindowsAndMessaging::{CreateWindowExW, WS_OVERLAPPED, CW_USEDEFAULT};
    use windows::core::w;
    
    CreateWindowExW(
        Default::default(),
        w!("STATIC"), // Use built-in STATIC class
        w!("Test WebView2"),
        WS_OVERLAPPED,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        800,
        600,
        None,
        None,
        None,
        None,
    ).unwrap_or_default()
}