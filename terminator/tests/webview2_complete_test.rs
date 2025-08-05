use std::sync::mpsc;
use webview2_com::Microsoft::Web::WebView2::Win32::CreateCoreWebView2Environment;
use webview2_com::{CreateCoreWebView2EnvironmentCompletedHandler, CreateCoreWebView2ControllerCompletedHandler, ExecuteScriptCompletedHandler};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED, CoUninitialize};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{CreateWindowExW, WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, MSG, PeekMessageW, DispatchMessageW, PM_REMOVE};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::core::PCWSTR;

#[test]
fn test_webview2_complete_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing COMPLETE WebView2 integration (Environment + Controller + Script)...");
    
    unsafe {
        let _com_init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        
        // Step 1: Create a test window (needed for WebView2 controller)
        let hwnd = create_test_window();
        println!("✅ Test window created: {:?}", hwnd);
        
        // Step 2: Create WebView2 Environment
        let environment = {
            let (tx, rx) = mpsc::channel();

            CreateCoreWebView2EnvironmentCompletedHandler::wait_for_async_operation(
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
            )?;

            let env = rx.recv()??; // Double ? to unwrap Result<Result<T, E>, RecvError>
            println!("✅ WebView2 environment created successfully!");
            env
        };
        
        // Step 3: Create WebView2 Controller
        let controller = {
            let (tx, rx) = mpsc::channel();

            let env_clone = environment.clone();
            CreateCoreWebView2ControllerCompletedHandler::wait_for_async_operation(
                Box::new(move |handler| unsafe {
                    env_clone
                        .CreateCoreWebView2Controller(hwnd, &handler)
                        .map_err(webview2_com::Error::WindowsError)
                }),
                Box::new(move |error_code, controller| {
                    if let Err(e) = error_code {
                        println!("❌ Controller creation failed: {:?}", e);
                        tx.send(Err(e)).expect("send over mpsc channel");
                    } else {
                        tx.send(Ok(controller.ok_or_else(|| {
                            windows::core::Error::from(windows::Win32::Foundation::E_POINTER)
                        }))).expect("send over mpsc channel");
                    }
                    Ok(())
                }),
            )?;

            let ctrl = rx.recv()??; // Double ? to unwrap Result<Result<T, E>, RecvError>
            println!("✅ WebView2 controller created successfully!");
            ctrl
        };
        
        // Step 4: Get CoreWebView2 interface
        let webview2 = controller.CoreWebView2()?;
        println!("✅ CoreWebView2 interface obtained!");
        
        // Step 5: Execute a test script
        let script = "document.title || 'No Title'";
        let (tx, rx) = mpsc::channel();
        
        ExecuteScriptCompletedHandler::wait_for_async_operation(
            Box::new({
                let webview2 = webview2.clone();
                let script = String::from(script);
                move |handler| unsafe {
                    let script_wide: Vec<u16> = script
                        .encode_utf16()
                        .chain(std::iter::once(0))
                        .collect();
                    let script_pcwstr = PCWSTR::from_raw(script_wide.as_ptr());
                    
                    webview2
                        .ExecuteScript(script_pcwstr, &handler)
                        .map_err(webview2_com::Error::WindowsError)
                }
            }),
            Box::new(move |error_code, result| {
                if let Err(e) = error_code {
                    println!("❌ Script execution failed: {:?}", e);
                    tx.send(Err(format!("Script execution error: {:?}", e)))
                        .expect("send over mpsc channel");
                } else {
                    // Convert PWSTR result to String  
                    let result_string = if let Some(result_pwstr) = result.as_ref() {
                        let result_str = unsafe { result_pwstr.to_string() };
                        
                        // Remove JSON quotes if present
                        if result_str.starts_with('"') && result_str.ends_with('"') && result_str.len() > 1 {
                            result_str[1..result_str.len() - 1]
                                .replace("\\\"", "\"")
                                .replace("\\\\", "\\")
                        } else {
                            result_str
                        }
                    } else {
                        String::new()
                    };
                    
                    println!("✅ Script execution completed: {}", result_string);
                    tx.send(Ok(result_string)).expect("send over mpsc channel");
                }
                Ok(())
            }),
        )?;
        
        match rx.recv()? {
            Ok(result) => {
                println!("🎉 COMPLETE WebView2 integration SUCCESS!");
                println!("📋 Script result: '{}'", result);
            }
            Err(error_msg) => {
                return Err(error_msg.into());
            }
        }
        
        CoUninitialize();
    }
    
    Ok(())
}

// Default window procedure
unsafe extern "system" fn default_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    windows::Win32::UI::WindowsAndMessaging::DefWindowProcW(hwnd, msg, wparam, lparam)
}

// Helper function to create a test window
unsafe fn create_test_window() -> HWND {
    use windows::core::w;
    use windows::Win32::UI::WindowsAndMessaging::{RegisterClassW, WNDCLASSW};
    use windows::Win32::Foundation::HINSTANCE;
    
    let window_class = WNDCLASSW {
        lpfnWndProc: Some(default_window_proc),
        lpszClassName: w!("TestWebView2Window"),
        ..Default::default()
    };

    RegisterClassW(&window_class);

    CreateWindowExW(
        Default::default(),
        w!("TestWebView2Window"),
        w!("Test WebView2 Window"),
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        800,
        600,
        None,
        None,
        GetModuleHandleW(None).ok().map(|h| HINSTANCE(h.0)),
        None,
    ).unwrap_or_default()
}