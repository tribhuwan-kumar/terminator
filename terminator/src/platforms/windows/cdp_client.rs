use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, warn};
use crate::errors::AutomationError;

/// Lightweight Chrome DevTools Protocol client for connecting to existing browsers
#[derive(Debug, Clone)]
pub struct CdpClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct TabInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    pub websocket_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct CdpRequest {
    id: u32,
    method: String,
    params: Value,
}

#[derive(Debug, Deserialize)]
struct CdpResponse {
    id: u32,
    result: Option<Value>,
    error: Option<Value>,
}

impl CdpClient {
    /// Create a new CDP client 
    pub fn new(debug_port: u16) -> Self {
        Self {
            base_url: format!("http://localhost:{}", debug_port),
            client: reqwest::Client::new(),
        }
    }

    /// Connect to Edge browser (default port 9222)
    pub fn edge() -> Self {
        Self::new(9222)
    }

    /// Connect to Chrome browser (default port 9222)  
    pub fn chrome() -> Self {
        Self::new(9222)
    }

    /// Check if a browser is running with DevTools enabled
    pub async fn is_available(&self) -> bool {
        match self.client.get(&format!("{}/json/version", self.base_url)).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Get list of all open tabs
    pub async fn get_tabs(&self) -> Result<Vec<TabInfo>, AutomationError> {
        let response = self
            .client
            .get(&format!("{}/json", self.base_url))
            .send()
            .await
            .map_err(|e| AutomationError::PlatformError(format!("Failed to get tabs: {}", e)))?;

        let tabs: Vec<TabInfo> = response
            .json()
            .await
            .map_err(|e| AutomationError::PlatformError(format!("Failed to parse tabs: {}", e)))?;

        debug!("Found {} open tabs", tabs.len());
        Ok(tabs)
    }

    /// Find tab by URL pattern
    pub async fn find_tab_by_url(&self, url_pattern: &str) -> Result<Option<TabInfo>, AutomationError> {
        let tabs = self.get_tabs().await?;
        
        for tab in tabs {
            if tab.url.contains(url_pattern) {
                debug!("Found tab with URL pattern '{}': {}", url_pattern, tab.url);
                return Ok(Some(tab));
            }
        }
        
        debug!("No tab found with URL pattern '{}'", url_pattern);
        Ok(None)
    }

    /// Execute JavaScript in a specific tab
    pub async fn execute_script(&self, tab_id: &str, script: &str) -> Result<Value, AutomationError> {
        let mut params = serde_json::Map::new();
        params.insert("expression".to_string(), Value::String(script.to_string()));
        params.insert("returnByValue".to_string(), Value::Bool(true));

        let request = CdpRequest {
            id: 1,
            method: "Runtime.evaluate".to_string(),
            params: Value::Object(params),
        };

        let response = self
            .client
            .post(&format!("{}/json/runtime/evaluate", self.base_url))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AutomationError::PlatformError(format!("CDP request failed: {}", e)))?;

        let cdp_response: CdpResponse = response
            .json()
            .await
            .map_err(|e| AutomationError::PlatformError(format!("Failed to parse CDP response: {}", e)))?;

        if let Some(error) = cdp_response.error {
            return Err(AutomationError::PlatformError(format!("CDP error: {}", error)));
        }

        if let Some(result) = cdp_response.result {
            if let Some(value) = result.get("value") {
                debug!("Script executed successfully: {}", value);
                return Ok(value.clone());
            }
        }

        warn!("Script returned no result");
        Ok(Value::Null)
    }

    /// Get element text by ID
    pub async fn get_element_text_by_id(&self, tab_id: &str, element_id: &str) -> Result<String, AutomationError> {
        let script = format!("document.getElementById('{}')?.textContent || ''", element_id);
        let result = self.execute_script(tab_id, &script).await?;
        
        Ok(result.as_str().unwrap_or("").to_string())
    }

    /// Get element HTML by ID
    pub async fn get_element_html_by_id(&self, tab_id: &str, element_id: &str) -> Result<String, AutomationError> {
        let script = format!("document.getElementById('{}')?.outerHTML || ''", element_id);
        let result = self.execute_script(tab_id, &script).await?;
        
        Ok(result.as_str().unwrap_or("").to_string())
    }

    /// Get page title
    pub async fn get_page_title(&self, tab_id: &str) -> Result<String, AutomationError> {
        let result = self.execute_script(tab_id, "document.title").await?;
        Ok(result.as_str().unwrap_or("").to_string())
    }

    /// Auto-discover and execute script on page containing URL pattern
    pub async fn execute_on_page(&self, url_pattern: &str, script: &str) -> Result<Value, AutomationError> {
        // Check if browser is available
        if !self.is_available().await {
            return Err(AutomationError::PlatformError(
                "Browser not available. Please open Edge/Chrome with --remote-debugging-port=9222".to_string()
            ));
        }

        // Find tab with URL pattern
        let tab = self.find_tab_by_url(url_pattern).await?
            .ok_or_else(|| AutomationError::PlatformError(
                format!("No tab found with URL pattern '{}'", url_pattern)
            ))?;

        // Execute script
        self.execute_script(&tab.id, script).await
    }
}

/// Enable debugging in browser by launching with --remote-debugging-port=9222
pub fn get_browser_launch_instructions() -> &'static str {
    r#"
To enable Chrome DevTools Protocol, launch your browser with:

Edge:
msedge.exe --remote-debugging-port=9222

Chrome:  
chrome.exe --remote-debugging-port=9222

Or add this flag when opening from code:
desktop.open_url_with_args("https://example.com", &["--remote-debugging-port=9222"])
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cdp_availability() {
        let client = CdpClient::edge();
        // This will fail if no browser is running with debugging enabled
        // but won't crash the test
        let available = client.is_available().await;
        println!("Browser with CDP available: {}", available);
    }
}