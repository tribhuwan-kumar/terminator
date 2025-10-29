# Workflow Compatibility Testing Plan

## Overview

Rigorous test suite to ensure backward compatibility with YAML workflows and forward compatibility with TypeScript workflows.

## Test Categories

### 1. **YAML Backward Compatibility Tests** ✅

**Purpose:** Ensure existing YAML workflows continue to work without any changes.

| Test | Description | Verifies |
|------|-------------|----------|
| `test_yaml_basic_execution` | Single step YAML workflow | Basic YAML execution |
| `test_yaml_multiple_steps` | 3-step sequential workflow | Multi-step YAML |
| `test_yaml_with_variables` | Workflow with variable substitution | Variable system |
| `test_yaml_start_from_step` | Resume from step 2 | State restoration |
| `test_yaml_end_at_step` | Stop after step 2 | Partial execution |
| `test_yaml_state_persistence` | State file creation | State caching |

**Coverage:** All existing YAML features must pass.

---

### 2. **TypeScript Workflow Tests** ✅

**Purpose:** Ensure new TypeScript workflows work correctly.

| Test | Description | Verifies |
|------|-------------|----------|
| `test_ts_basic_execution` | Simple 3-step TS workflow | Basic TS execution |
| `test_ts_start_from_step` | Resume from step 2 | State restoration |
| `test_ts_end_at_step` | Stop after step 2 | Partial execution |
| `test_ts_state_persistence` | State file creation | State caching |
| `test_ts_context_sharing` | Data sharing between steps | Context mechanism |
| `test_ts_metadata_extraction` | `.getMetadata()` call | Visualization data |

**Coverage:** All core TS features must work.

---

### 3. **Cross-Format Compatibility Tests** ✅

**Purpose:** Ensure YAML and TS workflows can coexist.

| Test | Description | Verifies |
|------|-------------|----------|
| `test_yaml_then_ts_workflow` | Run YAML, then TS workflow | No interference |
| `test_format_detection_yaml_file` | Detect `.yml` file | Format detection |
| `test_format_detection_ts_file` | Detect `.ts` file | Format detection |
| `test_format_detection_ts_project` | Detect TS project directory | Format detection |

**Coverage:** Both formats work independently in same session.

---

### 4. **Runtime Detection Tests** ✅

**Purpose:** Ensure bun/node fallback works correctly.

| Test | Description | Verifies |
|------|-------------|----------|
| `test_bun_runtime_detection` | Detect available runtime | Runtime detection |
| `test_ts_execution_with_bun` | Execute TS workflow with bun | Bun execution |
| `test_ts_execution_with_node` | Execute TS workflow with node | Node fallback |

**Coverage:** Both runtimes work, bun preferred.

---

### 5. **Error Handling Tests** ✅

**Purpose:** Ensure errors are handled gracefully.

| Test | Description | Verifies |
|------|-------------|----------|
| `test_yaml_invalid_step` | YAML with nonexistent tool | Error detection |
| `test_ts_workflow_error_handling` | TS step throws error | Error propagation |
| `test_missing_start_step` | Invalid start_from_step | Error validation |

**Coverage:** Proper error messages and recovery.

---

## Test Execution

### Running Tests

```bash
# Run all tests
cd terminator-mcp-agent
cargo test --test test_workflow_compatibility

# Run specific category
cargo test --test test_workflow_compatibility yaml_
cargo test --test test_workflow_compatibility ts_

# Run with output
cargo test --test test_workflow_compatibility -- --nocapture

# Run single test
cargo test --test test_workflow_compatibility test_yaml_basic_execution
```

### Prerequisites

```bash
# Install required runtimes
bun --version  # Optional (will fallback to node)
node --version # Required

# Install tsx for node TypeScript support
npm install -g tsx

# Install test dependencies
cd terminator-mcp-agent
cargo build --tests
```

---

## Test Coverage Matrix

