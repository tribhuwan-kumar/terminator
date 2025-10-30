// TypeScript workflow executor

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};
use std::fs;

use rmcp::ErrorData as McpError;

#[derive(Debug, Clone, PartialEq)]
pub enum JsRuntime {
    Bun,
    Node,
}

/// Detect available JavaScript runtime (prefer bun, fallback to node)
pub fn detect_js_runtime() -> JsRuntime {
    // Try bun first
    if let Ok(output) = Command::new("bun").arg("--version").output() {
        if output.status.success() {
            info!("Using bun runtime");
            return JsRuntime::Bun;
        }
    }

    // Fallback to node
    info!("Bun not found, using node runtime");
    JsRuntime::Node
}

#[derive(Debug)]
pub struct TypeScriptWorkflow {
    workflow_path: PathBuf,
    entry_file: String,
}

impl TypeScriptWorkflow {
    /// Validate that only one workflow exists in the folder
    fn validate_single_workflow(path: &PathBuf) -> Result<(), McpError> {
        use std::fs;

        // Count .ts files that might be workflows (excluding terminator.ts itself)
        let mut workflow_files = Vec::new();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            // Check for common workflow file patterns (but not terminator.ts)
                            if file_name.ends_with(".workflow.ts")
                                || (file_name.ends_with(".ts")
                                    && file_name != "terminator.ts"
                                    && file_name.contains("workflow"))
                            {
                                workflow_files.push(file_name.to_string());
                            }
                        }
                    }
                }
            }
        }

        if !workflow_files.is_empty() {
            return Err(McpError::invalid_params(
                format!(
                    "Multiple workflow files detected. Only one workflow per folder is allowed. Found: {}",
                    workflow_files.join(", ")
                ),
                Some(json!({
                    "path": path.display().to_string(),
                    "conflicting_files": workflow_files,
                    "hint": "Move additional workflows to separate folders or rename them to not include 'workflow' in the filename"
                })),
            ));
        }

        Ok(())
    }

    pub fn new(url: &str) -> Result<Self, McpError> {
        let path_str = url
            .strip_prefix("file://")
            .ok_or_else(|| {
                McpError::invalid_params(
                    "TypeScript workflows must use file:// URLs".to_string(),
                    Some(json!({"url": url})),
                )
            })?;

        let path = PathBuf::from(path_str);

        // Determine workflow path and entry file
        let (workflow_path, entry_file) = if path.is_dir() {
            // Directory: MUST have terminator.ts as entrypoint
            let terminator_path = path.join("terminator.ts");
            if !terminator_path.exists() {
                return Err(McpError::invalid_params(
                    "Missing required entrypoint: terminator.ts. TypeScript workflows must use 'terminator.ts' as the entry file.".to_string(),
                    Some(json!({
                        "path": path.display().to_string(),
                        "hint": "Rename your workflow file to 'terminator.ts' or create a terminator.ts that exports your workflow"
                    })),
                ));
            }

            // Validate single workflow per folder
            Self::validate_single_workflow(&path)?;

            (path, "terminator.ts".to_string())
        } else if path.is_file() {
            // File: use parent directory and file name
            let parent = path.parent().ok_or_else(|| {
                McpError::invalid_params(
                    "Cannot determine parent directory".to_string(),
                    Some(json!({"path": path.display().to_string()})),
                )
            })?;
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| {
                    McpError::invalid_params(
                        "Invalid file name".to_string(),
                        Some(json!({"path": path.display().to_string()})),
                    )
                })?;

            (parent.to_path_buf(), file_name.to_string())
        } else {
            return Err(McpError::invalid_params(
                "Workflow path does not exist".to_string(),
                Some(json!({"path": path.display().to_string()})),
            ));
        };

        Ok(Self {
            workflow_path,
            entry_file,
        })
    }

    /// Execute the entire TypeScript workflow with state management
    pub async fn execute(
        &self,
        inputs: Value,
        start_from_step: Option<&str>,
        end_at_step: Option<&str>,
        restored_state: Option<Value>,
    ) -> Result<TypeScriptWorkflowResult, McpError> {
        // Ensure dependencies are installed and cached
        self.ensure_dependencies().await?;

        // Create execution script
        let exec_script = self.create_execution_script(
            inputs,
            start_from_step,
            end_at_step,
            restored_state,
        )?;

        debug!("Executing TypeScript workflow with script:\n{}", exec_script);

        // Execute via bun (priority) or node (fallback)
        let runtime = detect_js_runtime();
        let output = match runtime {
            JsRuntime::Bun => {
                info!(
                    "Executing workflow with bun: {}/{}",
                    self.workflow_path.display(),
                    self.entry_file
                );
                Command::new("bun")
                    .current_dir(&self.workflow_path)
                    .arg("--eval")
                    .arg(&exec_script)
                    .output()
                    .map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute workflow with bun: {}", e),
                            Some(json!({"error": e.to_string()})),
                        )
                    })?
            }
            JsRuntime::Node => {
                info!(
                    "Executing workflow with node: {}/{}",
                    self.workflow_path.display(),
                    self.entry_file
                );
                Command::new("node")
                    .current_dir(&self.workflow_path)
                    .arg("--import")
                    .arg("tsx/esm")
                    .arg("--eval")
                    .arg(&exec_script)
                    .output()
                    .map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute workflow with node: {}", e),
                            Some(json!({"error": e.to_string()})),
                        )
                    })?
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(McpError::internal_error(
                format!("Workflow execution failed: {}", stderr),
                Some(json!({
                    "stderr": stderr.to_string(),
                    "stdout": stdout.to_string(),
                    "exit_code": output.status.code(),
                })),
            ));
        }

        // Parse result
        let result_json = String::from_utf8_lossy(&output.stdout);
        debug!("Workflow output:\n{}", result_json);

        let result: TypeScriptWorkflowResult = serde_json::from_str(&result_json).map_err(|e| {
            McpError::internal_error(
                format!("Invalid workflow result: {}", e),
                Some(json!({
                    "error": e.to_string(),
                    "output": result_json.to_string(),
                })),
            )
        })?;

        Ok(result)
    }

    fn create_execution_script(
        &self,
        inputs: Value,
        start_from_step: Option<&str>,
        end_at_step: Option<&str>,
        restored_state: Option<Value>,
    ) -> Result<String, McpError> {
        let inputs_json = serde_json::to_string(&inputs).map_err(|e| {
            McpError::internal_error(
                format!("Failed to serialize inputs: {}", e),
                Some(json!({"error": e.to_string()})),
            )
        })?;

        let start_from_json = start_from_step
            .map(|s| format!("'{}'", s.replace("'", "\\'")))
            .unwrap_or_else(|| "null".to_string());

        let end_at_json = end_at_step
            .map(|s| format!("'{}'", s.replace("'", "\\'")))
            .unwrap_or_else(|| "null".to_string());

        let restored_state_json = if let Some(state) = restored_state {
            serde_json::to_string(&state).map_err(|e| {
                McpError::internal_error(
                    format!("Failed to serialize restored state: {}", e),
                    Some(json!({"error": e.to_string()})),
                )
            })?
        } else {
            "null".to_string()
        };

        // Convert Windows path to forward slashes for file:// URL
        let workflow_path_str = self.workflow_path.display().to_string();
        let workflow_path = workflow_path_str.replace('\\', "/");
        let entry_file = &self.entry_file;

        Ok(format!(
            r#"
import {{ createWorkflowRunner }} from '../../packages/terminator-workflow/src/runner.js';

const workflow = await import('file://{workflow_path}/{entry_file}');

const runner = createWorkflowRunner({{
    workflow: workflow.default,
    inputs: {inputs_json},
    startFromStep: {start_from_json},
    endAtStep: {end_at_json},
    restoredState: {restored_state_json},
}});

const result = await runner.run();

// Output: metadata + execution result + state
console.log(JSON.stringify({{
    metadata: workflow.default.getMetadata(),
    result: result,
    state: runner.getState(),
}}));
"#
        ))
    }

    /// Ensure dependencies are installed
    ///
    /// Simple strategy: Just run bun/npm install in the workflow directory.
    /// Since workflow is mounted from S3, node_modules will be persisted there automatically.
    async fn ensure_dependencies(&self) -> Result<(), McpError> {
        let package_json_path = self.workflow_path.join("package.json");

        // Check if package.json exists
        if !package_json_path.exists() {
            info!("No package.json found - skipping dependency installation");
            return Ok(());
        }

        let workflow_node_modules = self.workflow_path.join("node_modules");

        // If node_modules already exists, we're good (already installed and persisted in S3)
        if workflow_node_modules.exists() {
            info!("✓ Dependencies already installed (node_modules exists in workflow directory)");
            return Ok(());
        }

        // Install dependencies in workflow directory (will be persisted to S3)
        info!("⏳ Installing dependencies...");
        let runtime = detect_js_runtime();

        let install_result = match runtime {
            JsRuntime::Bun => {
                Command::new("bun")
                    .arg("install")
                    .current_dir(&self.workflow_path)
                    .output()
            }
            JsRuntime::Node => {
                Command::new("npm")
                    .arg("install")
                    .current_dir(&self.workflow_path)
                    .output()
            }
        }
        .map_err(|e| {
            McpError::internal_error(
                format!("Failed to run dependency installation: {}", e),
                Some(json!({"error": e.to_string()})),
            )
        })?;

        if !install_result.status.success() {
            let stderr = String::from_utf8_lossy(&install_result.stderr);
            return Err(McpError::internal_error(
                format!("Dependency installation failed: {}", stderr),
                Some(json!({
                    "stderr": stderr.to_string(),
                    "stdout": String::from_utf8_lossy(&install_result.stdout).to_string(),
                })),
            ));
        }

        info!("✓ Dependencies installed successfully");
        info!("✓ node_modules will be persisted to S3 with workflow");

        Ok(())
    }

}

