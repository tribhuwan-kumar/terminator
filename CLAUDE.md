# Terminator - AI-Native GUI Automation

Open-source desktop automation framework (MIT). Gives AI hands to control any app on Windows/macOS/Linux.
100x faster than generic AI agents, >95% success rate.

**Mediar AI** | [$2.8M seed](https://x.com/louis030195/status/1948745185178914929) | [mediar.ai](https://mediar.ai)

## Packages

```
terminator/                    # Core Rust (terminator-rs)
packages/
  terminator-nodejs/          # @mediar-ai/terminator (npm)
  terminator-python/          # terminator (PyPI)
  workflow/                    # @mediar-ai/workflow (npm)
terminator-cli/               # CLI (version mgmt, workflows)
terminator-mcp-agent/         # MCP server (npm)
terminator-workflow-recorder/ # Record actions → YAML
```

**Current version**: `0.20.6` across all packages

## Release Management

**CRITICAL**: Use `terminator` CLI only. It syncs versions across workspace.

```bash
# Install once
cargo install --path terminator-cli

# Release (most common)
terminator release      # Bump patch → tag → push (triggers CI/CD)

# Manual
terminator status       # Check versions
terminator patch        # Bump 0.20.6 → 0.20.7
terminator sync         # Sync all packages
terminator tag          # Tag + push
```

**Never manually edit versions in package.json or Cargo.toml files.**

Git tag `v0.20.6` triggers:
- `publish-npm.yml` → @mediar-ai/terminator to npm
- `publish-mcp.yml` → terminator-mcp-agent to npm
- `ci-wheels.yml` → Python wheels (manual PyPI publish)

## Development

```bash
# Setup
git clone https://github.com/mediar-ai/terminator
cd terminator
cargo build

# Test
cargo test
cargo fmt && cargo clippy

# Speed up builds (optional)
cargo install sccache
export RUSTC_WRAPPER=sccache
```

## Commit Style

```
type(scope): description

feat(core): add locator strategy
fix(mcp): resolve timeout issue
refactor: rename terminator.js → @mediar-ai/terminator
```

## Package Names

- npm: `@mediar-ai/terminator` (5 platform packages: `-darwin-arm64`, `-darwin-x64`, `-linux-x64-gnu`, `-win32-arm64-msvc`, `-win32-x64-msvc`)
- PyPI: `terminator`
- crates.io: `terminator-rs`

**Recently renamed**: `terminator.js` → `@mediar-ai/terminator` (check old refs if issues)

## MCP Debugging

Logs: `%LOCALAPPDATA%\claude-cli-nodejs\Cache\*\mcp-logs-terminator-mcp-agent\*.txt` (Windows)

```json
{
  "mcpServers": {
    "terminator-mcp-agent": {
      "env": {
        "LOG_LEVEL": "debug",
        "RUST_BACKTRACE": "1"
      }
    }
  }
}
```

## Repo Rules

- ❌ NO dead code, redundant files, verbose docs
- ✅ Ask before creating files
- ✅ Clean up before commits
- ✅ Keep it high signal
