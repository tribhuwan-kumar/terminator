#!/usr/bin/env node

/**
 * Sync .cursor/rules/*.mdc files to Claude-compatible format
 * Can be run locally or in CI/CD
 */

const fs = require("fs");
const path = require("path");

function syncRules() {
  const rulesDir = ".cursor/rules";
  const claudeDir = ".claude";

  // Ensure directories exist
  if (!fs.existsSync(rulesDir)) {
    console.log("❌ No .cursor/rules directory found");
    process.exit(1);
  }

  if (!fs.existsSync(claudeDir)) {
    fs.mkdirSync(claudeDir, { recursive: true });
    console.log("📁 Created .claude directory");
  }

  // Read all .mdc rule files
  const ruleFiles = fs
    .readdirSync(rulesDir)
    .filter((file) => file.endsWith(".mdc"))
    .map((file) => {
      const filePath = path.join(rulesDir, file);
      const content = fs.readFileSync(filePath, "utf8");
      const name = file.replace(".mdc", "");

      // Extract title from first line or use filename
      const firstLine = content.split("\n")[0];
      const title = firstLine.startsWith("#")
        ? firstLine.replace(/^#+\s*/, "")
        : name.replace(/-/g, " ").replace(/\b\w/g, (l) => l.toUpperCase());

      return {
        name,
        title,
        path: filePath,
        content,
        size: content.length,
        lines: content.split("\n").length,
      };
    });

  if (ruleFiles.length === 0) {
    console.log("❌ No .mdc rule files found in .cursor/rules");
    process.exit(1);
  }

  // Create Claude rules configuration
  const claudeRules = {
    version: "1.0",
    description:
      "Auto-synced from .cursor/rules - Terminator project workspace rules",
    last_sync: new Date().toISOString(),
    sync_source: ".cursor/rules/*.mdc",
    total_rules: ruleFiles.length,
    rules: {},
  };

  // Process each rule
  ruleFiles.forEach((rule) => {
    claudeRules.rules[rule.name] = {
      title: rule.title,
      content: rule.content,
      source_file: rule.path,
      type: "workspace_rule",
      size_bytes: rule.size,
      line_count: rule.lines,
      last_modified: fs.statSync(rule.path).mtime.toISOString(),
    };
  });

  // Write Claude rules file
  const claudeRulesPath = path.join(claudeDir, "rules.json");
  fs.writeFileSync(claudeRulesPath, JSON.stringify(claudeRules, null, 2));

  // Create human-readable summary
  const summaryPath = path.join(claudeDir, "rules-summary.md");
  const summary = `# Claude Rules Summary

Auto-synced from \`.cursor/rules\` on ${new Date().toLocaleString()}

## Available Rules (${ruleFiles.length} total)

${ruleFiles
  .map(
    (rule) =>
      `### ${rule.title}
- **File**: \`${rule.path}\`
- **Size**: ${rule.size} bytes (${rule.lines} lines)
- **Description**: ${rule.content
        .split("\n")
        .slice(0, 3)
        .join(" ")
        .substring(0, 100)}...`
  )
  .join("\n\n")}

## Usage in Claude

These rules are automatically available when Claude works in this repository. Claude can reference them using the \`fetch_rules\` tool with these keys:

${ruleFiles.map((rule) => `- \`${rule.name}\`: ${rule.title}`).join("\n")}

## Sync Information

- **Total rules synced**: ${ruleFiles.length}
- **Last sync**: ${new Date().toLocaleString()}
- **Source directory**: \`.cursor/rules/\`
- **Target directory**: \`.claude/\`
- **Auto-sync**: Enabled via GitHub Actions on rule changes

## Manual Sync

To manually sync rules, run:
\`\`\`bash
node scripts/sync-cursor-claude-rules.js
\`\`\`
`;

  fs.writeFileSync(summaryPath, summary);

  // Create .gitignore for .claude if needed
  const gitignorePath = path.join(claudeDir, ".gitignore");
  if (!fs.existsSync(gitignorePath)) {
    fs.writeFileSync(
      gitignorePath,
      `# Auto-generated Claude files
*.tmp
*.log
`
    );
  }

  // Output results
  console.log("✅ Successfully synced Cursor rules to Claude format");
  console.log(`📊 Stats:`);
  console.log(`   - Rules processed: ${ruleFiles.length}`);
  console.log(
    `   - Total content: ${ruleFiles.reduce(
      (sum, rule) => sum + rule.size,
      0
    )} bytes`
  );
  console.log(
    `   - Average rule size: ${Math.round(
      ruleFiles.reduce((sum, rule) => sum + rule.size, 0) / ruleFiles.length
    )} bytes`
  );
  console.log(`📁 Files created:`);
  console.log(`   - ${claudeRulesPath}`);
  console.log(`   - ${summaryPath}`);

  return {
    success: true,
    rulesCount: ruleFiles.length,
    files: [claudeRulesPath, summaryPath],
  };
}

// Run if called directly
if (require.main === module) {
  try {
    syncRules();
  } catch (error) {
    console.error("❌ Error syncing rules:", error.message);
    process.exit(1);
  }
}

module.exports = { syncRules };