#[derive(Debug, Deserialize, Serialize)]
pub struct TypeScriptWorkflowResult {
    pub metadata: WorkflowMetadata,
    pub result: WorkflowExecutionResult,
    pub state: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkflowMetadata {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub input: Value,
    pub steps: Vec<StepMetadata>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StepMetadata {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkflowExecutionResult {
    pub status: String,
    pub last_step_id: Option<String>,
    pub last_step_index: usize,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_bun_or_node() {
        let runtime = detect_js_runtime();
        // Should return either Bun or Node (depending on environment)
        assert!(matches!(runtime, JsRuntime::Bun) || matches!(runtime, JsRuntime::Node));
    }

    #[test]
    fn test_typescript_workflow_from_file() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let workflow_file = temp_dir.path().join("test-workflow.ts");
        fs::write(&workflow_file, "export default {};").unwrap();

        let url = format!("file://{}", workflow_file.display());
        let ts_workflow = TypeScriptWorkflow::new(&url).unwrap();

        assert_eq!(ts_workflow.entry_file, "test-workflow.ts");
        assert_eq!(ts_workflow.workflow_path, temp_dir.path());
    }

    #[test]
    fn test_typescript_workflow_requires_terminator_ts() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create terminator.ts
        fs::write(temp_dir.path().join("terminator.ts"), "export default {};").unwrap();

        let url = format!("file://{}", temp_dir.path().display());
        let ts_workflow = TypeScriptWorkflow::new(&url).unwrap();

        assert_eq!(ts_workflow.entry_file, "terminator.ts");
        assert_eq!(ts_workflow.workflow_path, temp_dir.path());
    }

    #[test]
    fn test_typescript_workflow_missing_terminator_ts() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create other workflow file, but no terminator.ts
        fs::write(temp_dir.path().join("my-workflow.ts"), "export default {};").unwrap();

        let url = format!("file://{}", temp_dir.path().display());
        let result = TypeScriptWorkflow::new(&url);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Missing required entrypoint: terminator.ts"));
    }

    #[test]
    fn test_single_workflow_validation_passes() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create only terminator.ts (no other workflow files)
        fs::write(temp_dir.path().join("terminator.ts"), "export default {};").unwrap();
        fs::write(temp_dir.path().join("utils.ts"), "export const helper = () => {};").unwrap();

        let url = format!("file://{}", temp_dir.path().display());
        let result = TypeScriptWorkflow::new(&url);

        assert!(result.is_ok());
    }

    #[test]
    fn test_single_workflow_validation_fails_with_multiple_workflows() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create terminator.ts and another workflow file
        fs::write(temp_dir.path().join("terminator.ts"), "export default {};").unwrap();
        fs::write(temp_dir.path().join("my-workflow.ts"), "export default {};").unwrap();

        let url = format!("file://{}", temp_dir.path().display());
        let result = TypeScriptWorkflow::new(&url);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Multiple workflow files detected"));
        assert!(err.message.contains("my-workflow.ts"));
    }

}
