use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureVmConfig {
    pub subscription_id: String,
    pub resource_group: String,
    pub location: String,
    pub vm_name: String,
    pub vm_size: String,
    pub admin_username: String,
    pub admin_password: String,
    pub enable_rdp: bool,
    pub enable_winrm: bool,
    pub deploy_mcp: bool,
}

impl Default for AzureVmConfig {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let random_suffix: u32 = rng.gen_range(1000..9999);

        Self {
            subscription_id: String::new(),
            resource_group: format!("terminator-rg-{}", random_suffix),
            location: "eastus".to_string(),
            vm_name: format!("terminator-vm-{}", random_suffix),
            vm_size: "Standard_D2s_v3".to_string(),
            admin_username: "terminatoradmin".to_string(),
            admin_password: generate_secure_password(),
            enable_rdp: true,
            enable_winrm: true,
            deploy_mcp: true,
        }
    }
}

pub struct AzureVmManager {
    config: AzureVmConfig,
}

impl AzureVmManager {
    pub fn new(config: AzureVmConfig) -> Result<Self> {
        // Check if Azure CLI is installed
        let output = Command::new("az")
            .arg("--version")
            .output()
            .context("Azure CLI not found. Please install it first: https://docs.microsoft.com/en-us/cli/azure/install-azure-cli")?;

        if !output.status.success() {
            anyhow::bail!("Azure CLI is not properly installed");
        }

        // Check if logged in
        let output = Command::new("az")
            .args(&["account", "show"])
            .output()
            .context("Failed to check Azure login status")?;

        if !output.status.success() {
            anyhow::bail!("Not logged in to Azure. Please run 'az login' first");
        }

        Ok(Self { config })
    }

    pub async fn create_vm(&self) -> Result<VmDeploymentResult> {
        println!("🚀 Starting Azure VM deployment...");

        // Set subscription
        self.set_subscription()?;

        // Create resource group
        self.create_resource_group()?;

        // Create VM with all resources
        let public_ip = self.create_vm_with_resources()?;

        // Open additional ports after VM creation
        if self.config.enable_winrm {
            self.open_winrm_ports()?;
        }

        if self.config.deploy_mcp {
            self.open_mcp_port()?;
        }

        Ok(VmDeploymentResult {
            vm_name: self.config.vm_name.clone(),
            resource_group: self.config.resource_group.clone(),
            public_ip,
            admin_username: self.config.admin_username.clone(),
            admin_password: self.config.admin_password.clone(),
            rdp_enabled: self.config.enable_rdp,
            winrm_enabled: self.config.enable_winrm,
            mcp_deployed: self.config.deploy_mcp,
            mcp_url: if self.config.deploy_mcp {
                Some(format!("http://{}:3000", public_ip))
            } else {
                None
            },
        })
    }

    fn set_subscription(&self) -> Result<()> {
        println!("📋 Setting subscription...");

        let output = Command::new("az")
            .args(&[
                "account",
                "set",
                "--subscription",
                &self.config.subscription_id,
            ])
            .output()
            .context("Failed to set subscription")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to set subscription: {}", stderr);
        }

