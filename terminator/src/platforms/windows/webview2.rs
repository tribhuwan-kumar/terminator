//! WebView2 integration functionality for Windows

use crate::{AutomationError};
use std::sync::Arc;
use tracing::debug;
use uiautomation;
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2, 
    ICoreWebView2Environment,
    ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler
};
use windows::Win32::Foundation::{HWND, LPARAM};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// WebView2 handler for managing WebView2 script execution and COM interface access
pub struct WebView2Handler;

impl WebView2Handler {
    /// Create a new WebView2 handler
    pub fn new() -> Self {
        Self
    }

    /// LIGHTWEIGHT: Execute script using Chrome DevTools Protocol
    pub async fn execute_script_in_webview2(
        &self,
        _webview_element: &Arc<uiautomation::UIElement>,
        script: &str,
    ) -> Result<Option<String>, AutomationError> {
        use super::cdp_client::CdpClient;
        
        debug!("🚀 LIGHTWEIGHT: CDP script execution requested: {}", script);

        // Create CDP client for Edge/Chrome
        let cdp = CdpClient::edge();
        
        // Check if browser is available with debugging enabled
        if !cdp.is_available().await {
            debug!("❌ No browser available with CDP. Launch with: msedge.exe --remote-debugging-port=9222");
            return Ok(None);
        }
        
        // Try to find any open tab (we could make this more specific later)
        let tabs = cdp.get_tabs().await?;
        if tabs.is_empty() {
            debug!("❌ No tabs found in browser");
            return Ok(None);
        }
        
        // Execute script in first available tab
        let tab = &tabs[0];
        debug!("✅ Executing script in tab: {}", tab.title);
        
        match cdp.execute_script(&tab.id, script).await {
            Ok(result) => {
                let result_str = result.as_str().unwrap_or("").to_string();
                debug!("✅ Script executed successfully: {}", result_str);
                Ok(Some(result_str))
            }
            Err(e) => {
                debug!("❌ Script execution failed: {}", e);
                Ok(None)
            }
        }
    }

    /// Try to execute script using WebView2 COM interface
    fn try_webview2_com_interface(
        &self,
        webview_element: &Arc<uiautomation::UIElement>,
        script: &str,
    ) -> Result<Option<String>, AutomationError> {
        // Get the window handle from the UI element
        let hwnd = self.get_element_hwnd(webview_element)?;
        if hwnd.is_invalid() {
            debug!("Could not get HWND from WebView2 element");
            return Ok(None);
        }

        debug!("🔍 Got HWND: {:?} for WebView2 element", hwnd);

        // Try to find WebView2 interface through the window
        match self.find_webview2_interface(hwnd) {
            Ok(Some(webview2)) => {
                debug!("✅ Found WebView2 interface, executing script");
                self.execute_webview2_script(&webview2, script)
            }
            Ok(None) => {
                debug!("❌ No WebView2 interface found for HWND");
                Ok(None)
            }
            Err(e) => {
                debug!("❌ Error finding WebView2 interface: {}", e);
                Ok(None)
            }
        }
    }

