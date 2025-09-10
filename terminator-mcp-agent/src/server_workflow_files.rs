// Local helper replicated from server.rs to avoid cross-module privacy issues
fn scan_yaml_files(folder_path: &str) -> Result<Vec<serde_json::Value>, McpError> {
    let mut files = Vec::new();

    let dir = std::fs::read_dir(folder_path).map_err(|e| {
        McpError::invalid_params(
            "Failed to read directory",
            Some(json!({"error": e.to_string(), "path": folder_path})),
        )
    })?;

    for entry in dir {
        let entry = entry.map_err(|e| {
            McpError::internal_error(
                "Directory entry error",
                Some(json!({"error": e.to_string()})),
            )
        })?;

        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "yaml" || ext == "yml" {
                    let metadata = entry.metadata().ok();
                    let file_name = path
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    files.push(json!({
                        "name": file_name,
                        "file_path": path.to_string_lossy(),
                        "size": metadata.as_ref().map(|m| m.len()).unwrap_or(0),
                        "modified": metadata.and_then(|m| m.modified().ok())
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                    }));
                }
            }
        }
    }

    Ok(files)
}

// Helper to scan and optionally read YAML files
fn scan_yaml_files_with_content(
    folder_path: &str,
    return_raw: bool,
) -> Result<Vec<serde_json::Value>, McpError> {
    let mut files = Vec::new();

    let dir = std::fs::read_dir(folder_path).map_err(|e| {
        McpError::invalid_params(
            "Failed to read directory",
            Some(json!({"error": e.to_string(), "path": folder_path})),
        )
    })?;

    for entry in dir {
        let entry = entry.map_err(|e| {
            McpError::internal_error(
                "Directory entry error",
                Some(json!({"error": e.to_string()})),
            )
        })?;

        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "yaml" || ext == "yml" {
                    let path_str = path.to_string_lossy().to_string();

                    if return_raw {
                        // Read and parse each file
                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                let parse_result =
                                    serde_yaml::from_str::<serde_json::Value>(&content);

                                match parse_result {
                                    Ok(workflow) => {
                                        files.push(json!({
                                            "file_path": path_str,
                                            "raw_yaml": content,
                                            "parsed": workflow
                                        }));
                                    }
                                    Err(e) => {
                                        files.push(json!({
                                            "file_path": path_str,
                                            "raw_yaml": content,
                                            "parsed": null,
                                            "parse_error": e.to_string()
                                        }));
                                    }
                                }
                            }
                            Err(e) => {
                                files.push(json!({
                                    "file_path": path_str,
                                    "read_error": e.to_string()
                                }));
                            }
                        }
                    } else {
                        // Original behavior - just metadata
                        let metadata = entry.metadata().ok();
                        let file_name = path
                            .file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        files.push(json!({
                            "name": file_name,
                            "file_path": path_str,
                            "size": metadata.as_ref().map(|m| m.len()).unwrap_or(0),
                            "modified": metadata.and_then(|m| m.modified().ok())
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs())
                        }));
                    }
                }
            }
        }
    }

    Ok(files)
}
use crate::utils::DesktopWrapper;
use crate::utils::{ExportWorkflowSequenceArgs, ImportWorkflowSequenceArgs};
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use serde_json::json;

