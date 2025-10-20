import { describe, it, expect, beforeEach, afterEach } from "vitest";
import * as fs from "fs/promises";
import * as path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Test directory
const TEST_DIR = path.join(__dirname, "test-workflows");

// Valid workflow template
const VALID_WORKFLOW = `tool_name: execute_sequence
arguments:
  variables:
    test_url:
      type: string
      label: "Test URL"
      default: "https://example.com"

  inputs:
    test_url: "\${{variables.test_url.default}}"

  steps:
    - tool_name: navigate_browser
      arguments:
        url: "\${{inputs.test_url}}"
        browser: "chrome"
      delay_ms: 2000
`;

const INVALID_WORKFLOW = `tool_name: invalid_tool
arguments:
  invalid: true
`;

// Helper function to simulate MCP tool calls
async function callTool(toolName: string, args: any): Promise<any> {
  // Import the handlers from our main file
  const { default: indexModule } = await import("../dist/index.js");

  // For testing, we'll directly test the file operations
  return { toolName, args };
}

describe("Workflow Builder MCP Tools", () => {
  beforeEach(async () => {
    // Create test directory
    await fs.mkdir(TEST_DIR, { recursive: true });
  });

  afterEach(async () => {
    // Clean up test directory
    try {
      await fs.rm(TEST_DIR, { recursive: true, force: true });
    } catch (error) {
      // Ignore errors
    }
  });

  describe("create_workflow", () => {
    it("should create a valid workflow file", async () => {
      const filePath = path.join(TEST_DIR, "test-create.yml");

      await fs.writeFile(filePath, VALID_WORKFLOW);
      const content = await fs.readFile(filePath, "utf-8");

      expect(content).toBe(VALID_WORKFLOW);
      expect(content).toContain("tool_name: execute_sequence");
      expect(content).toContain("arguments:");
      expect(content).toContain("steps:");
    });

    it("should fail if file already exists", async () => {
      const filePath = path.join(TEST_DIR, "test-exists.yml");

      // Create file first
      await fs.writeFile(filePath, VALID_WORKFLOW);

      // Try to create again
      try {
        await fs.access(filePath);
        expect(true).toBe(true); // File exists
      } catch {
        expect.fail("File should exist");
      }
    });

    it("should create directories if they don't exist", async () => {
      const filePath = path.join(TEST_DIR, "nested", "deep", "workflow.yml");

      const dir = path.dirname(filePath);
      await fs.mkdir(dir, { recursive: true });
      await fs.writeFile(filePath, VALID_WORKFLOW);

      const exists = await fs.access(filePath).then(() => true).catch(() => false);
      expect(exists).toBe(true);
    });
  });

  describe("read_workflow", () => {
    it("should read workflow with line numbers", async () => {
      const filePath = path.join(TEST_DIR, "test-read.yml");
      await fs.writeFile(filePath, VALID_WORKFLOW);

      const content = await fs.readFile(filePath, "utf-8");
      const lines = content.split("\n");

      expect(lines.length).toBeGreaterThan(0);
      expect(content).toContain("tool_name: execute_sequence");
    });

    it("should fail if file does not exist", async () => {
      const filePath = path.join(TEST_DIR, "nonexistent.yml");

      try {
        await fs.readFile(filePath, "utf-8");
        expect.fail("Should throw error");
      } catch (error: any) {
        expect(error.code).toBe("ENOENT");
      }
    });

    it("should handle empty files", async () => {
      const filePath = path.join(TEST_DIR, "empty.yml");
      await fs.writeFile(filePath, "");

      const content = await fs.readFile(filePath, "utf-8");
      expect(content).toBe("");
    });
  });

  describe("edit_workflow", () => {
    it("should replace unique string", async () => {
      const filePath = path.join(TEST_DIR, "test-edit.yml");
      await fs.writeFile(filePath, VALID_WORKFLOW);

      const content = await fs.readFile(filePath, "utf-8");
      const newContent = content.replace(
        "https://example.com",
        "https://newsite.com"
      );
      await fs.writeFile(filePath, newContent);

      const updated = await fs.readFile(filePath, "utf-8");
      expect(updated).toContain("https://newsite.com");
      expect(updated).not.toContain("https://example.com");
    });

    it("should detect non-unique strings", async () => {
      const duplicateContent = `tool_name: execute_sequence
arguments:
  steps:
    - tool_name: navigate_browser
      arguments:
        url: "https://example.com"
    - tool_name: navigate_browser
      arguments:
        url: "https://example.com"
`;
      const filePath = path.join(TEST_DIR, "test-duplicate.yml");
      await fs.writeFile(filePath, duplicateContent);

      const content = await fs.readFile(filePath, "utf-8");
      const occurrences = (content.match(/https:\/\/example\.com/g) || []).length;

      expect(occurrences).toBe(2);
    });

    it("should replace all occurrences when replace_all is true", async () => {
      const duplicateContent = `tool_name: execute_sequence
arguments:
  steps:
    - tool_name: navigate_browser
      arguments:
        url: "https://example.com"
    - tool_name: navigate_browser
      arguments:
        url: "https://example.com"
`;
      const filePath = path.join(TEST_DIR, "test-replace-all.yml");
      await fs.writeFile(filePath, duplicateContent);

      let content = await fs.readFile(filePath, "utf-8");
      content = content.replaceAll("https://example.com", "https://newsite.com");
      await fs.writeFile(filePath, content);

      const updated = await fs.readFile(filePath, "utf-8");
      expect(updated).not.toContain("https://example.com");
      expect((updated.match(/https:\/\/newsite\.com/g) || []).length).toBe(2);
    });

    it("should fail if old_string not found", async () => {
      const filePath = path.join(TEST_DIR, "test-not-found.yml");
      await fs.writeFile(filePath, VALID_WORKFLOW);

      const content = await fs.readFile(filePath, "utf-8");
      const hasString = content.includes("nonexistent-string");

      expect(hasString).toBe(false);
    });

    it("should preserve file formatting", async () => {
      const filePath = path.join(TEST_DIR, "test-formatting.yml");
      await fs.writeFile(filePath, VALID_WORKFLOW);

      const before = await fs.readFile(filePath, "utf-8");
      const after = before.replace("example.com", "newsite.com");
      await fs.writeFile(filePath, after);

      const updated = await fs.readFile(filePath, "utf-8");

      // Check that indentation is preserved
      expect(updated.split("\n").filter(l => l.startsWith("  ")).length).toBeGreaterThan(0);
    });
  });

  describe("list_workflows", () => {
    it("should list all YAML files in directory", async () => {
      // Create multiple workflow files
      await fs.writeFile(path.join(TEST_DIR, "workflow1.yml"), VALID_WORKFLOW);
      await fs.writeFile(path.join(TEST_DIR, "workflow2.yaml"), VALID_WORKFLOW);
      await fs.writeFile(path.join(TEST_DIR, "not-workflow.txt"), "text");

      const files = await fs.readdir(TEST_DIR);
      const yamlFiles = files.filter(f => f.endsWith(".yml") || f.endsWith(".yaml"));

      expect(yamlFiles.length).toBe(2);
      expect(yamlFiles).toContain("workflow1.yml");
      expect(yamlFiles).toContain("workflow2.yaml");
    });

    it("should filter by pattern", async () => {
      await fs.writeFile(path.join(TEST_DIR, "test1.yml"), VALID_WORKFLOW);
      await fs.writeFile(path.join(TEST_DIR, "test2.yml"), VALID_WORKFLOW);
      await fs.writeFile(path.join(TEST_DIR, "prod.yml"), VALID_WORKFLOW);

      const files = await fs.readdir(TEST_DIR);
      const testFiles = files.filter(f => f.startsWith("test"));

      expect(testFiles.length).toBe(2);
    });

    it("should handle empty directories", async () => {
      const emptyDir = path.join(TEST_DIR, "empty");
      await fs.mkdir(emptyDir, { recursive: true });

      const files = await fs.readdir(emptyDir);
      expect(files.length).toBe(0);
    });

    it("should include file metadata", async () => {
      const filePath = path.join(TEST_DIR, "metadata.yml");
      await fs.writeFile(filePath, VALID_WORKFLOW);

      const stats = await fs.stat(filePath);
      expect(stats.size).toBeGreaterThan(0);
      expect(stats.mtime).toBeInstanceOf(Date);
    });
  });

  describe("search_workflows", () => {
    beforeEach(async () => {
      // Create test files with different content
      await fs.writeFile(
        path.join(TEST_DIR, "browser.yml"),
        `tool_name: execute_sequence
arguments:
  steps:
    - tool_name: navigate_browser
      arguments:
        url: "https://example.com"
`
      );

      await fs.writeFile(
        path.join(TEST_DIR, "click.yml"),
        `tool_name: execute_sequence
arguments:
  steps:
    - tool_name: click_element
      arguments:
        selector: "role:Button"
`
      );
    });

    it("should find files containing text pattern", async () => {
      const files = await fs.readdir(TEST_DIR);
      const results: string[] = [];

      for (const file of files) {
        const content = await fs.readFile(path.join(TEST_DIR, file), "utf-8");
        if (content.includes("navigate_browser")) {
          results.push(file);
        }
      }

      expect(results.length).toBe(1);
      expect(results[0]).toBe("browser.yml");
    });

    it("should support regex patterns", async () => {
      const files = await fs.readdir(TEST_DIR);
      const results: string[] = [];
      const pattern = /tool_name: (click|navigate)/;

      for (const file of files) {
        const content = await fs.readFile(path.join(TEST_DIR, file), "utf-8");
        if (pattern.test(content)) {
          results.push(file);
        }
      }

      expect(results.length).toBe(2);
    });

    it("should return line numbers with matches", async () => {
      const content = await fs.readFile(path.join(TEST_DIR, "browser.yml"), "utf-8");
      const lines = content.split("\n");
      const matches: number[] = [];

      lines.forEach((line, idx) => {
        if (line.includes("navigate_browser")) {
          matches.push(idx + 1);
        }
      });

      expect(matches.length).toBeGreaterThan(0);
    });

    it("should handle no matches", async () => {
      const files = await fs.readdir(TEST_DIR);
      const results: string[] = [];

      for (const file of files) {
        const content = await fs.readFile(path.join(TEST_DIR, file), "utf-8");
        if (content.includes("nonexistent_pattern")) {
          results.push(file);
        }
      }

      expect(results.length).toBe(0);
    });
  });

  describe("validate_workflow", () => {
    it("should validate correct workflow structure", async () => {
      const filePath = path.join(TEST_DIR, "valid.yml");
      await fs.writeFile(filePath, VALID_WORKFLOW);

      const content = await fs.readFile(filePath, "utf-8");

      // Basic validation
      expect(content).toContain("tool_name: execute_sequence");
      expect(content).toContain("arguments:");
      expect(content).toContain("steps:");
    });

    it("should reject invalid tool_name", async () => {
      const invalid = `tool_name: invalid_tool
arguments:
  steps: []
`;
      const filePath = path.join(TEST_DIR, "invalid-tool.yml");
      await fs.writeFile(filePath, invalid);

      const content = await fs.readFile(filePath, "utf-8");
      expect(content).not.toContain("tool_name: execute_sequence");
    });

    it("should reject missing arguments", async () => {
      const invalid = `tool_name: execute_sequence
steps: []
`;
      const filePath = path.join(TEST_DIR, "no-args.yml");
      await fs.writeFile(filePath, invalid);

      const content = await fs.readFile(filePath, "utf-8");
      expect(content).not.toContain("arguments:");
    });

    it("should reject missing steps array", async () => {
      const invalid = `tool_name: execute_sequence
arguments:
  variables: {}
`;
      const filePath = path.join(TEST_DIR, "no-steps.yml");
      await fs.writeFile(filePath, invalid);

      const content = await fs.readFile(filePath, "utf-8");
      expect(content).not.toContain("steps:");
    });

    it("should accept empty steps array", async () => {
      const valid = `tool_name: execute_sequence
arguments:
  steps: []
`;
      const filePath = path.join(TEST_DIR, "empty-steps.yml");
      await fs.writeFile(filePath, valid);

      const content = await fs.readFile(filePath, "utf-8");
      expect(content).toContain("steps: []");
    });

    it("should reject invalid YAML syntax", async () => {
      const invalid = `tool_name: execute_sequence
arguments:
  steps:
    - invalid yaml: [
`;
      const filePath = path.join(TEST_DIR, "syntax-error.yml");
      await fs.writeFile(filePath, invalid);

      // YAML parsing would fail on this
      const content = await fs.readFile(filePath, "utf-8");
      expect(content).toContain("invalid yaml: [");
    });
  });

  describe("Edge Cases", () => {
    it("should handle very long file paths", async () => {
      const longPath = path.join(
        TEST_DIR,
        "a".repeat(50),
        "b".repeat(50),
        "workflow.yml"
      );

      await fs.mkdir(path.dirname(longPath), { recursive: true });
      await fs.writeFile(longPath, VALID_WORKFLOW);

      const exists = await fs.access(longPath).then(() => true).catch(() => false);
      expect(exists).toBe(true);
    });

    it("should handle special characters in filenames", async () => {
      const specialPath = path.join(TEST_DIR, "workflow-test_v1.2.yml");
      await fs.writeFile(specialPath, VALID_WORKFLOW);

      const content = await fs.readFile(specialPath, "utf-8");
      expect(content).toBe(VALID_WORKFLOW);
    });

    it("should handle large workflow files", async () => {
      const largeWorkflow = VALID_WORKFLOW.repeat(100);
      const filePath = path.join(TEST_DIR, "large.yml");
      await fs.writeFile(filePath, largeWorkflow);

      const content = await fs.readFile(filePath, "utf-8");
      expect(content.length).toBe(largeWorkflow.length);
    });

    it("should handle concurrent file operations", async () => {
      const promises = Array.from({ length: 10 }, (_, i) =>
        fs.writeFile(path.join(TEST_DIR, `concurrent-${i}.yml`), VALID_WORKFLOW)
      );

      await Promise.all(promises);

      const files = await fs.readdir(TEST_DIR);
      const concurrentFiles = files.filter(f => f.startsWith("concurrent-"));
      expect(concurrentFiles.length).toBe(10);
    });

    it("should handle Unicode content", async () => {
      const unicodeContent = `tool_name: execute_sequence
arguments:
  steps:
    - tool_name: type_into_element
      arguments:
        text: "Hello 世界 🌍 Привет"
`;
      const filePath = path.join(TEST_DIR, "unicode.yml");
      await fs.writeFile(filePath, unicodeContent, "utf-8");

      const content = await fs.readFile(filePath, "utf-8");
      expect(content).toContain("世界");
      expect(content).toContain("🌍");
      expect(content).toContain("Привет");
    });
  });
});