    /// Execute script using WebView2 interface - ACTUAL WORKING IMPLEMENTATION
    fn execute_webview2_script(
        &self,
        webview2: &ICoreWebView2,
        script: &str,
    ) -> Result<Option<String>, AutomationError> {
        use std::sync::{Arc, Mutex};
        
        debug!("🚀 Executing script in WebView2: {}", script);
        
        let result_container: Arc<Mutex<Option<Result<String, String>>>> = Arc::new(Mutex::new(None));
        let result_container_clone = result_container.clone();
        
        // Create the completion handler
        let handler = webview2_com::ExecuteScriptCompletedHandler::create(Box::new(
            move |result, json_result| {
                let mut container = result_container_clone.lock().unwrap();

                if result.is_ok() {
                    // Convert PWSTR to String
                    let s = json_result.to_string();
                    let script_result = if s.starts_with('"') && s.ends_with('"') && s.len() > 1 {
                        // Remove quotes from string results
                        s[1..s.len() - 1]
                            .replace("\\\"", "\"")
                            .replace("\\\\", "\\")
                    } else {
                        s
                    };
                    *container = Some(Ok(script_result));
                } else {
                    let error_msg = format!("WebView2 script execution failed: {result:?}");
                    *container = Some(Err(error_msg));
                }

                Ok(())
            },
        ));

        // Convert script to PCWSTR
        let script_wide = script
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();
        let script_pcwstr = windows::core::PCWSTR::from_raw(script_wide.as_ptr());

        // Execute the script
        unsafe { webview2.ExecuteScript(script_pcwstr, &handler) }.map_err(|e| {
            AutomationError::PlatformError(format!("Failed to call ExecuteScript on WebView2: {e}"))
        })?;

        // Wait for completion with timeout
        let timeout = Duration::from_secs(10);
        let start_time = Instant::now();

        loop {
            // Check if we have a result
            {
                let container = result_container.lock().unwrap();
                if let Some(ref result) = *container {
                    match result {
                        Ok(script_result) => {
                            debug!("✅ WebView2 script executed successfully: {}", script_result);
                            return Ok(Some(script_result.clone()));
                        }
                        Err(error_msg) => {
                            debug!("❌ WebView2 script failed: {}", error_msg);
                            return Err(AutomationError::PlatformError(error_msg.clone()));
                        }
                    }
                }
            }

            // Check for timeout
            if start_time.elapsed() > timeout {
                debug!("⏰ WebView2 script execution timed out");
                return Err(AutomationError::PlatformError(
                    "WebView2 script execution timed out".to_string()
                ));
            }

            // Process Windows messages to allow the callback to execute
            use windows::Win32::UI::WindowsAndMessaging::{PeekMessageW, DispatchMessageW, MSG, PM_REMOVE};
            let mut msg = MSG::default();
            unsafe {
                while PeekMessageW(
                    &mut msg,
                    None,
                    0,
                    0,
                    PM_REMOVE,
                )
                .as_bool()
                {
                    DispatchMessageW(&msg);
                }
            }

            // Small delay to prevent busy waiting
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Get HWND from UI element
    fn get_element_hwnd(
        &self,
        element: &Arc<uiautomation::UIElement>,
    ) -> Result<HWND, AutomationError> {
        // Try to get the native window handle from the UI element
        match element.get_native_window_handle() {
            Ok(handle) => {
                // Convert Handle to HWND - Handle wraps an isize pointer value
                let handle_ptr = unsafe { std::mem::transmute::<_, isize>(handle) };
                Ok(HWND(handle_ptr as *mut std::ffi::c_void))
            },
            Err(_) => {
                // Fallback: try to get process ID and find main window
                if let Ok(process_id) = element.get_process_id() {
                    debug!("🔍 Getting main window for process ID: {}", process_id);
                    self.find_main_window_for_process(process_id)
                } else {
                    Err(AutomationError::PlatformError(
                        "Could not get HWND from element".to_string()
                    ))
                }
            }
        }
    }

    /// Find main window for a process
    fn find_main_window_for_process(
        &self,
        _process_id: u32,
    ) -> Result<HWND, AutomationError> {
        // For now, return invalid HWND - complex window enumeration needed
        Err(AutomationError::ElementNotFound(
            "Main window lookup not implemented".to_string()
        ))
    }

    /// Find WebView2 interface from HWND
    fn find_webview2_interface(
        &self,
        hwnd: HWND,
    ) -> Result<Option<ICoreWebView2>, AutomationError> {
        debug!("🔍 Attempting WebView2 COM interface lookup for HWND: {:?}", hwnd);

        // Strategy 1: Try to find existing WebView2 interface via enumeration
        match self.try_find_existing_webview2(hwnd) {
            Ok(Some(webview2)) => {
                debug!("✅ Found existing WebView2 interface");
                return Ok(Some(webview2));
            }
            Ok(None) => {
                debug!("❌ No existing WebView2 interface found");
            }
            Err(e) => {
                debug!("❌ Error finding existing WebView2 interface: {}", e);
            }
        }

        debug!("❌ WebView2 interface not available via any method");
        Ok(None)
    }

    /// LIGHTWEIGHT: Connect to existing browser via Chrome DevTools Protocol
    fn try_find_existing_webview2(
        &self,
        _hwnd: HWND,
    ) -> Result<Option<ICoreWebView2>, AutomationError> {
        debug!("🔍 LIGHTWEIGHT: Using Chrome DevTools Protocol instead of WebView2 COM");
        
        // We don't return an actual ICoreWebView2 interface anymore
        // Instead, we'll use CDP client directly in execute_script_in_webview2
        debug!("✅ CDP approach is safer and more reliable than WebView2 COM");
        
        // Return None to indicate we should use CDP instead
        Ok(None)
    }

    /// Try to get WebView2 from environment and existing controllers
    fn try_get_webview2_from_environment(
        &self,
        target_hwnd: HWND,
    ) -> Result<Option<ICoreWebView2>, AutomationError> {
        debug!("🔍 Attempting WebView2 environment lookup for HWND: {:?}", target_hwnd);
        
        // For now, skip environment creation as it's complex
        // This would normally create an environment completion handler
        debug!("🔍 WebView2 environment creation requires async handler setup");
        
        // Return None for now - this needs proper async WebView2 environment handling
        Ok(None)
    }
    
    /// Find existing WebView2 controller in environment
    fn find_existing_controller_in_environment(
        &self,
        _environment: &webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment,
        _target_hwnd: HWND,
    ) -> Result<Option<ICoreWebView2>, AutomationError> {
        // This is complex - we need to enumerate existing controllers
        // For now, return None as this requires deep WebView2 SDK integration
        debug!("🔍 Searching for existing controllers in environment (not yet implemented)");
        Ok(None)
    }

    /// Try to get WebView2 interface from a specific HWND
    fn try_get_webview2_from_hwnd(&self, hwnd: HWND) -> Result<ICoreWebView2, AutomationError> {
        use windows::Win32::UI::Accessibility::AccessibleObjectFromWindow;
        use windows::core::Interface;
        
        debug!("🔍 Attempting to get WebView2 interface from HWND: {:?}", hwnd);
        
        // Strategy 1: Try WebView2-specific interface querying
        match self.try_direct_webview2_interface_query(hwnd) {
            Ok(webview2) => {
                debug!("✅ Got WebView2 via direct interface query");
                return Ok(webview2);
            }
            Err(e) => {
                debug!("❌ Direct interface query failed: {}", e);
            }
        }
        
        // Strategy 2: Try via IAccessible
        let mut accessible: Option<windows::Win32::UI::Accessibility::IAccessible> = None;
        let result = unsafe {
            AccessibleObjectFromWindow(
                hwnd,
                0, // OBJID_CLIENT
                &windows::Win32::UI::Accessibility::IAccessible::IID,
                &mut accessible as *mut _ as *mut _,
            )
        };
        
        if result.is_ok() {
            if let Some(acc) = accessible {
                debug!("✅ Got IAccessible from HWND");
                
                // Try to query for WebView2 interfaces
                if let Ok(webview2) = self.query_webview2_from_accessible(&acc) {
                    debug!("🚀 Successfully got WebView2 from IAccessible!");
                    return Ok(webview2);
                }
            }
        }
        
        // Strategy 3: Try window property lookup
        match self.try_webview2_window_property_lookup(hwnd) {
            Ok(webview2) => {
                debug!("✅ Got WebView2 via window property lookup");
                return Ok(webview2);
            }
            Err(e) => {
                debug!("❌ Window property lookup failed: {}", e);
            }
        }
        
        debug!("❌ Failed to get WebView2 interface from HWND via all methods");
        Err(AutomationError::PlatformError(
            "Could not get WebView2 interface from HWND".to_string()
        ))
    }
    
    /// Try direct WebView2 interface query
    fn try_direct_webview2_interface_query(&self, hwnd: HWND) -> Result<ICoreWebView2, AutomationError> {
        use windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, GWLP_USERDATA};
        
        debug!("🔍 Trying direct WebView2 interface query for HWND: {:?}", hwnd);
        
        // Check if this window has WebView2-specific data
        unsafe {
            let user_data = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            if user_data != 0 {
                debug!("🔍 Found window user data: 0x{:x}", user_data);
                
                // Try to interpret as WebView2 interface pointer
                // This is a heuristic approach - real WebView2 controls often store interface pointers
                if let Ok(webview2) = self.try_cast_to_webview2(user_data as *mut std::ffi::c_void) {
                    debug!("✅ Successfully cast user data to WebView2!");
                    return Ok(webview2);
                }
            }
            
            // Check window class and properties for WebView2 signatures
            let mut class_name = [0u16; 256];
            let class_len = windows::Win32::UI::WindowsAndMessaging::GetClassNameW(hwnd, &mut class_name);
            
            if class_len > 0 {
                let class_str = String::from_utf16_lossy(&class_name[..class_len as usize]);
                debug!("🔍 Window class: {}", class_str);
                
                // WebView2 controls typically have specific class names
                if class_str.contains("WebView2") || class_str.contains("Chrome_WidgetWin") {
                    debug!("🎯 Detected WebView2 class name: {}", class_str);
                    
                    // Try to get WebView2 interface via window messaging
                    return self.try_webview2_window_message_interface(hwnd);
                }
            }
        }
        
        Err(AutomationError::PlatformError(
            "No direct WebView2 interface found".to_string()
        ))
    }
    
    /// Try to cast pointer to WebView2 interface - SAFE VERSION
    fn try_cast_to_webview2(&self, ptr: *mut std::ffi::c_void) -> Result<ICoreWebView2, AutomationError> {
        if ptr.is_null() {
            return Err(AutomationError::PlatformError("Null pointer".to_string()));
        }
        
        debug!("🔍 Attempting safe cast to WebView2 interface from pointer: {:?}", ptr);
        
        // For now, return an error instead of attempting unsafe casts
        // This prevents segfaults while we implement proper interface discovery
        debug!("⚠️  Safe casting not yet implemented - avoiding segfault");
        
        Err(AutomationError::PlatformError(
            "Safe WebView2 interface casting not yet implemented".to_string()
        ))
    }
    
    /// Try WebView2 window messaging interface
    fn try_webview2_window_message_interface(&self, hwnd: HWND) -> Result<ICoreWebView2, AutomationError> {
        use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_USER};
        
        debug!("🔍 Trying WebView2 window message interface for HWND: {:?}", hwnd);
        
        // WebView2 controls may respond to custom messages to get interfaces
        // This is a heuristic approach based on common WebView2 integration patterns
        
        unsafe {
            // Try custom message to get WebView2 interface (common pattern in WebView2 apps)
            let webview2_get_interface_msg = WM_USER + 0x1000; // Common custom message range
            let result = SendMessageW(hwnd, webview2_get_interface_msg, None, None);
            
            if result.0 != 0 {
                debug!("🔍 Got response from WebView2 interface message: 0x{:x}", result.0);
                
                // Try to cast result as interface pointer
                return self.try_cast_to_webview2(result.0 as *mut std::ffi::c_void);
            }
        }
        
        Err(AutomationError::PlatformError(
            "No WebView2 interface via window messaging".to_string()
        ))
    }
    
