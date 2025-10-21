#!/usr/bin/env node
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  ListToolsRequestSchema,
  CallToolRequestSchema,
  ErrorCode,
  McpError,
} from "@modelcontextprotocol/sdk/types.js";
import * as fs from "fs/promises";
import * as path from "path";
import YAML from "yaml";
import { z } from "zod";

// ============================================================================
// Zod Schemas for Tool Arguments
// ============================================================================

const ReadWorkflowSchema = z.object({
  file_path: z.string().describe("Absolute path to the workflow YAML file"),
});

const ListWorkflowsSchema = z.object({
  directory: z.string().describe("Directory path to search for workflow files"),
  pattern: z.string().optional().describe("Glob pattern to filter files (e.g., '*.yml', '**/*.yaml')"),
});

const SearchWorkflowsSchema = z.object({
  directory: z.string().describe("Directory path to search in"),
  pattern: z.string().describe("Text pattern or regex to search for"),
  use_regex: z.boolean().optional().default(false).describe("Use regex for pattern matching"),
});

const EditWorkflowSchema = z.object({
  file_path: z.string().describe("Absolute path to the workflow file to edit"),
  old_string: z.string().describe("Exact string to find and replace"),
  new_string: z.string().describe("String to replace with"),
  replace_all: z.boolean().optional().default(false).describe("Replace all occurrences (default: false)"),
});

const CreateWorkflowSchema = z.object({
  file_path: z.string().describe("Absolute path for the new workflow file"),
  content: z.string().describe("YAML content for the workflow"),
});

const ValidateWorkflowSchema = z.object({
  file_path: z.string().describe("Absolute path to the workflow file to validate"),
});

// ============================================================================
// Helper Functions
// ============================================================================

async function readFile(filePath: string): Promise<string> {
  try {
    return await fs.readFile(filePath, "utf-8");
  } catch (error: any) {
    if (error.code === "ENOENT") {
      throw new McpError(ErrorCode.InvalidParams, `File not found: ${filePath}`);
    }
    throw new McpError(ErrorCode.InternalError, `Failed to read file: ${error.message}`);
  }
}

async function writeFile(filePath: string, content: string): Promise<void> {
  try {
    const dir = path.dirname(filePath);
    await fs.mkdir(dir, { recursive: true });
    await fs.writeFile(filePath, content, "utf-8");
  } catch (error: any) {
    throw new McpError(ErrorCode.InternalError, `Failed to write file: ${error.message}`);
  }
}

async function listFiles(directory: string, pattern?: string): Promise<string[]> {
  try {
    const stats = await fs.stat(directory);
    if (!stats.isDirectory()) {
      throw new McpError(ErrorCode.InvalidParams, `Not a directory: ${directory}`);
    }

    const files = await fs.readdir(directory, { recursive: true });
    const yamlFiles = files.filter((file) => {
      const ext = path.extname(file);
      return ext === ".yml" || ext === ".yaml";
    });

    if (pattern) {
      const regex = new RegExp(pattern.replace(/\*/g, ".*"));
      return yamlFiles.filter((file) => regex.test(file)).map((file) => path.join(directory, file));
    }

    return yamlFiles.map((file) => path.join(directory, file));
  } catch (error: any) {
    if (error.code === "ENOENT") {
      throw new McpError(ErrorCode.InvalidParams, `Directory not found: ${directory}`);
    }
    throw new McpError(ErrorCode.InternalError, `Failed to list files: ${error.message}`);
  }
}

async function searchInFiles(
  directory: string,
  pattern: string,
  useRegex: boolean
): Promise<Array<{ file: string; matches: number; lines: string[] }>> {
  const files = await listFiles(directory);
  const results: Array<{ file: string; matches: number; lines: string[] }> = [];

  for (const file of files) {
    try {
      const content = await readFile(file);
      const lines = content.split("\n");
      const matchingLines: string[] = [];

      if (useRegex) {
        const regex = new RegExp(pattern, "gi");
        lines.forEach((line, idx) => {
          if (regex.test(line)) {
            matchingLines.push(`${idx + 1}: ${line}`);
          }
        });
      } else {
        lines.forEach((line, idx) => {
          if (line.includes(pattern)) {
            matchingLines.push(`${idx + 1}: ${line}`);
          }
        });
      }

      if (matchingLines.length > 0) {
        results.push({
          file,
          matches: matchingLines.length,
          lines: matchingLines,
        });
      }
    } catch (error) {
      // Skip files that can't be read
      continue;
    }
  }

  return results;
}

