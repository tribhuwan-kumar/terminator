import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import * as fs from "fs/promises";
import * as path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const TEST_DIR = path.join(__dirname, "test-workflows-integration");

describe("Workflow Builder MCP - Integration Tests", () => {
  let client: Client;
  let transport: StdioClientTransport;

  beforeAll(async () => {
    // Create test directory
    await fs.mkdir(TEST_DIR, { recursive: true });

    // Start MCP server
    client = new Client(
      {
        name: "test-client",
        version: "1.0.0",
      },
      {
        capabilities: {},
      }
    );

    transport = new StdioClientTransport({
      command: "node",
      args: [path.join(__dirname, "../dist/index.js")],
      env: process.env,
    });

    await client.connect(transport);
  });

  afterAll(async () => {
    // Cleanup
    await client.close();

    // Remove test files
    try {
      await fs.rm(TEST_DIR, { recursive: true, force: true });
    } catch (e) {
      // Ignore cleanup errors
    }
  });

  describe("MCP Server", () => {
    it("should list all available tools", async () => {
      const tools = await client.listTools();

      expect(tools.tools).toBeDefined();
      expect(tools.tools.length).toBe(6);

      const toolNames = tools.tools.map((t) => t.name);
      expect(toolNames).toContain("read_workflow");
      expect(toolNames).toContain("list_workflows");
      expect(toolNames).toContain("search_workflows");
      expect(toolNames).toContain("edit_workflow");
      expect(toolNames).toContain("create_workflow");
      expect(toolNames).toContain("validate_workflow");
    });
  });

  describe("create_workflow", () => {
    it("should create a valid workflow", async () => {
      const filePath = path.join(TEST_DIR, "test-create.yml");
      const content = `tool_name: execute_sequence
arguments:
  steps:
    - tool_name: navigate_browser
      arguments:
        url: "https://example.com"
        browser: "chrome"
`;

      const result = await client.callTool({
        name: "create_workflow",
        arguments: {
          file_path: filePath,
          content,
        },
      });

      expect(result.content).toBeDefined();
      expect(result.content[0].text).toContain("Successfully created workflow");
      expect(result.content[0].text).toContain(filePath);
    });

    it("should reject invalid workflow", async () => {
      const filePath = path.join(TEST_DIR, "invalid.yml");
      const content = `tool_name: wrong_tool
arguments:
  invalid: true
`;

      await expect(
        client.callTool({
          name: "create_workflow",
          arguments: {
            file_path: filePath,
            content,
          },
        })
      ).rejects.toThrow();
    });

    it("should reject duplicate file", async () => {
      const filePath = path.join(TEST_DIR, "duplicate.yml");
      const content = `tool_name: execute_sequence
arguments:
  steps: []
`;

      // Create first time
      await client.callTool({
        name: "create_workflow",
        arguments: {
          file_path: filePath,
          content,
        },
      });

      // Try to create again
      await expect(
        client.callTool({
          name: "create_workflow",
          arguments: {
            file_path: filePath,
            content,
          },
        })
      ).rejects.toThrow();
    });
  });

  describe("read_workflow", () => {
    it("should read workflow with line numbers", async () => {
      const filePath = path.join(TEST_DIR, "read-test.yml");
      const content = `tool_name: execute_sequence
arguments:
  steps: []
`;

      // Create workflow first
      await client.callTool({
        name: "create_workflow",
        arguments: { file_path: filePath, content },
      });

      const result = await client.callTool({
        name: "read_workflow",
        arguments: { file_path: filePath },
      });

      expect(result.content[0].text).toContain("1\ttool_name: execute_sequence");
      expect(result.content[0].text).toContain("2\targuments:");
    });
  });

  describe("edit_workflow", () => {
    it("should edit workflow with unique string", async () => {
      const filePath = path.join(TEST_DIR, "edit-test.yml");
      const content = `tool_name: execute_sequence
arguments:
  steps:
    - tool_name: navigate_browser
      arguments:
        url: "https://example.com"
`;

      await client.callTool({
        name: "create_workflow",
        arguments: { file_path: filePath, content },
      });

      const result = await client.callTool({
        name: "edit_workflow",
        arguments: {
          file_path: filePath,
          old_string: "https://example.com",
          new_string: "https://mediar.ai",
        },
      });

      expect(result.content[0].text).toContain("Successfully edited");
      expect(result.content[0].text).toContain("https://mediar.ai");
    });

    it("should reject non-unique string without replace_all", async () => {
      const filePath = path.join(TEST_DIR, "non-unique.yml");
      const content = `tool_name: execute_sequence
arguments:
  steps:
    - tool_name: navigate_browser
      arguments:
        url: "https://example.com"
    - tool_name: navigate_browser
      arguments:
        url: "https://example.com"
`;

      await client.callTool({
        name: "create_workflow",
        arguments: { file_path: filePath, content },
      });

      await expect(
        client.callTool({
          name: "edit_workflow",
          arguments: {
            file_path: filePath,
            old_string: "https://example.com",
            new_string: "https://mediar.ai",
          },
        })
      ).rejects.toThrow();
    });

    it("should replace all with replace_all flag", async () => {
      const filePath = path.join(TEST_DIR, "replace-all.yml");
      const content = `tool_name: execute_sequence
arguments:
  steps:
    - tool_name: navigate_browser
      arguments:
        url: "https://example.com"
    - tool_name: navigate_browser
      arguments:
        url: "https://example.com"
`;

      await client.callTool({
        name: "create_workflow",
        arguments: { file_path: filePath, content },
      });

      const result = await client.callTool({
        name: "edit_workflow",
        arguments: {
          file_path: filePath,
          old_string: "https://example.com",
          new_string: "https://mediar.ai",
          replace_all: true,
        },
      });

      expect(result.content[0].text).toContain("Successfully edited");
      expect(result.content[0].text).toContain("all occurrences");
    });
  });

  describe("list_workflows", () => {
    it("should list all workflows in directory", async () => {
      const result = await client.callTool({
        name: "list_workflows",
        arguments: { directory: TEST_DIR },
      });

      expect(result.content[0].text).toContain("workflow file(s)");
    });
  });

  describe("search_workflows", () => {
    it("should find workflows with pattern", async () => {
      const result = await client.callTool({
        name: "search_workflows",
        arguments: {
          directory: TEST_DIR,
          pattern: "navigate_browser",
        },
      });

      expect(result.content[0].text).toBeDefined();
    });
  });

  describe("validate_workflow", () => {
    it("should validate correct workflow", async () => {
      const filePath = path.join(TEST_DIR, "valid.yml");
      const content = `tool_name: execute_sequence
arguments:
  steps:
    - tool_name: navigate_browser
      arguments:
        url: "https://example.com"
`;

      await client.callTool({
        name: "create_workflow",
        arguments: { file_path: filePath, content },
      });

      const result = await client.callTool({
        name: "validate_workflow",
        arguments: { file_path: filePath },
      });

      expect(result.content[0].text).toContain("✅ Workflow is valid");
    });
  });
});