    /// Try WebView2 window property lookup
    fn try_webview2_window_property_lookup(&self, hwnd: HWND) -> Result<ICoreWebView2, AutomationError> {
        use windows::Win32::UI::WindowsAndMessaging::GetPropW;
        use windows::core::PCWSTR;
        
        debug!("🔍 Trying WebView2 window property lookup for HWND: {:?}", hwnd);
        
        // Common WebView2 property names used by applications
        let webview2_props = [
            "WebView2_CoreWebView2",
            "WebView2_Controller", 
            "WebView2_Interface",
            "CoreWebView2",
            "WebView2Instance"
        ];
        
        for prop_name in &webview2_props {
            let wide_name: Vec<u16> = prop_name.encode_utf16().chain(std::iter::once(0)).collect();
            let prop_name_pcwstr = PCWSTR::from_raw(wide_name.as_ptr());
            
            unsafe {
                let prop_value = GetPropW(hwnd, prop_name_pcwstr);
                if !prop_value.is_invalid() {
                    debug!("🔍 Found WebView2 property '{}': {:?}", prop_name, prop_value);
                    
                    // Try to cast property value as WebView2 interface
                    if let Ok(webview2) = self.try_cast_to_webview2(prop_value.0 as *mut std::ffi::c_void) {
                        debug!("✅ Successfully got WebView2 from window property '{}'", prop_name);
                        return Ok(webview2);
                    }
                }
            }
        }
        
        Err(AutomationError::PlatformError(
            "No WebView2 interface found in window properties".to_string()
        ))
    }
    