function validateYAML(content: string): { valid: boolean; error?: string; parsed?: any } {
  try {
    const parsed = YAML.parse(content);

    // Basic workflow validation
    if (!parsed || typeof parsed !== "object") {
      return { valid: false, error: "Workflow must be a YAML object" };
    }

    // Check for required execute_sequence structure
    if (parsed.tool_name !== "execute_sequence") {
      return {
        valid: false,
        error: "Workflow must have 'tool_name: execute_sequence' at root level"
      };
    }

    if (!parsed.arguments || typeof parsed.arguments !== "object") {
      return {
        valid: false,
        error: "Workflow must have 'arguments' object"
      };
    }

    if (!Array.isArray(parsed.arguments.steps)) {
      return {
        valid: false,
        error: "Workflow must have 'arguments.steps' array"
      };
    }

    return { valid: true, parsed };
  } catch (error: any) {
    return { valid: false, error: error.message };
  }
}

// ============================================================================
// Tool Implementations
// ============================================================================

async function handleReadWorkflow(args: z.infer<typeof ReadWorkflowSchema>) {
  const content = await readFile(args.file_path);
  const lines = content.split("\n");
  const numberedContent = lines.map((line, idx) => `${idx + 1}\t${line}`).join("\n");

  return {
    content: [
      {
        type: "text",
        text: `File: ${args.file_path}\n\n${numberedContent}`,
      },
    ],
  };
}

async function handleListWorkflows(args: z.infer<typeof ListWorkflowsSchema>) {
  const files = await listFiles(args.directory, args.pattern);

  const fileDetails = await Promise.all(
    files.map(async (file) => {
      try {
        const stats = await fs.stat(file);
        const content = await readFile(file);
        const validation = validateYAML(content);

        return {
          path: file,
          size: stats.size,
          modified: stats.mtime.toISOString(),
          valid: validation.valid,
          error: validation.error,
        };
      } catch (error: any) {
        return {
          path: file,
          error: error.message,
        };
      }
    })
  );

  return {
    content: [
      {
        type: "text",
        text: `Found ${files.length} workflow file(s) in ${args.directory}\n\n${JSON.stringify(fileDetails, null, 2)}`,
      },
    ],
  };
}

async function handleSearchWorkflows(args: z.infer<typeof SearchWorkflowsSchema>) {
  const results = await searchInFiles(args.directory, args.pattern, args.use_regex || false);

  if (results.length === 0) {
    return {
      content: [
        {
          type: "text",
          text: `No matches found for pattern: ${args.pattern}`,
        },
      ],
    };
  }

  const summary = results.map((r) => ({
    file: r.file,
    matches: r.matches,
    preview: r.lines.slice(0, 5),
  }));

  return {
    content: [
      {
        type: "text",
        text: `Found ${results.length} file(s) with matches:\n\n${JSON.stringify(summary, null, 2)}`,
      },
    ],
  };
}

async function handleEditWorkflow(args: z.infer<typeof EditWorkflowSchema>) {
  const content = await readFile(args.file_path);

  // Check if old_string exists
  if (!content.includes(args.old_string)) {
    throw new McpError(
      ErrorCode.InvalidParams,
      `String not found in file: "${args.old_string}"`
    );
  }

  // Check if old_string is unique (unless replace_all is true)
  if (!args.replace_all) {
    const occurrences = content.split(args.old_string).length - 1;
    if (occurrences > 1) {
      throw new McpError(
        ErrorCode.InvalidParams,
        `String appears ${occurrences} times in file. Use replace_all: true to replace all occurrences, or provide a more unique string.`
      );
    }
  }

  // Perform replacement
  const newContent = args.replace_all
    ? content.replaceAll(args.old_string, args.new_string)
    : content.replace(args.old_string, args.new_string);

  await writeFile(args.file_path, newContent);

  // Validate after editing
  const validation = validateYAML(newContent);

  return {
    content: [
      {
        type: "text",
        text: `Successfully edited ${args.file_path}\n\nReplaced ${args.replace_all ? "all occurrences" : "1 occurrence"} of:\n"${args.old_string}"\n\nWith:\n"${args.new_string}"\n\nValidation: ${validation.valid ? "✅ Valid" : `❌ Invalid - ${validation.error}`}`,
      },
    ],
  };
}

