# Terminator Scripts

Utility scripts for the Terminator project.

## Cursor Rules Sync

### `sync-cursor-claude-rules.js`

Automatically syncs `.cursor/rules/*.mdc` files to Claude-compatible format in `.claude/rules.json`.

**Usage:**

```bash
# Manual sync
node scripts/sync-cursor-claude-rules.js

# Or from any directory
node /path/to/terminator/scripts/sync-cursor-claude-rules.js
```

**What it does:**

- Reads all `.cursor/rules/*.mdc` files
- Converts them to Claude-compatible JSON format
- Creates `.claude/rules.json` and `.claude/rules-summary.md`
- Provides detailed statistics and file information

**Auto-sync:**

- GitHub Actions automatically runs this script when `.cursor/rules/*.mdc` files change
- No manual intervention needed for rule synchronization

**Output files:**

- `.claude/rules.json` - Machine-readable rules for Claude
- `.claude/rules-summary.md` - Human-readable summary
- `.claude/.gitignore` - Ignores temporary files

**Requirements:**

- Node.js (built-in modules only, no external dependencies)
- `.cursor/rules/` directory with `.mdc` files