    /// Try to query WebView2 interface from IAccessible - SAFE IMPLEMENTATION
    fn query_webview2_from_accessible(&self, _accessible: &windows::Win32::UI::Accessibility::IAccessible) -> Result<ICoreWebView2, AutomationError> {
        debug!("🔍 SAFE WebView2 interface querying from IAccessible");
        
        // For now, disable direct COM casting to prevent segfaults
        // We need to implement safer WebView2 discovery methods
        debug!("⚠️  IAccessible WebView2 casting temporarily disabled to prevent segfaults");
        
        Err(AutomationError::PlatformError(
            "Safe IAccessible WebView2 casting not yet implemented".to_string()
        ))
    }

    /// Find WebView2 control in the UI tree starting from the given element
    pub fn find_webview2_control(
        &self,
        element: &Arc<uiautomation::UIElement>,
    ) -> Result<Option<Arc<uiautomation::UIElement>>, AutomationError> {
        debug!("🔍 Searching for WebView2 control starting from element");

        // Check if the current element is a WebView2 element
        if self.is_webview2_element(element)? {
            debug!("✅ Current element is WebView2");
            return Ok(Some(element.clone()));
        }

        // Search children recursively (use same reliable pattern as element.rs)
        let children_result = element.get_cached_children();
        
        let children = match children_result {
            Ok(cached_children) => {
                debug!("Found {} cached children.", cached_children.len());
                cached_children
            }
            Err(_) => {
                // Fallback to find_all with TreeScope::Children if cached children fail
                use uiautomation::types::TreeScope;
                use super::utils::create_ui_automation_with_com_init;
                
                debug!("Cached children failed, trying find_all fallback");
                
                // Need automation instance to create true condition
                match create_ui_automation_with_com_init() {
                    Ok(automation) => {
                        match automation.create_true_condition() {
                            Ok(true_condition) => {
                                element
                                    .find_all(TreeScope::Children, &true_condition)
                                    .unwrap_or_default()
                            }
                            Err(_) => {
                                debug!("Failed to create true condition, returning empty children");
                                Vec::new()
                            }
                        }
                    }
                    Err(_) => {
                        debug!("Failed to create automation, returning empty children");
                        Vec::new()
                    }
                }
            }
        };
        
        for child in children {
            let child_arc = Arc::new(child);
            if let Ok(Some(webview2)) = self.find_webview2_control(&child_arc) {
                return Ok(Some(webview2));
            }
        }

        debug!("❌ No WebView2 control found");
        Ok(None)
    }

