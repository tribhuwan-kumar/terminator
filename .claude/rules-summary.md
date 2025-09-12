# Claude Rules Summary

Auto-synced from `.cursor/rules` on 9/12/2025, 12:31:30 AM

## Available Rules (13 total)

### Always Read This
- **File**: `.cursor/rules/Always-read-this.mdc`
- **Size**: 1255 bytes (23 lines)
- **Description**: --- alwaysApply: true ---...

### Documentation Standards
- **File**: `.cursor/rules/documentation-standards.mdc`
- **Size**: 2633 bytes (125 lines)
- **Description**: --- description: If you're writing docs alwaysApply: false...

### Git Operations
- **File**: `.cursor/rules/git-operations.mdc`
- **Size**: 1032 bytes (40 lines)
- **Description**: --- description: "Safe git operations, pushing code, committing changes, git workflow protocols" ---...

### Mcp Debugging Testing
- **File**: `.cursor/rules/mcp-debugging-testing.mdc`
- **Size**: 5121 bytes (174 lines)
- **Description**: --- alwaysApply: true ---...

### Mcp Development Workflow
- **File**: `.cursor/rules/mcp-development-workflow.mdc`
- **Size**: 3482 bytes (137 lines)
- **Description**: --- description: Building MCPs stuff alwaysApply: false...

### Mediar Terminator Overview
- **File**: `.cursor/rules/mediar-terminator-overview.mdc`
- **Size**: 2273 bytes (57 lines)
- **Description**: --- description:  globs: ...

### Output Parser Javascript
- **File**: `.cursor/rules/output-parser-javascript.mdc`
- **Size**: 3644 bytes (121 lines)
- **Description**: --- description: If you're building output parser  alwaysApply: false...

### Pr Preparation Guide
- **File**: `.cursor/rules/pr-preparation-guide.mdc`
- **Size**: 4170 bytes (87 lines)
- **Description**: --- description:  globs: ...

### Terminal Rules
- **File**: `.cursor/rules/terminal-rules.mdc`
- **Size**: 3223 bytes (82 lines)
- **Description**: --- description: globs:...

### Terminator Development Guide
- **File**: `.cursor/rules/terminator-development-guide.mdc`
- **Size**: 5000 bytes (145 lines)
- **Description**: --- description:  globs: ...

### Terminator Project Standards
- **File**: `.cursor/rules/terminator-project-standards.mdc`
- **Size**: 3622 bytes (104 lines)
- **Description**: --- alwaysApply: true ---...

### Workflow Recorder Testing
- **File**: `.cursor/rules/workflow-recorder-testing.mdc`
- **Size**: 4428 bytes (75 lines)
- **Description**: --- description: globs:...

### Workflow Yaml Standards
- **File**: `.cursor/rules/workflow-yaml-standards.mdc`
- **Size**: 4479 bytes (193 lines)
- **Description**: --- description: Building workflows alwaysApply: false...

## Usage in Claude

These rules are automatically available when Claude works in this repository. Claude can reference them using the `fetch_rules` tool with these keys:

- `Always-read-this`: Always Read This
- `documentation-standards`: Documentation Standards
- `git-operations`: Git Operations
- `mcp-debugging-testing`: Mcp Debugging Testing
- `mcp-development-workflow`: Mcp Development Workflow
- `mediar-terminator-overview`: Mediar Terminator Overview
- `output-parser-javascript`: Output Parser Javascript
- `pr-preparation-guide`: Pr Preparation Guide
- `terminal-rules`: Terminal Rules
- `terminator-development-guide`: Terminator Development Guide
- `terminator-project-standards`: Terminator Project Standards
- `workflow-recorder-testing`: Workflow Recorder Testing
- `workflow-yaml-standards`: Workflow Yaml Standards

## Sync Information

- **Total rules synced**: 13
- **Last sync**: 9/12/2025, 12:31:30 AM
- **Source directory**: `.cursor/rules/`
- **Target directory**: `.claude/`
- **Auto-sync**: Enabled via GitHub Actions on rule changes

## Manual Sync

To manually sync rules, run:
```bash
node scripts/sync-cursor-claude-rules.js
```