async function handleCreateWorkflow(args: z.infer<typeof CreateWorkflowSchema>) {
  // Check if file already exists
  try {
    await fs.access(args.file_path);
    throw new McpError(
      ErrorCode.InvalidParams,
      `File already exists: ${args.file_path}. Use edit_workflow to modify existing files.`
    );
  } catch (error: any) {
    if (error.code !== "ENOENT") {
      throw error;
    }
  }

  // Validate YAML before creating
  const validation = validateYAML(args.content);
  if (!validation.valid) {
    throw new McpError(
      ErrorCode.InvalidParams,
      `Invalid workflow YAML: ${validation.error}`
    );
  }

  await writeFile(args.file_path, args.content);

  return {
    content: [
      {
        type: "text",
        text: `Successfully created workflow: ${args.file_path}\n\n✅ YAML validation passed`,
      },
    ],
  };
}

async function handleValidateWorkflow(args: z.infer<typeof ValidateWorkflowSchema>) {
  const content = await readFile(args.file_path);
  const validation = validateYAML(content);

  if (validation.valid) {
    const workflow = validation.parsed;
    const stepCount = workflow.arguments?.steps?.length || 0;
    const hasVariables = !!workflow.arguments?.variables;
    const hasInputs = !!workflow.arguments?.inputs;

    return {
      content: [
        {
          type: "text",
          text: `✅ Workflow is valid\n\nFile: ${args.file_path}\nSteps: ${stepCount}\nVariables: ${hasVariables ? "Yes" : "No"}\nInputs: ${hasInputs ? "Yes" : "No"}\n\nStructure:\n${JSON.stringify(workflow, null, 2)}`,
        },
      ],
    };
  } else {
    return {
      content: [
        {
          type: "text",
          text: `❌ Workflow is invalid\n\nFile: ${args.file_path}\nError: ${validation.error}`,
        },
      ],
    };
  }
}

// ============================================================================
// MCP Server Setup
// ============================================================================