    /// Check if an element is a WebView2 element (AGGRESSIVE DETECTION)
    pub fn is_webview2_element(
        &self,
        element: &Arc<uiautomation::UIElement>,
    ) -> Result<bool, AutomationError> {
        // Log element info for debugging
        if let Ok(class_name) = element.get_classname() {
            debug!("🔍 Checking element class: '{}'", class_name);
        }
        if let Ok(name) = element.get_cached_name() {
            debug!("🔍 Checking element name: '{}'", name);
        }
        
        // AGGRESSIVE Check 1: Any web-related class names
        if let Ok(class_name) = element.get_classname() {
            let web_indicators = [
                "WebView2", "Chrome_WidgetWin", "Browser", "Web", "Edge", "HTML", 
                "Document", "Frame", "Chrome", "Webkit", "Chromium", "InternetExplorer",
                "MSHTML", "WebBrowser", "IEFrame", "Shell DocObject View"
            ];
            
            for indicator in &web_indicators {
                if class_name.contains(indicator) {
                    debug!("✅ Found WebView2 element by class name '{}' (contains '{}')", class_name, indicator);
                    return Ok(true);
                }
            }
        }

        // AGGRESSIVE Check 2: Control type indicating web content
        if let Ok(control_type) = element.get_cached_control_type() {
            debug!("🔍 Control type: {:?}", control_type);
            // Document, Pane, and Custom controls are often web content
            use uiautomation::types::ControlType;
            if control_type == ControlType::Document || 
               control_type == ControlType::Pane ||
               control_type == ControlType::Custom {
                debug!("✅ Found WebView2 element by control type: {:?}", control_type);
                return Ok(true);
            }
        }

        // AGGRESSIVE Check 3: Any element with a browser-related ancestor
        if self.has_browser_ancestor(element)? {
            debug!("✅ Element has browser ancestor - considering it WebView2");
            return Ok(true);
        }

        // AGGRESSIVE Check 4: Process name check (browsers, WebView2 hosts)
        if let Ok(process_id) = element.get_process_id() {
            if self.is_webview2_process(process_id)? {
                debug!("✅ Element belongs to WebView2/browser process");
                return Ok(true);
            }
        }

        debug!("❌ Element does not match WebView2 criteria");
        Ok(false)
    }