| Feature | YAML Tests | TS Tests | Status |
|---------|------------|----------|--------|
| **Basic Execution** | ✅ `test_yaml_basic_execution` | ✅ `test_ts_basic_execution` | Complete |
| **Multi-Step** | ✅ `test_yaml_multiple_steps` | ✅ (in basic test) | Complete |
| **State Caching** | ✅ `test_yaml_state_persistence` | ✅ `test_ts_state_persistence` | Complete |
| **Start From Step** | ✅ `test_yaml_start_from_step` | ✅ `test_ts_start_from_step` | Complete |
| **End At Step** | ✅ `test_yaml_end_at_step` | ✅ `test_ts_end_at_step` | Complete |
| **Variables/Inputs** | ✅ `test_yaml_with_variables` | ✅ (Zod schema) | Complete |
| **Context Sharing** | ✅ (env vars) | ✅ `test_ts_context_sharing` | Complete |
| **Metadata** | N/A | ✅ `test_ts_metadata_extraction` | Complete |
| **Error Handling** | ✅ `test_yaml_invalid_step` | ✅ `test_ts_workflow_error_handling` | Complete |
| **Format Detection** | ✅ `test_format_detection_yaml_file` | ✅ `test_format_detection_ts_*` | Complete |
| **Runtime Detection** | N/A | ✅ `test_bun_runtime_detection` | Complete |

**Total Tests:** 21
**YAML-specific:** 6
**TS-specific:** 8
**Cross-format:** 4
**Runtime:** 3

---

## Critical Path Tests

These tests MUST pass before merging:

### Critical for YAML Backward Compatibility
1. ✅ `test_yaml_basic_execution` - Core YAML execution
2. ✅ `test_yaml_state_persistence` - State caching (existing feature)
3. ✅ `test_yaml_start_from_step` - Resume functionality (existing feature)

### Critical for TS Forward Compatibility
1. ✅ `test_ts_basic_execution` - Core TS execution
2. ✅ `test_ts_state_persistence` - State caching (new implementation)
3. ✅ `test_ts_start_from_step` - Resume functionality (new implementation)
4. ✅ `test_ts_metadata_extraction` - Visualization support

### Critical for Coexistence
1. ✅ `test_yaml_then_ts_workflow` - No interference between formats
2. ✅ `test_format_detection_yaml_file` - Correct YAML detection
3. ✅ `test_format_detection_ts_project` - Correct TS detection

---

## Test Fixtures

### YAML Test Workflow Structure
```yaml
# Basic 3-step workflow
steps:
  - id: step1
    name: Step 1
    tool_name: run_command
    arguments:
      run: echo "Step 1"

  - id: step2
    name: Step 2
    tool_name: run_command
    arguments:
      run: echo "Step 2"

  - id: step3
    name: Step 3
    tool_name: run_command
    arguments:
      run: echo "Step 3"
```

### TypeScript Test Workflow Structure
```typescript
// packages/terminator-workflow test fixture
import { createStep, createWorkflow } from '@mediar/terminator-workflow';
import { z } from 'zod';

const step1 = createStep({
  id: 'step1',
  name: 'Step 1',
  execute: async ({ logger, context }) => {
    logger.info('Executing step 1');
    context.data.step1 = { executed: true };
    return { result: 'step1 complete' };
  },
});

const step2 = createStep({
  id: 'step2',
  name: 'Step 2',
  execute: async ({ logger, context }) => {
    logger.info('Executing step 2');
    context.data.step2 = {
      executed: true,
      fromStep1: context.data.step1  // Context sharing
    };
    return { result: 'step2 complete' };
  },
});

const step3 = createStep({
  id: 'step3',
  name: 'Step 3',
  execute: async ({ logger, context }) => {
    logger.info('Executing step 3');
    context.data.step3 = { executed: true };
    return { result: 'step3 complete' };
  },
});

export default createWorkflow({
  name: 'Test Workflow',
  version: '1.0.0',
  input: z.object({
    testInput: z.string().default('test'),
  }),
})
  .step(step1)
  .step(step2)
  .step(step3)
  .build();
```

