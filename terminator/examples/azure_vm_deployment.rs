//! Example: Deploy Azure VM with Terminator MCP Server
//!
//! This example demonstrates how to use the Terminator CLI to:
//! 1. Create an Azure Windows VM
//! 2. Automatically install the MCP server
//! 3. Connect to the VM and use Terminator
//!
//! Prerequisites:
//! - Azure CLI installed and logged in (`az login`)
//! - Azure subscription ID
//! - Terminator CLI built (`cargo build --release`)
//!
//! Run this example:
//! ```bash
//! # Set your Azure subscription ID
//! export AZURE_SUBSCRIPTION_ID="your-subscription-id"
//!
//! # Run the example
//! cargo run --example azure_vm_deployment
//! ```

use std::env;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Terminator Azure VM Deployment Example");
    println!("=========================================\n");

    // Check prerequisites
    check_prerequisites()?;

    // Get subscription ID from environment
    let subscription_id = env::var("AZURE_SUBSCRIPTION_ID")
        .expect("Please set AZURE_SUBSCRIPTION_ID environment variable");

    println!("📋 Using subscription: {}", subscription_id);

    // Step 1: Create Azure VM with MCP server
    println!("\n1️⃣ Creating Azure VM with MCP server...");
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "terminator",
            "--",
            "azure",
            "create",
            "--subscription-id",
            &subscription_id,
            "--vm-size",
            "Standard_D2s_v3",
            "--location",
            "eastus",
            "--save-to",
            "vm-connection.json",
        ])
        .output()?;

    if !output.status.success() {
        eprintln!(
            "Failed to create VM: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Ok(());
    }

    println!("✅ VM created successfully!");

    // Parse connection info
    let connection_json = std::fs::read_to_string("vm-connection.json")?;
    let connection: serde_json::Value = serde_json::from_str(&connection_json)?;

    let public_ip = connection["public_ip"].as_str().unwrap();
    let username = connection["admin_username"].as_str().unwrap();
    let password = connection["admin_password"].as_str().unwrap();
    let resource_group = connection["resource_group"].as_str().unwrap();

    println!("\n📍 VM Details:");
    println!("   Public IP: {}", public_ip);
    println!("   Username: {}", username);
    println!("   Resource Group: {}", resource_group);

    // Step 2: Wait for VM to be ready
    println!("\n2️⃣ Waiting for VM to be ready (this may take a few minutes)...");
    std::thread::sleep(std::time::Duration::from_secs(120));

    // Step 3: Test RDP connection
    println!("\n3️⃣ Testing connectivity...");
    println!("   You can now connect via RDP:");
    println!("   mstsc /v:{}:3389", public_ip);
    println!("   Username: {}", username);
    println!("   Password: {}", password);

    // Step 4: Example Terminator automation script
    println!("\n4️⃣ Example Terminator automation (to run on the VM):");
    println!("```rust");
    println!(
        r#"use terminator::Desktop;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize desktop
    let desktop = Desktop::new()?;
    
    // Open Notepad
    desktop.open_application("notepad").await?;
    
    // Type some text
    desktop.type_text("Hello from Terminator on Azure!").await?;
    
    // Save the file
    desktop.key_combination(&["ctrl", "s"]).await?;
    desktop.type_text("azure-test.txt").await?;
    desktop.key("enter").await?;
    
    Ok(())
}"#
    );
    println!("```");

    // Step 5: Cleanup instructions
    println!("\n5️⃣ When done, clean up resources:");
    println!(
        "   cargo run --bin terminator -- azure delete {} --subscription-id {}",
        resource_group, subscription_id
    );

    println!("\n✨ Example complete! The VM is ready for Terminator automation.");

    Ok(())
}

fn check_prerequisites() -> Result<(), Box<dyn std::error::Error>> {
    // Check Azure CLI
    let output = Command::new("az").arg("--version").output()?;

    if !output.status.success() {
        eprintln!("❌ Azure CLI not found. Please install it first:");
        eprintln!("   https://docs.microsoft.com/en-us/cli/azure/install-azure-cli");
        std::process::exit(1);
    }

    // Check if logged in
    let output = Command::new("az").args(&["account", "show"]).output()?;

    if !output.status.success() {
        eprintln!("❌ Not logged in to Azure. Please run: az login");
        std::process::exit(1);
    }

    println!("✅ Prerequisites checked");
    Ok(())
}
