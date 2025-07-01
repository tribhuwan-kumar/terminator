use anyhow::{Context, Result};
use reqwest::Client;
use rmcp::{
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    object,
    transport::StreamableHttpClientTransport,
    ServiceExt,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, Write};
use tracing::info;

#[derive(Debug, Serialize)]
struct McpRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u32,
}

#[derive(Debug, Deserialize)]
struct McpResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<McpError>,
    id: u32,
}

#[derive(Debug, Deserialize)]
struct McpError {
    code: i32,
    message: String,
    data: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct Tool {
    name: String,
    description: Option<String>,
    #[serde(rename = "inputSchema")]
    input_schema: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ToolsResult {
    tools: Vec<Tool>,
}

pub struct McpClient {
    client: Client,
    base_url: String,
    request_id: u32,
}

impl McpClient {
    pub fn new(base_url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url,
            request_id: 0,
        })
    }

    async fn send_request(&mut self, method: &str, params: Value) -> Result<Value> {
        self.request_id += 1;

        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: self.request_id,
        };

        let response = self
            .client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }

        let mcp_response: McpResponse =
            response.json().await.context("Failed to parse response")?;

        if let Some(error) = mcp_response.error {
            anyhow::bail!("MCP error {}: {}", error.code, error.message);
        }

        mcp_response
            .result
            .ok_or_else(|| anyhow::anyhow!("No result in response"))
    }

    pub async fn initialize(&mut self) -> Result<()> {
        println!("🔌 Initializing MCP connection...");

        let params = serde_json::json!({
            "protocolVersion": "0.1.0",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "terminator-cli",
                "version": "1.0.0"
            }
        });

        self.send_request("initialize", params).await?;
        println!("✅ MCP connection initialized");

        Ok(())
    }

    pub async fn list_tools(&mut self) -> Result<Vec<Tool>> {
        let result = self
            .send_request("tools/list", serde_json::json!({}))
            .await?;

        let tools_result: ToolsResult =
            serde_json::from_value(result).context("Failed to parse tools list")?;

        Ok(tools_result.tools)
    }

    pub async fn call_tool(&mut self, tool_name: &str, arguments: Value) -> Result<String> {
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": arguments
        });

        let result = self.send_request("tools/call", params).await?;

        // Extract text content from the result
        if let Some(content) = result.get("content") {
            if let Some(array) = content.as_array() {
                let mut text_parts = Vec::new();
                for item in array {
                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                        text_parts.push(text.to_string());
                    }
                }
                return Ok(text_parts.join("\n"));
            }
        }

        Ok(format!("{}", result))
    }

    pub async fn health_check(&self) -> Result<bool> {
        let health_url = format!("{}/health", self.base_url.trim_end_matches('/'));

        let response = self
            .client
            .get(&health_url)
            .send()
            .await
            .context("Failed to check health")?;

        Ok(response.status().is_success())
    }
}

pub async fn interactive_chat(url: String) -> Result<()> {
    println!("🤖 Terminator MCP Chat Client");
    println!("=============================");
    println!("Connecting to: {}", url);

    // Create transport
    let transport = StreamableHttpClientTransport::from_uri(&url);

    // Create client info
    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "terminator-cli".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    // Connect to server
    let client = client_info.serve(transport).await.inspect_err(|e| {
        tracing::error!("Failed to connect to server: {:?}", e);
    })?;

    // Get server info
    let server_info = client.peer_info();
    println!("✅ Connected to server: {}", server_info.name);
    if let Some(version) = &server_info.version {
        println!("   Version: {}", version);
    }

    // List available tools
    let tools = client.list_all_tools().await?;
    println!("\n📋 Available tools ({}):", tools.len());
    for (i, tool) in tools.iter().enumerate() {
        if i < 10 {
            println!(
                "   🔧 {} - {}",
                tool.name,
                tool.description.as_deref().unwrap_or("No description")
            );
        } else if i == 10 {
            println!("   ... and {} more tools", tools.len() - 10);
            break;
        }
    }

    println!("\n💡 Examples:");
    println!("  - get_desktop_info");
    println!("  - list_applications");
    println!("  - open_application notepad");
    println!("  - type_text 'Hello from Terminator!'");
    println!("  - take_screenshot");
    println!("\nType 'help' to see all tools, 'exit' to quit");
    println!("=====================================\n");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("🔧 Tool (or command): ");
        stdout.flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" {
            println!("👋 Goodbye!");
            break;
        }

        if input == "help" {
            println!("\n📚 All available tools:");
            for tool in &tools {
                println!(
                    "   {} - {}",
                    tool.name,
                    tool.description.as_deref().unwrap_or("No description")
                );
                if let Some(schema) = &tool.input_schema {
                    if let Some(props) = schema.get("properties") {
                        println!("      Parameters: {}", serde_json::to_string(props)?);
                    }
                }
            }
            println!();
            continue;
        }

        // Parse tool call
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let tool_name = parts[0];

        // Build arguments
        let arguments = if parts.len() > 1 {
            // Try to parse as JSON first
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(parts[1]) {
                json.as_object().cloned()
            } else {
                // Otherwise, try to build simple arguments
                match tool_name {
                    "open_application" => Some(object!({ "name": parts[1] })),
                    "type_text" => Some(object!({ "text": parts[1] })),
                    _ => None,
                }
            }
        } else {
            None
        };

        println!(
            "\n⚡ Calling {} with args: {}",
            tool_name,
            arguments
                .as_ref()
                .map(|a| serde_json::to_string(a).unwrap_or_default())
                .unwrap_or_else(|| "{}".to_string())
        );

        match client
            .call_tool(CallToolRequestParam {
                name: tool_name.to_string(),
                arguments,
            })
            .await
        {
            Ok(result) => {
                println!("✅ Result:");
                for content in &result.content {
                    if let Some(text) = content.as_text() {
                        println!("{}", text);
                    } else if let Some(image) = content.as_image() {
                        println!("[Image: {}]", image.url);
                    } else if let Some(resource) = content.as_resource() {
                        println!("[Resource: {}]", resource.uri);
                    }
                }
                println!();
            }
            Err(e) => {
                println!("❌ Error: {}\n", e);
            }
        }
    }

    // Cancel the client connection
    client.cancel().await?;

    Ok(())
}

pub async fn execute_command(url: String, tool: String, args: Option<String>) -> Result<()> {
    // Initialize logging for non-interactive mode
    init_logging();

    // Create transport
    let transport = StreamableHttpClientTransport::from_uri(&url);

    // Create client info
    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "terminator-cli".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    // Connect to server
    info!("Connecting to server: {}", url);
    let client = client_info.serve(transport).await?;

    // Parse arguments
    let arguments = if let Some(args_str) = args {
        serde_json::from_str::<serde_json::Value>(&args_str)
            .ok()
            .and_then(|v| v.as_object().cloned())
    } else {
        None
    };

    println!(
        "⚡ Calling {} with args: {}",
        tool,
        arguments
            .as_ref()
            .map(|a| serde_json::to_string(a).unwrap_or_default())
            .unwrap_or_else(|| "{}".to_string())
    );

    let result = client
        .call_tool(CallToolRequestParam {
            name: tool,
            arguments,
        })
        .await?;

    println!("✅ Result:");
    for content in &result.content {
        if let Some(text) = content.as_text() {
            println!("{}", text);
        } else if let Some(image) = content.as_image() {
            println!("[Image: {}]", image.url);
        } else if let Some(resource) = content.as_resource() {
            println!("[Resource: {}]", resource.uri);
        }
    }

    // Cancel the client connection
    client.cancel().await?;

    Ok(())
}

fn init_logging() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