async function main() {
  const server = new Server(
    {
      name: "workflow-builder-mcp",
      version: "0.1.0",
    },
    {
      capabilities: {
        tools: {},
      },
    }
  );

  server.setRequestHandler(ListToolsRequestSchema, async () => ({
    tools: [
      {
        name: "read_workflow",
        description: "Read a workflow YAML file with line numbers. Similar to Claude Code's 'read' tool.",
        inputSchema: {
          type: "object",
          properties: {
            file_path: {
              type: "string",
              description: "Absolute path to the workflow YAML file",
            },
          },
          required: ["file_path"],
        },
      },
      {
        name: "list_workflows",
        description: "List all workflow files in a directory with metadata and validation status. Similar to Claude Code's 'glob' tool.",
        inputSchema: {
          type: "object",
          properties: {
            directory: {
              type: "string",
              description: "Directory path to search for workflow files",
            },
            pattern: {
              type: "string",
              description: "Optional glob pattern to filter files (e.g., '*.yml', '**/*.yaml')",
            },
          },
          required: ["directory"],
        },
      },
      {
        name: "search_workflows",
        description: "Search for text patterns across workflow files. Similar to Claude Code's 'grep' tool.",
        inputSchema: {
          type: "object",
          properties: {
            directory: {
              type: "string",
              description: "Directory path to search in",
            },
            pattern: {
              type: "string",
              description: "Text pattern or regex to search for",
            },
            use_regex: {
              type: "boolean",
              description: "Use regex for pattern matching (default: false)",
              default: false,
            },
          },
          required: ["directory", "pattern"],
        },
      },
      {
        name: "edit_workflow",
        description: "Edit a workflow file using exact string replacement. Similar to Claude Code's 'edit' tool. Fails if the old_string is not found or not unique (unless replace_all is true).",
        inputSchema: {
          type: "object",
          properties: {
            file_path: {
              type: "string",
              description: "Absolute path to the workflow file to edit",
            },
            old_string: {
              type: "string",
              description: "Exact string to find and replace (must be unique unless replace_all is true)",
            },
            new_string: {
              type: "string",
              description: "String to replace with",
            },
            replace_all: {
              type: "boolean",
              description: "Replace all occurrences (default: false)",
              default: false,
            },
          },
          required: ["file_path", "old_string", "new_string"],
        },
      },
      {
        name: "create_workflow",
        description: "Create a new workflow YAML file. Similar to Claude Code's 'write' tool. Validates YAML syntax before creating.",
        inputSchema: {
          type: "object",
          properties: {
            file_path: {
              type: "string",
              description: "Absolute path for the new workflow file",
            },
            content: {
              type: "string",
              description: "YAML content for the workflow",
            },
          },
          required: ["file_path", "content"],
        },
      },
      {
        name: "validate_workflow",
        description: "Validate a workflow file's YAML syntax and Terminator schema requirements.",
        inputSchema: {
          type: "object",
          properties: {
            file_path: {
              type: "string",
              description: "Absolute path to the workflow file to validate",
            },
          },
          required: ["file_path"],
        },
      },
    ],
  }));

  server.setRequestHandler(CallToolRequestSchema, async (request: any) => {
    try {
      const { name, arguments: args } = request.params;

      switch (name) {
        case "read_workflow": {
          const validated = ReadWorkflowSchema.parse(args);
          return await handleReadWorkflow(validated);
        }

        case "list_workflows": {
          const validated = ListWorkflowsSchema.parse(args);
          return await handleListWorkflows(validated);
        }

        case "search_workflows": {
          const validated = SearchWorkflowsSchema.parse(args);
          return await handleSearchWorkflows(validated);
        }

        case "edit_workflow": {
          const validated = EditWorkflowSchema.parse(args);
          return await handleEditWorkflow(validated);
        }

        case "create_workflow": {
          const validated = CreateWorkflowSchema.parse(args);
          return await handleCreateWorkflow(validated);
        }

        case "validate_workflow": {
          const validated = ValidateWorkflowSchema.parse(args);
          return await handleValidateWorkflow(validated);
        }

        default:
          throw new McpError(ErrorCode.MethodNotFound, `Unknown tool: ${name}`);
      }
    } catch (error: any) {
      if (error instanceof z.ZodError) {
        throw new McpError(
          ErrorCode.InvalidParams,
          `Invalid parameters: ${error.errors.map((e) => e.message).join(", ")}`
        );
      }

      if (error instanceof McpError) {
        throw error;
      }

      throw new McpError(ErrorCode.InternalError, error.message || "An unexpected error occurred");
    }
  });

  // Check if running in HTTP mode
  const useHttp = process.env.MCP_TRANSPORT === "http" || process.argv.includes("--http");
  const port = parseInt(process.env.MCP_PORT || process.env.PORT || "3000");

  if (useHttp) {
    // Streamable HTTP transport
    const { StreamableHTTPServerTransport } = await import("@modelcontextprotocol/sdk/server/streamableHttp.js");
    const express = (await import("express")).default;
    const { randomUUID } = await import("node:crypto");

    const app = express();
    app.use(express.json());
    // Enable CORS for Tauri app
    const cors = (await import("cors")).default;
    app.use(cors({
      origin: true,
      credentials: true,
      methods: ['GET', 'POST', 'DELETE', 'OPTIONS'],
      allowedHeaders: ['Content-Type', 'Authorization'],
    }));


    const transport = new StreamableHTTPServerTransport({
      sessionIdGenerator: () => randomUUID(),
    });
    await server.connect(transport);

    // Handle all HTTP methods (GET for SSE, POST for JSON-RPC, DELETE for session close)
    app.all("/mcp", async (req, res) => {
      // Cast Express request to IncomingMessage for MCP transport
      // Express req extends IncomingMessage so this is safe
      await transport.handleRequest(req as any, res as any, req.body);
    });

    app.listen(port, () => {
      console.error(`Workflow Builder MCP server running on http://localhost:${port}/mcp`);
    });
  } else {
    // Stdio transport (default)
    const transport = new StdioServerTransport();
    await server.connect(transport);

    console.error("Workflow Builder MCP server running (stdio)");
  }
}

main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
