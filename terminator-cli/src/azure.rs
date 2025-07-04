use anyhow::{Context, Result};
use rand::Rng;
use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct AzureVmConfig {
    pub subscription_id: String,
    pub resource_group: String,
    pub location: String,
    pub vm_name: String,
    pub vm_size: String,
    pub admin_username: String,
    #[serde(skip_serializing)] // Don't include password in general serialization
    pub admin_password: String,
}

fn generate_secure_password() -> String {
    format!("TerminatorP@ssw0rd{}", rand::random::<u32>())
}

impl Default for AzureVmConfig {
    fn default() -> Self {
        let random_suffix: u32 = rand::thread_rng().gen_range(1000..9999);
        Self {
            subscription_id: String::new(),
            resource_group: format!("term-rg-{}", random_suffix),
            location: "eastus".to_string(),
            vm_name: format!("term-vm-{}", random_suffix),
            vm_size: "Standard_D2s_v3".to_string(),
            admin_username: "terminatoradmin".to_string(),
            admin_password: generate_secure_password(),
        }
    }
}

pub struct AzureArmManager {
    config: AzureVmConfig,
}

impl AzureArmManager {
    pub fn new(config: AzureVmConfig) -> Result<Self> {
        // Basic check if Azure CLI is installed and logged in.
        let output = Command::new("az").arg("account").arg("show").output()?;
        if !output.status.success() {
            anyhow::bail!("Failed to get Azure account status. Please run 'az login'.");
        }
        Ok(Self { config })
    }

    async fn create_resource_group_if_not_exists(&self) -> Result<()> {
        println!(
            "Checking/creating resource group: {}",
            &self.config.resource_group
        );
        let output = Command::new("az")
            .args([
                "group",
                "create",
                "--name",
                &self.config.resource_group,
                "--location",
                &self.config.location,
            ])
            .output()
            .context("Failed to execute 'az group create'")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create resource group: {}", stderr);
        }

        println!("✅ Resource group is ready.");
        Ok(())
    }

    pub async fn create_vm(&self) -> Result<()> {
        println!("🚀 Starting Azure VM deployment via ARM Template...");

        self.set_subscription()?;
        self.create_resource_group_if_not_exists().await?;

        let template_path = "terminator-cli/src/azure_deploy.json";

        let location_param = format!("location={}", self.config.location);
        let vm_name_param = format!("vmName={}", self.config.vm_name);
        let vm_size_param = format!("vmSize={}", self.config.vm_size);
        let admin_username_param = format!("adminUsername={}", self.config.admin_username);
        let admin_password_param = format!("adminPassword={}", self.config.admin_password);

        let args = vec![
            "deployment",
            "group",
            "create",
            "--resource-group",
            &self.config.resource_group,
            "--template-file",
            template_path,
            "--parameters",
            &location_param,
            &vm_name_param,
            &vm_size_param,
            &admin_username_param,
            &admin_password_param,
        ];

        let output = Command::new("az")
            .args(&args)
            .output()
            .context("Failed to execute 'az deployment group create'")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("ARM deployment failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);

        println!("\n🎉 Azure VM Deployment Complete!");
        Ok(())
    }

    fn set_subscription(&self) -> Result<()> {
        let output = Command::new("az")
            .args([
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

    pub async fn delete_resource_group(&self) -> Result<()> {
        println!(
            "🗑️  Deleting resource group: {}",
            self.config.resource_group
        );

        let output = Command::new("az")
            .args([
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
        Ok(())
    }
}
