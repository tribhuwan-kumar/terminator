use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use rmcp::handler::server::tool::Parameters;
use terminator_mcp_agent::server::DesktopWrapper;
use terminator_mcp_agent::utils::{init_logging, ExecuteSequenceArgs};
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Execute Terminator MCP workflow sequences from GitHub gists or local JSON files"
)]
struct Args {
    /// Input source - can be a GitHub gist URL, raw gist URL, or local file path
    #[arg(help = "GitHub gist URL, raw gist URL, or path to local JSON file")]
    input: String,

    /// Input type
    #[arg(short, long, value_enum, default_value = "auto")]
    input_type: InputType,

    /// Dry run - parse and validate the workflow without executing
    #[arg(short, long)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Stop on first error (default: true)
    #[arg(long)]
    no_stop_on_error: bool,

    /// Include detailed results (default: true)
    #[arg(long)]
    no_detailed_results: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum InputType {
    /// Auto-detect based on input string
    Auto,
    /// GitHub gist URL (https://gist.github.com/...)
    Gist,
    /// Raw gist URL (https://gist.githubusercontent.com/...)
    Raw,
    /// Local JSON file path
    File,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    init_logging()?;

    if args.verbose {
        std::env::set_var("RUST_LOG", "debug");
        tracing_subscriber::fmt::init();
    }

    info!("Starting Terminator MCP Gist Executor");
    info!("Input: {}", args.input);
    info!("Input type: {:?}", args.input_type);

    // Determine input type and fetch content
    let json_content = match determine_input_type(&args.input, args.input_type) {
        InputType::File => {
            info!("Reading from local file: {}", args.input);
            read_local_file(&args.input).await?
        }
        InputType::Gist => {
            info!("Fetching from GitHub gist: {}", args.input);
            let raw_url = convert_gist_to_raw_url(&args.input)?;
            fetch_remote_content(&raw_url).await?
        }
        InputType::Raw => {
            info!("Fetching from raw URL: {}", args.input);
            fetch_remote_content(&args.input).await?
        }
        InputType::Auto => unreachable!(), // Should be resolved by determine_input_type
    };

    // Parse JSON content
    info!("Parsing JSON content...");
    
    // First try to parse as raw ExecuteSequenceArgs
    let workflow: ExecuteSequenceArgs = match serde_json::from_str(&json_content) {
        Ok(workflow) => workflow,
        Err(_) => {
            // If that fails, try to parse as a tool call wrapper and extract arguments
            info!("Direct parsing failed, trying to extract from tool call wrapper...");
            let tool_call: serde_json::Value = serde_json::from_str(&json_content)
                .context("Failed to parse JSON content")?;
            
            // Check if it's a tool call wrapper with execute_sequence
            if let Some(tool_name) = tool_call.get("tool_name") {
                if tool_name == "execute_sequence" {
                    if let Some(arguments) = tool_call.get("arguments") {
                        info!("Found execute_sequence tool call, extracting arguments...");
                        serde_json::from_value(arguments.clone())
                            .context("Failed to parse arguments as ExecuteSequenceArgs")?
                    } else {
                        return Err(anyhow::anyhow!("Tool call missing 'arguments' field"));
                    }
                } else {
                    return Err(anyhow::anyhow!("Expected execute_sequence tool call, found: {}", tool_name));
                }
            } else {
                return Err(anyhow::anyhow!("JSON does not contain 'steps' field or valid tool call format"));
            }
        }
    };

    info!("Successfully parsed workflow with {} steps", workflow.steps.len());

    if args.dry_run {
        info!("Dry run mode - validating workflow structure");
        validate_workflow(&workflow)?;
        println!("✅ Workflow validation successful!");
        println!("📊 Workflow Summary:");
        println!("   • Steps: {}", workflow.steps.len());
        println!("   • Variables: {}", workflow.variables.as_ref().map_or(0, |v| v.len()));
        println!("   • Selectors: {}", workflow.selectors.as_ref().map_or(0, |v| v.as_object().map_or(0, |o| o.len())));
        println!("   • Stop on error: {}", workflow.stop_on_error.unwrap_or(true));
        return Ok(());
    }

    // Initialize desktop wrapper
    info!("Initializing desktop automation...");
    let desktop_wrapper = DesktopWrapper::new().await?;

    // Override settings based on command line args
    let mut modified_workflow = workflow;
    if args.no_stop_on_error {
        modified_workflow.stop_on_error = Some(false);
    }
    if args.no_detailed_results {
        modified_workflow.include_detailed_results = Some(false);
    }

    // Execute the workflow
    info!("Executing workflow sequence...");
    match desktop_wrapper.execute_sequence(Parameters(modified_workflow)).await {
        Ok(result) => {
            info!("Workflow execution completed successfully");
            println!("✅ Execution completed!");
            
            // Extract and display results
            if args.verbose {
                println!("📋 Detailed Results:");
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("📊 Execution Summary: Check verbose output for details");
            }
        }
        Err(e) => {
            error!("Workflow execution failed: {}", e);
            println!("❌ Execution failed: {e}");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn determine_input_type(input: &str, specified_type: InputType) -> InputType {
    match specified_type {
        InputType::Auto => {
            if input.starts_with("https://gist.github.com/") {
                InputType::Gist
            } else if input.starts_with("https://gist.githubusercontent.com/") {
                InputType::Raw
            } else if input.starts_with("http://") || input.starts_with("https://") {
                InputType::Raw
            } else {
                InputType::File
            }
        }
        other => other,
    }
}

fn convert_gist_to_raw_url(gist_url: &str) -> Result<String> {
    // Convert GitHub gist URL to raw URL
    // Example: https://gist.github.com/username/gist_id -> https://gist.githubusercontent.com/username/gist_id/raw
    
    if !gist_url.starts_with("https://gist.github.com/") {
        return Err(anyhow::anyhow!("Invalid GitHub gist URL format"));
    }
    
    let raw_url = gist_url.replace("https://gist.github.com/", "https://gist.githubusercontent.com/");
    
    // If URL doesn't end with /raw, add it
    if raw_url.ends_with("/raw") {
        Ok(raw_url)
    } else {
        Ok(format!("{}/raw", raw_url))
    }
}

async fn read_local_file(file_path: &str) -> Result<String> {
    tokio::fs::read_to_string(file_path)
        .await
        .with_context(|| format!("Failed to read file: {}", file_path))
}

async fn fetch_remote_content(url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "terminator-mcp-gist-executor/1.0")
        .send()
        .await
        .with_context(|| format!("Failed to fetch URL: {}", url))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "HTTP request failed with status: {} for URL: {}",
            response.status(),
            url
        ));
    }

    response
        .text()
        .await
        .with_context(|| format!("Failed to read response body from URL: {}", url))
}

fn validate_workflow(workflow: &ExecuteSequenceArgs) -> Result<()> {
    if workflow.steps.is_empty() {
        return Err(anyhow::anyhow!("Workflow must contain at least one step"));
    }

    for (i, step) in workflow.steps.iter().enumerate() {
        if step.tool_name.is_none() && step.group_name.is_none() {
            return Err(anyhow::anyhow!(
                "Step {} must have either tool_name or group_name", i
            ));
        }
        if step.tool_name.is_some() && step.group_name.is_some() {
            return Err(anyhow::anyhow!(
                "Step {} cannot have both tool_name and group_name", i
            ));
        }
    }

    // Validate variable schema if present
    if let Some(variables) = &workflow.variables {
        for (name, def) in variables {
            if name.is_empty() {
                return Err(anyhow::anyhow!("Variable name cannot be empty"));
            }
            if def.label.is_empty() {
                return Err(anyhow::anyhow!("Variable '{}' must have a label", name));
            }
        }
    }

    info!("Workflow validation passed");
    Ok(())
} 