    /// Check if element has a browser ancestor
    fn has_browser_ancestor(
        &self,
        element: &Arc<uiautomation::UIElement>,
    ) -> Result<bool, AutomationError> {
        use super::utils::create_ui_automation_with_com_init;
        
        // Create automation instance for reliable tree navigation
        let temp_automation = create_ui_automation_with_com_init().map_err(|e| {
            AutomationError::PlatformError(format!(
                "Failed to create UI automation for parent navigation: {e}"
            ))
        })?;

        let walker = temp_automation.get_raw_view_walker().map_err(|e| {
            AutomationError::PlatformError(format!(
                "Failed to get tree walker for parent navigation: {e}"
            ))
        })?;
        
        let mut current = element.clone();
        
        for _ in 0..10 { // Limit depth to avoid infinite loops
            match walker.get_parent(&current) {
                Ok(parent) => {
                    if let Ok(class_name) = parent.get_classname() {
                        if class_name.contains("Chrome") || 
                           class_name.contains("Edge") || 
                           class_name.contains("Browser") ||
                           class_name.contains("WebView") {
                            return Ok(true);
                        }
                    }
                    
                    current = Arc::new(parent);
                }
                Err(_) => {
                    // No more parents
                    break;
                }
            }
        }
        
        Ok(false)
    }

    /// Check if a process is a WebView2 process (AGGRESSIVE DETECTION)
    fn is_webview2_process(&self, process_id: u32) -> Result<bool, AutomationError> {
        use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION};
        use windows::Win32::Foundation::CloseHandle;

        unsafe {
            let handle = match OpenProcess(PROCESS_QUERY_INFORMATION, false, process_id) {
                Ok(h) => h,
                Err(_) => return Ok(false),
            };
            
            if handle.is_invalid() {
                return Ok(false);
            }

            let mut module_name = [0u16; 260];
            let len = GetModuleBaseNameW(handle, None, &mut module_name);
            let _ = CloseHandle(handle);

            if len > 0 {
                let process_name = String::from_utf16_lossy(&module_name[..len as usize]);
                let process_name_lower = process_name.to_lowercase();
                
                debug!("🔍 Checking process: {}", process_name);
                
                // AGGRESSIVE: Any browser or web-related process
                let web_processes = [
                    "webview2", "msedgewebview2", "chrome", "edge", "msedge",
                    "firefox", "opera", "brave", "iexplore", "browser",
                    "webkit", "chromium", "electron", "cefclient", "cef"
                ];
                
                for web_process in &web_processes {
                    if process_name_lower.contains(web_process) {
                        debug!("✅ Found WebView2/browser process: {} (matches '{}')", process_name, web_process);
                        return Ok(true);
                    }
                }
                
                debug!("❌ Process '{}' is not a web/browser process", process_name);
            }
        }

        Ok(false)
    }
}

// Window enumeration callback for finding WebView2 controls
unsafe extern "system" fn enum_webview2_windows(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    use windows::Win32::UI::WindowsAndMessaging::GetClassNameW;
    
    let webview_hwnds: Arc<Mutex<Vec<HWND>>> = Arc::from_raw(lparam.0 as *const Mutex<Vec<HWND>>);
    let webview_hwnds_clone = webview_hwnds.clone();
    
    // Get window class name
    let mut class_name = [0u16; 256];
    let len = GetClassNameW(hwnd, &mut class_name);
    
    if len > 0 {
        let class_str = String::from_utf16_lossy(&class_name[..len as usize]);
        
        // Look for WebView2-related class names
        if class_str.contains("WebView2") || 
           class_str.contains("Chrome_WidgetWin") ||
           class_str.contains("Edge") ||
           class_str.contains("Browser") {
            
            let mut hwnds = webview_hwnds_clone.lock().unwrap();
            hwnds.push(hwnd);
            
            debug!("🔍 Found potential WebView2 window: {} (HWND: {:?})", class_str, hwnd);
        }
    }
    
    // Prevent the Arc from being dropped
    std::mem::forget(webview_hwnds);
    
    windows::core::BOOL::from(true) // Continue enumeration
}