        Ok(())
    }

    fn create_resource_group(&self) -> Result<()> {
        println!("📁 Creating resource group: {}", self.config.resource_group);

        let output = Command::new("az")
            .args(&[
                "group",
                "create",
                "--name",
                &self.config.resource_group,
                "--location",
                &self.config.location,
            ])
            .output()
            .context("Failed to create resource group")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create resource group: {}", stderr);
        }

        println!("✅ Resource group created");
        Ok(())
    }

    fn create_vm_with_resources(&self) -> Result<String> {
        println!("🖥️  Creating VM: {}", self.config.vm_name);

        // Prepare custom data script if MCP deployment is requested
        let custom_data = if self.config.deploy_mcp {
            let script = include_str!("scripts/install_mcp.ps1");
            let encoded = STANDARD.encode(script);
            Some(encoded)
        } else {
            None
        };

        // Build VM create command
        let mut args = vec![
            "vm",
            "create",
            "--resource-group",
            &self.config.resource_group,
            "--name",
            &self.config.vm_name,
            "--image",
            "Win2022Datacenter",
            "--size",
            &self.config.vm_size,
            "--admin-username",
            &self.config.admin_username,
            "--admin-password",
            &self.config.admin_password,
            "--location",
            &self.config.location,
            "--public-ip-address-allocation",
            "static",
            "--public-ip-sku",
            "Standard",
        ];

        // Add custom data if provided
        let custom_data_arg;
        if let Some(ref data) = custom_data {
            custom_data_arg = format!("@data:text/plain;base64,{}", data);
            args.push("--custom-data");
            args.push(&custom_data_arg);
        }

        // Add NSG rules
        let mut nsg_rules = Vec::new();
        if self.config.enable_rdp {
            nsg_rules.push("RDP");
        }
        if !nsg_rules.is_empty() {
            args.push("--nsg-rule");
            for rule in &nsg_rules {
                args.push(rule);
            }
        }

        // Create the VM
        let output = Command::new("az")
            .args(&args)
            .output()
            .context("Failed to create VM")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create VM: {}", stderr);
        }

        println!("✅ VM created successfully");

        // Get public IP
        let public_ip = self.get_public_ip()?;
        println!("📍 Public IP: {}", public_ip);

        Ok(public_ip)
    }

    fn open_winrm_ports(&self) -> Result<()> {
        println!("🔓 Opening WinRM ports...");

        // Open WinRM HTTP port (5985)
        let output = Command::new("az")
            .args(&[
                "vm",
                "open-port",
                "--resource-group",
                &self.config.resource_group,
                "--name",
                &self.config.vm_name,
                "--port",
                "5985",
                "--priority",
                "1001",
            ])
            .output()
            .context("Failed to open WinRM HTTP port")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("⚠️  Warning: Failed to open WinRM HTTP port: {}", stderr);
        }

        // Open WinRM HTTPS port (5986)
        let output = Command::new("az")
            .args(&[
                "vm",
                "open-port",
                "--resource-group",
                &self.config.resource_group,
                "--name",
                &self.config.vm_name,
                "--port",
                "5986",
                "--priority",
                "1002",
            ])
            .output()
            .context("Failed to open WinRM HTTPS port")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("⚠️  Warning: Failed to open WinRM HTTPS port: {}", stderr);
        }

        Ok(())
    }

    fn open_mcp_port(&self) -> Result<()> {
        println!("🔓 Opening MCP HTTP port (3000)...");

        let output = Command::new("az")
            .args(&[
                "vm",
                "open-port",
                "--resource-group",
                &self.config.resource_group,
                "--name",
                &self.config.vm_name,
                "--port",
                "3000",
                "--priority",
                "1003",
            ])
            .output()
            .context("Failed to open MCP HTTP port")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("⚠️  Warning: Failed to open MCP HTTP port: {}", stderr);
        } else {
            println!("✅ MCP HTTP port 3000 opened");
        }

        Ok(())
    }

    fn get_public_ip(&self) -> Result<String> {
        let output = Command::new("az")
            .args(&[
                "vm",
                "show",
                "--resource-group",
                &self.config.resource_group,
                "--name",
                &self.config.vm_name,
                "--show-details",
                "--query",
                "publicIps",
                "--output",
                "tsv",
            ])
            .output()
            .context("Failed to get VM public IP")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to get public IP: {}", stderr);
        }

        let public_ip = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if public_ip.is_empty() {
            anyhow::bail!("No public IP assigned to VM");
        }

        Ok(public_ip)
    }

    pub async fn delete_resource_group(&self) -> Result<()> {
        println!(
            "🗑️  Deleting resource group: {}",
            self.config.resource_group
        );

        let output = Command::new("az")
            .args(&[
                "group",
                "delete",
                "--name",
                &self.config.resource_group,
                "--yes",
                "--no-wait",
            ])
            .output()
            .context("Failed to delete resource group")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to delete resource group: {}", stderr);
        }

        println!("✅ Resource group deletion initiated");
        println!("   (This may take several minutes to complete in the background)");
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmDeploymentResult {
    pub vm_name: String,
    pub resource_group: String,
    pub public_ip: String,
    pub admin_username: String,
    pub admin_password: String,
    pub rdp_enabled: bool,
    pub winrm_enabled: bool,
    pub mcp_deployed: bool,
    pub mcp_url: Option<String>,
}

impl VmDeploymentResult {
    pub fn print_connection_info(&self) {
        println!("\n🎉 Azure VM Deployment Complete!");
        println!("================================");
        println!("VM Name: {}", self.vm_name);
        println!("Resource Group: {}", self.resource_group);
        println!("Public IP: {}", self.public_ip);
        println!("Admin Username: {}", self.admin_username);
        println!("Admin Password: {}", self.admin_password);

        if self.rdp_enabled {
            println!("\n📱 RDP Connection:");
            println!("  Windows: mstsc /v:{}:3389", self.public_ip);
            println!("  macOS: Use Microsoft Remote Desktop from App Store");
            println!(
                "  Linux: xfreerdp /v:{} /u:{} /p:'{}'",
                self.public_ip, self.admin_username, self.admin_password
            );
        }

        if self.winrm_enabled {
            println!("\n🔧 WinRM Connection:");
            println!(
                "  Enter-PSSession -ComputerName {} -Credential (Get-Credential {})",
                self.public_ip, self.admin_username
            );
        }

        if let Some(ref mcp_url) = self.mcp_url {
            println!("\n🤖 MCP Server:");
            println!("  HTTP Endpoint: {}", mcp_url);
            println!("  Health Check: {}/health", mcp_url);
            println!("  Installation log: C:\\TerminatorMCP\\install.log");
            println!("\n  Test with: terminator mcp chat --url {}", mcp_url);
        }

        println!("\n⚠️  Security Notes:");
        println!("  1. The VM is accessible from the internet.");
        println!("  2. Consider restricting NSG rules to your IP address.");
        println!("  3. Use strong passwords and enable MFA where possible.");
    }

    pub fn save_to_file(&self, filename: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(filename, json)?;
        println!("\n💾 Connection info saved to: {}", filename);
        Ok(())
    }
}

fn generate_secure_password() -> String {
    use rand::distributions::Alphanumeric;

    let mut rng = rand::thread_rng();

    // Generate base password
    let base: String = (0..12)
        .map(|_| rng.sample(Alphanumeric))
        .map(char::from)
        .collect();

    // Add special characters and numbers to meet Azure requirements
    let special_chars = ['!', '@', '#', '$', '%', '^', '&', '*'];
    let special = special_chars[rng.gen_range(0..special_chars.len())];
    let number = rng.gen_range(100..999);

    // Ensure we have uppercase and lowercase
    let mut password = format!("{}{}Az{}", base, special, number);

    // Shuffle the password
    use rand::seq::SliceRandom;
    let mut chars: Vec<char> = password.chars().collect();
    chars.shuffle(&mut rng);

    chars.into_iter().collect()
}