impl DesktopWrapper {
    pub async fn export_workflow_sequence_impl(
        &self,
        args: ExportWorkflowSequenceArgs,
    ) -> Result<CallToolResult, McpError> {
        let use_regex = args.use_regex.unwrap_or(false);
        let create_if_missing = args.create_if_missing.unwrap_or(true);

        // Read existing file or start with empty content
        let current_content = match std::fs::read_to_string(&args.file_path) {
            Ok(content) => content,
            Err(_) => {
                if create_if_missing {
                    String::new()
                } else {
                    return Err(McpError::invalid_params(
                        "File does not exist and create_if_missing is false",
                        Some(json!({"file_path": args.file_path})),
                    ));
                }
            }
        };

        let new_content = if let Some(find_pattern) = &args.find_pattern {
            // Replace mode - find and replace pattern with content
            if use_regex {
                // Use regex replacement
                match regex::Regex::new(find_pattern) {
                    Ok(re) => {
                        // CRITICAL: Escape $ characters in the replacement string to prevent
                        // them from being interpreted as capture group references.
                        // Without this, template strings like "${{ selectors.window }}" get corrupted.
                        let escaped_content = args.content.replace("$", "$$");
                        let result = re.replace_all(&current_content, escaped_content.as_str());
                        if result == current_content {
                            return Err(McpError::invalid_params(
                                "Pattern not found in file",
                                Some(json!({"pattern": find_pattern, "file": args.file_path})),
                            ));
                        }
                        result.to_string()
                    }
                    Err(e) => {
                        return Err(McpError::invalid_params(
                            "Invalid regex pattern",
                            Some(json!({"pattern": find_pattern, "error": e.to_string()})),
                        ));
                    }
                }
            } else {
                // Simple string replacement
                if !current_content.contains(find_pattern) {
                    return Err(McpError::invalid_params(
                        "Pattern not found in file",
                        Some(json!({"pattern": find_pattern, "file": args.file_path})),
                    ));
                }
                current_content.replace(find_pattern, &args.content)
            }
        } else {
            // Append mode - add content to end of file
            if current_content.is_empty() {
                args.content
            } else if current_content.ends_with('\n') {
                format!("{}{}", current_content, args.content)
            } else {
                format!("{}\n{}", current_content, args.content)
            }
        };

        // Write back to file
        std::fs::write(&args.file_path, &new_content).map_err(|e| {
            McpError::internal_error(
                "Failed to write file",
                Some(json!({"error": e.to_string(), "path": args.file_path})),
            )
        })?;

        // Return success
        Ok(CallToolResult::success(vec![Content::json(json!({
            "action": "edit_workflow_file",
            "status": "success",
            "file_path": args.file_path,
            "operation": if args.find_pattern.is_some() { "replace" } else { "append" },
            "pattern_type": if use_regex { "regex" } else { "string" },
            "file_size": new_content.len(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))?]))
    }

    pub async fn import_workflow_sequence_impl(
        &self,
        args: ImportWorkflowSequenceArgs,
    ) -> Result<CallToolResult, McpError> {
        let return_raw = args.return_raw.unwrap_or(false);

        match (args.file_path, args.folder_path) {
            // Load single file
            (Some(file_path), None) => {
                let content = std::fs::read_to_string(&file_path).map_err(|e| {
                    McpError::invalid_params(
                        "Failed to read file",
                        Some(json!({"error": e.to_string(), "path": file_path})),
                    )
                })?;

                if return_raw {
                    // Try to parse YAML but don't fail if invalid
                    let parse_result = serde_yaml::from_str::<serde_json::Value>(&content);

                    match parse_result {
                        Ok(workflow) => {
                            // Successfully parsed - return both raw and parsed
                            Ok(CallToolResult::success(vec![Content::json(json!({
                                "raw_yaml": content,
                                "parsed": {
                                    "workflow": workflow,
                                    "workflow_info": {
                                        "valid": true,
                                        "file_path": file_path
                                    }
                                },
                                "file_path": file_path
                            }))?]))
                        }
                        Err(e) => {
                            // Parse failed - still return raw content with error
                            Ok(CallToolResult::success(vec![Content::json(json!({
                                "raw_yaml": content,
                                "parsed": null,
                                "parse_error": e.to_string(),
                                "file_path": file_path
                            }))?]))
                        }
                    }
                } else {
                    // Original behavior - parse and fail on error
                    let workflow: serde_json::Value =
                        serde_yaml::from_str(&content).map_err(|e| {
                            McpError::invalid_params(
                                "Invalid YAML format",
                                Some(json!({"error": e.to_string()})),
                            )
                        })?;

                    Ok(CallToolResult::success(vec![Content::json(json!({
                        "operation": "load_file",
                        "file_path": file_path,
                        "workflow": workflow
                    }))?]))
                }
            }
            // Scan folder
            (None, Some(folder_path)) => {
                if return_raw {
                    // Use new helper that reads content
                    let files = scan_yaml_files_with_content(&folder_path, true)?;

                    Ok(CallToolResult::success(vec![Content::json(json!({
                        "files": files
                    }))?]))
                } else {
                    // Original behavior
                    let files = scan_yaml_files(&folder_path)?;

                    Ok(CallToolResult::success(vec![Content::json(json!({
                        "operation": "scan_folder",
                        "folder_path": folder_path,
                        "files": files,
                        "count": files.len()
                    }))?]))
                }
            }
            // Error cases
            (Some(_), Some(_)) => Err(McpError::invalid_params(
                "Provide either file_path OR folder_path, not both",
                None,
            )),
            (None, None) => Err(McpError::invalid_params(
                "Must provide either file_path or folder_path",
                None,
            )),
        }
    }
}