---

## State File Validation

Tests verify state files have correct structure:

### YAML State File
```json
{
  "last_updated": "2025-01-29T10:30:00Z",
  "last_step_id": "step2",
  "last_step_index": 1,
  "workflow_file": "workflow.yml",
  "env": {
    "step1_result": { "output": "Step 1" },
    "step1_status": "success",
    "step2_result": { "output": "Step 2" },
    "step2_status": "success"
  }
}
```

### TypeScript State File
```json
{
  "last_updated": "2025-01-29T10:30:00Z",
  "last_step_id": "step2",
  "last_step_index": 1,
  "workflow_file": "workflow.ts",
  "env": {
    "context": {
      "data": {
        "step1": { "executed": true },
        "step2": { "executed": true, "fromStep1": {...} }
      },
      "state": {},
      "variables": { "testInput": "test" }
    },
    "stepResults": {
      "step1": { "status": "success", "result": {...} },
      "step2": { "status": "success", "result": {...} }
    }
  }
}
```

---

## Continuous Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/test-workflows.yml
name: Workflow Compatibility Tests

on:
  push:
    branches: [main, feature/javascript-workflows]
  pull_request:
    branches: [main]

jobs:
  test-backward-compatibility:
    name: YAML Backward Compatibility
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run YAML tests
        run: |
          cd terminator-mcp-agent
          cargo test --test test_workflow_compatibility yaml_

  test-typescript-workflows:
    name: TypeScript Workflow Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: oven-sh/setup-bun@v1
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install tsx
        run: npm install -g tsx
      - name: Run TS tests
        run: |
          cd terminator-mcp-agent
          cargo test --test test_workflow_compatibility ts_

  test-cross-format:
    name: Cross-Format Compatibility
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: oven-sh/setup-bun@v1
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run all tests
        run: |
          cd terminator-mcp-agent
          cargo test --test test_workflow_compatibility
```

---

## Manual Testing Checklist

Before release, manually verify:

### YAML Workflows
- [ ] Existing production workflow runs without changes
- [ ] State files are created in `.workflow_state/`
- [ ] Resume from middle step works
- [ ] Stop at specific step works
- [ ] Variables are substituted correctly
- [ ] Error handling works as before

### TypeScript Workflows
- [ ] Example workflows in `examples/typescript-workflow/` run
- [ ] State files are created in `.workflow_state/`
- [ ] Resume from middle step works
- [ ] Stop at specific step works
- [ ] Context sharing between steps works
- [ ] Metadata extraction works for UI

### Runtime
- [ ] Works with bun installed
- [ ] Works with only node installed
- [ ] Fallback from bun to node works if bun fails

### Desktop App Integration
- [ ] YAML workflows appear in workflow list
- [ ] TS workflows appear in workflow list
- [ ] Start/stop buttons work for both formats
- [ ] Workflow visualization works for TS workflows
- [ ] State restoration works after app restart

---

## Regression Testing

After each change, run regression suite:

```bash
# Full regression suite
./scripts/test-regression.sh

# Or manual:
cargo test --test test_workflow_compatibility -- --test-threads=1
```

**Regression must pass 100% before merge.**

---

## Performance Benchmarks

Track performance to ensure TS workflows aren't slower:

| Metric | YAML | TypeScript | Acceptable |
|--------|------|------------|------------|
| Startup (load metadata) | <100ms | <500ms | ✅ |
| Single step execution | <50ms | <100ms | ✅ |
| State save | <50ms | <50ms | ✅ |
| State load | <50ms | <50ms | ✅ |
| 10-step workflow | <1s | <2s | ✅ |

---

## Summary

**Test Coverage:** 21 comprehensive tests
**Critical Path:** 10 must-pass tests
**Backward Compatibility:** 6 YAML-specific tests
**Forward Compatibility:** 8 TS-specific tests
**Runtime:** Bun priority with Node fallback

**All tests must pass before merging PR #318.**
