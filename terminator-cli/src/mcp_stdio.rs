use anyhow::Result;
use rmcp::{
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
    ServiceExt,
};
use std::io::{self, Write};
use tokio::process::Command;
use tracing::info;

pub async fn interactive_chat_stdio(command: Vec<String>) -> Result<()> {
    println!("🤖 Terminator MCP Chat Client (stdio)");
    println!("====================================");
    println!("Starting: {}", command.join(" "));

    // Create command
    let mut cmd = Command::new(&command[0]);
    if command.len() > 1 {
        cmd.args(&command[1..]);
    }

    // Create transport
    let transport = TokioChildProcess::new(cmd)?;

    // Create client (no client info needed for stdio)
    let client = ().serve(transport).await?;

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

    println!("\n💡 Type tool names to execute, 'help' for all tools, 'exit' to quit");
    println!("=====================================\n");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("🔧 Tool: ");
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

        println!("\n⚡ Calling {} ...", tool_name);

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

pub async fn execute_command_stdio(
    command: Vec<String>,
    tool: String,
    args: Option<String>,
) -> Result<()> {
    // Initialize logging
    init_logging();

    info!("Starting MCP server: {}", command.join(" "));

    // Create command
    let mut cmd = Command::new(&command[0]);
    if command.len() > 1 {
        cmd.args(&command[1..]);
    }

    // Create transport
    let transport = TokioChildProcess::new(cmd)?;

    // Connect to server
    let client = ().serve(transport).await?;

    // Parse arguments
    let arguments = if let Some(args_str) = args {
        serde_json::from_str::<serde_json::Value>(&args_str)
            .ok()
            .and_then(|v| v.as_object().cloned())
    } else {
        None
    };

    println!("⚡ Calling {} ...", tool);

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
