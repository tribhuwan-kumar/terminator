use anyhow::Result;
use rmcp::{
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    object,
    transport::{StreamableHttpClientTransport, TokioChildProcess},
    ServiceExt,
};
use std::io::{self, Write};
use tokio::process::Command;
use tracing::info;

pub enum Transport {
    Http(String),
    Stdio(Vec<String>),
}

pub async fn interactive_chat(transport: Transport) -> Result<()> {
    println!("🤖 Terminator MCP Chat Client");
    println!("=============================");

    // Connect based on transport type
    let client = match &transport {
        Transport::Http(url) => {
            println!("Connecting to: {}", url);
            let transport = StreamableHttpClientTransport::from_uri(url);
            let client_info = ClientInfo {
                protocol_version: Default::default(),
                capabilities: ClientCapabilities::default(),
                client_info: Implementation {
                    name: "terminator-cli".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
            };
            client_info.serve(transport).await?
        }
        Transport::Stdio(command) => {
            println!("Starting: {}", command.join(" "));
            let mut cmd = Command::new(&command[0]);
            if command.len() > 1 {
                cmd.args(&command[1..]);
            }
            let transport = TokioChildProcess::new(cmd)?;
            ().serve(transport).await?
        }
    };

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

pub async fn execute_command(
    transport: Transport,
    tool: String,
    args: Option<String>,
) -> Result<()> {
    // Initialize logging for non-interactive mode
    init_logging();

    // Connect based on transport type
    let client = match &transport {
        Transport::Http(url) => {
            info!("Connecting to server: {}", url);
            let transport = StreamableHttpClientTransport::from_uri(url);
            let client_info = ClientInfo {
                protocol_version: Default::default(),
                capabilities: ClientCapabilities::default(),
                client_info: Implementation {
                    name: "terminator-cli".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
            };
            client_info.serve(transport).await?
        }
        Transport::Stdio(command) => {
            info!("Starting MCP server: {}", command.join(" "));
            let mut cmd = Command::new(&command[0]);
            if command.len() > 1 {
                cmd.args(&command[1..]);
            }
            let transport = TokioChildProcess::new(cmd)?;
            ().serve(transport).await?
        }
    };

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
