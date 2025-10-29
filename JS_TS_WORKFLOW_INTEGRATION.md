# JavaScript/TypeScript Workflow Integration with execute_sequence

## Overview

This document outlines how to integrate the new TypeScript workflow SDK with the existing `execute_sequence` Rust implementation, ensuring backward compatibility with YAML workflows while adding support for JS/TS projects.

## Requirements

1. ✅ **execute_sequence can run JS/TS projects** instead of YAML
2. ✅ **Desktop app: start/stop from/at any step** like before
3. ✅ **Development/debugging: caching state** for development use case
4. ✅ **Desktop app: visualization** of steps/workflow/loops/conditions
5. ✅ **Backward compatibility** with YAML workflows

---

## Architecture Design

### 1. Project Detection Strategy

**Goal:** Detect whether `url` parameter points to a YAML workflow or JS/TS project.

```rust
// In server_sequence.rs, around line 282

pub enum WorkflowFormat {
    Yaml,           // Traditional YAML workflow
    TypeScript,     // TypeScript project with workflow.ts
}

async fn detect_workflow_format(url: &str) -> Result<WorkflowFormat, McpError> {
    if url.starts_with("file://") {
        let path = url.strip_prefix("file://").unwrap_or(url);
        let path_obj = Path::new(path);

        // Check if it's a directory
        if path_obj.is_dir() {
            // Look for package.json AND workflow.ts/index.ts
            let package_json = path_obj.join("package.json");
            let workflow_ts = path_obj.join("workflow.ts");
            let index_ts = path_obj.join("index.ts");

            if package_json.exists() && (workflow_ts.exists() || index_ts.exists()) {
                return Ok(WorkflowFormat::TypeScript);
            }
        } else if path_obj.is_file() {
            // Check file extension
            if let Some(ext) = path_obj.extension() {
                match ext.to_str() {
                    Some("ts") | Some("js") => return Ok(WorkflowFormat::TypeScript),
                    Some("yml") | Some("yaml") => return Ok(WorkflowFormat::Yaml),
                    _ => {}
                }
            }
        }
    }

    // Default to YAML for backward compatibility
    Ok(WorkflowFormat::Yaml)
}
```

**Detection Logic:**

| URL Pattern | Detection | Example |
|------------|-----------|---------|
| `file://path/to/workflow.yml` | File extension `.yml` or `.yaml` | YAML workflow |
| `file://path/to/workflow.ts` | File extension `.ts` or `.js` | TypeScript workflow file |
| `file://path/to/project/` | Directory with `package.json` + `workflow.ts` | TypeScript project |
| `https://example.com/workflow.yml` | Remote YAML (existing) | YAML workflow |

---

### 2. TypeScript Workflow Direct Execution

**Goal:** Execute TypeScript workflows directly using Node.js, bypassing the need for step-by-step conversion.

**Key Insight:** Instead of converting TS workflows to `ExecuteSequenceArgs` and executing each step via MCP tools, we execute the entire workflow directly in Node.js and manage state externally.

```rust
// New file: terminator-mcp-agent/src/workflow_typescript.rs

use serde_json::Value;
use std::process::Command;
use std::path::{Path, PathBuf};

pub enum JsRuntime {
    Bun,
    Node,
}

/// Detect available JavaScript runtime (prefer bun, fallback to node)
fn detect_js_runtime() -> JsRuntime {
    // Try bun first
    if Command::new("bun").arg("--version").output().is_ok() {
        info!("Using bun runtime");
        return JsRuntime::Bun;
    }

    // Fallback to node
    info!("Bun not found, using node runtime");
    JsRuntime::Node
}

pub struct TypeScriptWorkflow {
    workflow_path: PathBuf,
    entry_file: String,  // "workflow.ts" or "index.ts"
}

impl TypeScriptWorkflow {
    pub fn new(workflow_path: PathBuf, entry_file: String) -> Self {
        Self { workflow_path, entry_file }
    }

    /// Execute the entire TypeScript workflow with state management
    pub async fn execute(
        &self,
        inputs: Value,
        start_from_step: Option<&str>,
        end_at_step: Option<&str>,
        restored_state: Option<Value>,
    ) -> Result<TypeScriptWorkflowResult, McpError> {
        // Create execution script
        let exec_script = format!(r#"
            import {{ createWorkflowRunner }} from '@mediar/terminator-workflow/runner';

            const workflow = await import('file://{}/{}');

            const runner = createWorkflowRunner({{
                workflow: workflow.default,
                inputs: {},
                startFromStep: {},
                endAtStep: {},
                restoredState: {},
            }});

            const result = await runner.run();

            // Output: metadata + execution result + state
            console.log(JSON.stringify({{
                metadata: workflow.default.getMetadata(),
                result: result,
                state: runner.getState(),
            }}));
        "#,
            self.workflow_path.display(),
            self.entry_file,
            serde_json::to_string(&inputs).unwrap(),
            start_from_step.map(|s| format!("'{}'", s)).unwrap_or("null".to_string()),
            end_at_step.map(|s| format!("'{}'", s)).unwrap_or("null".to_string()),
            restored_state.map(|s| serde_json::to_string(&s).unwrap()).unwrap_or("null".to_string()),
        );

        // Execute via bun (priority) or node (fallback)
        let runtime = detect_js_runtime();
        let output = match runtime {
            JsRuntime::Bun => {
                Command::new("bun")
                    .current_dir(&self.workflow_path)
                    .args(&["--eval", &exec_script])
                    .output()
                    .map_err(|e| McpError::internal_error(format!("Failed to execute workflow with bun: {}", e), None))?
            }
            JsRuntime::Node => {
                Command::new("node")
                    .current_dir(&self.workflow_path)
                    .args(&["--import", "tsx/esm", "--eval", &exec_script])
                    .output()
                    .map_err(|e| McpError::internal_error(format!("Failed to execute workflow with node: {}", e), None))?
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(McpError::internal_error(
                format!("Workflow execution failed: {}", stderr),
                None
            ));
        }

        // Parse result
        let result_json = String::from_utf8_lossy(&output.stdout);
        let result: TypeScriptWorkflowResult = serde_json::from_str(&result_json)
            .map_err(|e| McpError::internal_error(format!("Invalid result: {}", e), None))?;

        Ok(result)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TypeScriptWorkflowResult {
    pub metadata: WorkflowMetadata,
    pub result: WorkflowExecutionResult,
    pub state: Value,  // Serializable state for caching
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkflowMetadata {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub input: Value,  // Zod schema (serialized)
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
    pub status: String,  // "success" | "error"
    pub last_step_id: Option<String>,
    pub last_step_index: usize,
    pub error: Option<String>,
}
```

---

### 3. State Caching for TypeScript Workflows

**Goal:** Reuse existing `.workflow_state` mechanism for TS workflows.

**Simplified Approach:** The TS workflow runner manages state internally, and we save/load it externally.

```rust
// In server_sequence.rs - modify to handle TypeScript workflows

async fn execute_sequence_impl(&self, args: ExecuteSequenceArgs) -> Result<Value, McpError> {
    // Detect format
    let format = if let Some(url) = &args.url {
        detect_workflow_format(url).await?
    } else {
        WorkflowFormat::Yaml
    };

    match format {
        WorkflowFormat::Yaml => {
            // Existing YAML execution (no changes)
            self.execute_yaml_workflow(args).await
        }
        WorkflowFormat::TypeScript => {
            // NEW: TypeScript execution
            self.execute_typescript_workflow(args).await
        }
    }
}

async fn execute_typescript_workflow(&self, args: ExecuteSequenceArgs) -> Result<Value, McpError> {
    let url = args.url.as_ref().unwrap();
    let workflow_path = extract_file_path(url)?;

    // Load saved state if resuming
    let restored_state = if args.start_from_step.is_some() {
        load_workflow_state(url).await?
    } else {
        None
    };

    // Execute TypeScript workflow
    let ts_workflow = TypeScriptWorkflow::new(
        workflow_path.clone(),
        "workflow.ts".to_string()
    );

    let result = ts_workflow.execute(
        args.inputs.unwrap_or(json!({})),
        args.start_from_step.as_deref(),
        args.end_at_step.as_deref(),
        restored_state,
    ).await?;

    // Save state for resumption
    if result.result.last_step_id.is_some() {
        save_workflow_state(
            url,
            result.result.last_step_id.as_deref(),
            result.result.last_step_index,
            &result.state,
        ).await?;
    }

    Ok(json!({
        "status": result.result.status,
        "metadata": result.metadata,
        "state": result.state,
    }))
}
```

**State File Structure (Same as YAML):**

```json
{
  "last_updated": "2025-01-15T10:30:00Z",
  "last_step_id": "fill-sap-form",
  "last_step_index": 2,
  "workflow_file": "workflow.ts",
  "env": {
    "context": {
      "data": { /* shared data */ },
      "state": { /* step state */ },
      "variables": { /* inputs */ }
    },
    "stepResults": {
      "read-json": { "status": "success", "result": {...} },
      "check-duplicates": { "status": "success" }
    }
  }
}
```

---

### 4. Start/Stop at Any Step

**Goal:** Implement start/stop functionality directly in TypeScript workflow runner.

**Approach:** Add a `WorkflowRunner` class that handles step execution with start/stop logic.

```typescript
// New file: packages/terminator-workflow/src/runner.ts

import { Workflow, WorkflowContext } from './types';
import { Desktop } from 'terminator.js';

export interface WorkflowRunnerOptions {
  workflow: Workflow;
  inputs: any;
  startFromStep?: string;
  endAtStep?: string;
  restoredState?: any;
}

export interface WorkflowState {
  context: WorkflowContext;
  stepResults: Record<string, { status: string; result?: any; error?: string }>;
  lastStepId?: string;
  lastStepIndex: number;
}

export class WorkflowRunner {
  private workflow: Workflow;
  private inputs: any;
  private startFromStep?: string;
  private endAtStep?: string;
  private state: WorkflowState;
  private desktop: Desktop;

  constructor(options: WorkflowRunnerOptions) {
    this.workflow = options.workflow;
    this.inputs = options.inputs;
    this.startFromStep = options.startFromStep;
    this.endAtStep = options.endAtStep;

    // Initialize or restore state
    if (options.restoredState) {
      this.state = options.restoredState;
    } else {
      this.state = {
        context: {
          data: {},
          state: {},
          variables: this.inputs,
        },
        stepResults: {},
        lastStepIndex: -1,
      };
    }

    this.desktop = new Desktop();
  }

  async run(): Promise<{ status: string; lastStepId?: string; lastStepIndex: number }> {
    const steps = this.workflow.steps;

    // Find start and end indices
    let startIndex = 0;
    if (this.startFromStep) {
      startIndex = steps.findIndex(s => s.config.id === this.startFromStep);
      if (startIndex === -1) {
        throw new Error(`Start step '${this.startFromStep}' not found`);
      }
    }

    let endIndex = steps.length - 1;
    if (this.endAtStep) {
      endIndex = steps.findIndex(s => s.config.id === this.endAtStep);
      if (endIndex === -1) {
        throw new Error(`End step '${this.endAtStep}' not found`);
      }
    }

    // Execute steps
    for (let i = startIndex; i <= endIndex; i++) {
      const step = steps[i];

      console.log(`[${i + 1}/${steps.length}] ${step.config.name}`);

      try {
        const result = await step.run({
          desktop: this.desktop,
          input: this.inputs,
          context: this.state.context,
          logger: console,
        });

        // Save step result
        this.state.stepResults[step.config.id] = {
          status: 'success',
          result,
        };
        this.state.lastStepId = step.config.id;
        this.state.lastStepIndex = i;

      } catch (error: any) {
        // Save step error
        this.state.stepResults[step.config.id] = {
          status: 'error',
          error: error.message,
        };
        this.state.lastStepId = step.config.id;
        this.state.lastStepIndex = i;

        throw error;
      }
    }

    return {
      status: 'success',
      lastStepId: this.state.lastStepId,
      lastStepIndex: this.state.lastStepIndex,
    };
  }

  getState(): WorkflowState {
    return this.state;
  }
}

export function createWorkflowRunner(options: WorkflowRunnerOptions): WorkflowRunner {
  return new WorkflowRunner(options);
}
```

**Usage from Rust:**

```json
{
  "tool_name": "execute_sequence",
  "arguments": {
    "url": "file://path/to/project/workflow.ts",
    "start_from_step": "fill-sap-form",
    "end_at_step": "submit-form",
    "inputs": { "jsonFile": "./data.json" }
  }
}
```

**The runner handles:**
1. Finding step indices by ID
2. Restoring state from previous run
3. Executing only the specified range
4. Saving updated state after each step

---

### 5. Visualization: Parsing JS/TS for UI

**Goal:** Extract workflow structure (steps, loops, conditions) for UI rendering.

**Approach 1: Runtime Metadata (Recommended for MVP)**

```typescript
// In workflow.ts, enhance getMetadata()

interface WorkflowMetadata {
  name: string;
  description?: string;
  version?: string;
  input: any;  // Zod schema (can be serialized to JSON Schema)
  steps: StepMetadata[];

  // NEW: Visualization metadata
  visualization?: {
    loops?: LoopMetadata[];
    conditions?: ConditionMetadata[];
    branches?: BranchMetadata[];
  };
}

interface StepMetadata {
  id: string;
  name: string;
  description?: string;

  // NEW: Step type for visualization
  type?: 'action' | 'condition' | 'loop' | 'error-handler';

  // NEW: Visual hints
  icon?: string;  // e.g., "desktop", "file", "network"
  tags?: string[];  // e.g., ["SAP", "Excel", "API"]
}

interface LoopMetadata {
  stepId: string;
  type: 'for' | 'while' | 'forEach';
  description: string;
}

interface ConditionMetadata {
  stepId: string;
  condition: string;  // String representation
  description: string;
}
```

**Enhanced Step Definition:**

```typescript
// In packages/terminator-workflow/src/step.ts

export function createStep<TInput, TOutput>(
  config: StepConfig<TInput, TOutput>
): Step<TInput, TOutput> {
  return {
    config,

    async run(context: StepContext<TInput>): Promise<TOutput | void> {
      // Check condition
      if (config.condition && !config.condition({ input: context.input, context: context.context })) {
        context.logger.info(`⏭️  Skipping step (condition not met)`);
        return;
      }

      // ... existing execution logic ...
    },

    getMetadata() {
      return {
        id: config.id,
        name: config.name,
        description: config.description,

        // NEW: Enhanced metadata
        type: config.condition ? 'condition' : 'action',
        hasErrorHandler: !!config.onError,
        hasCondition: !!config.condition,
        timeout: config.timeout,
      };
    },
  };
}
```

**UI Consumption:**

```tsx
// In mediar-app frontend

import { WorkflowMetadata } from '@mediar/terminator-workflow';

function WorkflowVisualization({ metadata }: { metadata: WorkflowMetadata }) {
  return (
    <div className="workflow-timeline">
      {metadata.steps.map((step, i) => (
        <div key={step.id} className="step-card">
          {/* Step number badge */}
          <div className="step-number">{i + 1}</div>

          {/* Step info */}
          <div className="step-info">
            <span className="step-id">{step.id}</span>
            <h3>{step.name}</h3>
            <p>{step.description}</p>

            {/* Badges for features */}
            {step.hasCondition && <Badge>Conditional</Badge>}
            {step.hasErrorHandler && <Badge>Error Recovery</Badge>}
            {step.type === 'loop' && <Badge>Loop</Badge>}
          </div>

          {/* Connector line */}
          {i < metadata.steps.length - 1 && <div className="connector" />}
        </div>
      ))}
    </div>
  );
}
```

**Approach 2: Static AST Parsing (Future Enhancement)**

For more advanced visualization (e.g., showing actual loop logic, condition expressions), use AST parsing:

```typescript
// Future: @mediar/workflow-parser

import { parse } from '@typescript-eslint/parser';
import ts from 'typescript';

export function parseWorkflowAST(filePath: string): WorkflowVisualization {
  const sourceFile = ts.createSourceFile(
    filePath,
    fs.readFileSync(filePath, 'utf-8'),
    ts.ScriptTarget.Latest,
    true
  );

  // Extract loops, conditions, etc. from AST
  const loops = extractLoops(sourceFile);
  const conditions = extractConditions(sourceFile);

  return { loops, conditions, /* ... */ };
}
```

---

### 6. Backward Compatibility with YAML

**Goal:** Ensure existing YAML workflows continue to work.

**Strategy:** Zero changes to YAML workflow loading!

```rust
// In server_sequence.rs

async fn execute_sequence_impl(&self, args: ExecuteSequenceArgs) -> Result<Value, McpError> {
    // Detect format
    let format = if let Some(url) = &args.url {
        detect_workflow_format(url).await?
    } else {
        WorkflowFormat::Yaml  // Direct args = YAML format
    };

    // Branch based on format
    let final_args = match format {
        WorkflowFormat::Yaml => {
            // Existing YAML loading logic (lines 282-476)
            // NO CHANGES!
            self.load_yaml_workflow(args).await?
        }
        WorkflowFormat::TypeScript => {
            // NEW: TypeScript loading logic
            self.load_typescript_workflow(args).await?
        }
    };

    // Continue with unified execution (lines 503-2060)
    // NO CHANGES! Works for both formats.
    // ...
}
```

**Testing Matrix:**

| Test Case | URL | Expected Behavior |
|-----------|-----|-------------------|
| YAML file | `file://workflow.yml` | Load as YAML (existing) |
| YAML remote | `https://example.com/workflow.yml` | Load as YAML (existing) |
| TS file | `file://workflow.ts` | Load as TypeScript (new) |
| TS project | `file://project/` | Load as TypeScript (new) |
| Legacy direct args | `{ steps: [...] }` | Load as YAML (existing) |

---

## Implementation Plan

### Phase 1: Core TypeScript Executor (Week 1)

**Files to Create:**
- ✅ `terminator-mcp-agent/src/workflow_typescript.rs` - TypeScript workflow executor
- ✅ `terminator-mcp-agent/src/workflow_format.rs` - Format detection
- ✅ `packages/terminator-workflow/src/runner.ts` - Workflow runner with start/stop

**Files to Modify:**
- ✅ `terminator-mcp-agent/src/server_sequence.rs` - Add format branching
- ✅ `packages/terminator-workflow/src/index.ts` - Export runner

**Tasks:**
1. Implement `detect_workflow_format()`
2. Implement `TypeScriptWorkflow::execute()`
3. Implement `WorkflowRunner` class in TypeScript
4. Add format branching to `execute_sequence_impl()`
5. Integrate with existing state save/load functions

**Testing:**
- Load simple TS workflow and execute
- Verify all steps run in order
- Check Desktop instance is properly initialized

### Phase 2: State Caching (Week 1)

**Files to Modify:**
- ✅ `terminator-mcp-agent/src/server_sequence.rs` - Enhance context handling

**Tasks:**
1. Serialize TS `context` object into `env`
2. Deserialize `context` from `env` on resume
3. Pass context between step executions
4. Test start/stop at specific steps with state restoration

**Testing:**
- Run TS workflow with 3 steps
- Stop after step 2
- Check `.workflow_state/workflow.json` contains context
- Resume from step 3
- Verify context data is restored

### Phase 3: Visualization Metadata (Week 2)

**Files to Modify:**
- ✅ `packages/terminator-workflow/src/types.ts` - Add visualization metadata
- ✅ `packages/terminator-workflow/src/step.ts` - Enhance `getMetadata()`
- ✅ `packages/terminator-workflow/src/workflow.ts` - Enhance `getMetadata()`

**Files to Create:**
- ✅ `packages/terminator-workflow/src/visualization.ts` - Visualization types
- ✅ `examples/typescript-workflow/workflow-viewer-enhanced.html` - Enhanced viewer

**Tasks:**
1. Add `type`, `hasCondition`, `hasErrorHandler` to step metadata
2. Add `visualization` section to workflow metadata
3. Create example with loops and conditions
4. Update HTML viewer to show enhanced metadata

**Testing:**
- Load production workflow metadata
- Verify metadata includes all visualization info
- Render in HTML viewer
- Check all step types display correctly

### Phase 4: Integration Testing (Week 2)

**Files to Create:**
- ✅ `terminator-mcp-agent/tests/integration/test_typescript_workflow.rs`

**Tasks:**
1. Test YAML backward compatibility (all existing tests pass)
2. Test TS workflow loading and execution
3. Test state caching and resume
4. Test start/stop at specific steps
5. Test metadata extraction and visualization

**Test Scenarios:**
- ✅ Simple TS workflow (2-3 steps)
- ✅ Production TS workflow (with error recovery)
- ✅ Resume from middle step
- ✅ Stop at specific step
- ✅ YAML workflow still works
- ✅ Mixed usage (YAML + TS in same session)

---

## API Examples

### Example 1: Execute TypeScript Workflow

```json
{
  "tool_name": "execute_sequence",
  "arguments": {
    "url": "file://C:/workflows/sap-login/workflow.ts",
    "inputs": {
      "username": "john.doe",
      "maxRetries": 3
    }
  }
}
```

**Flow:**
1. Detect `workflow.ts` → TypeScript format
2. Execute `node` to load metadata via `getMetadata()`
3. Convert to `ExecuteSequenceArgs`
4. Execute steps via `run_typescript_step` tool
5. Save state after each step to `.workflow_state/workflow.json`

### Example 2: Resume from Specific Step

```json
{
  "tool_name": "execute_sequence",
  "arguments": {
    "url": "file://C:/workflows/sap-login/workflow.ts",
    "start_from_step": "submit-form",
    "inputs": {
      "username": "john.doe",
      "maxRetries": 3
    }
  }
}
```

**Flow:**
1. Load metadata
2. Find step index for `"submit-form"`
3. Load state from `.workflow_state/workflow.json`
4. Restore context from state
5. Execute from step `"submit-form"` onward

### Example 3: Bounded Execution

```json
{
  "tool_name": "execute_sequence",
  "arguments": {
    "url": "file://C:/workflows/sap-login/workflow.ts",
    "start_from_step": "check-duplicates",
    "end_at_step": "fill-sap-form",
    "inputs": {
      "jsonFile": "./test-data.json"
    }
  }
}
```

**Flow:**
1. Execute only steps `"check-duplicates"` → `"fill-sap-form"`
2. Skip output parser (partial execution)
3. Save state after each step (can resume later)

---

## File Structure

### After Implementation

```
terminator/
├── terminator-mcp-agent/
│   ├── src/
│   │   ├── server.rs                    # Register run_typescript_step
│   │   ├── server_sequence.rs           # Add format branching
│   │   ├── workflow_adapter.rs          # NEW: TS adapter
│   │   ├── workflow_format.rs           # NEW: Format detection
│   │   └── utils.rs                     # Existing ExecuteSequenceArgs
│   └── tests/
│       └── integration/
│           └── test_typescript_workflow.rs  # NEW: TS workflow tests
│
├── packages/
│   └── terminator-workflow/
│       └── src/
│           ├── workflow.ts              # Enhanced getMetadata()
│           ├── step.ts                  # Enhanced getMetadata()
│           ├── types.ts                 # Add visualization types
│           └── visualization.ts         # NEW: Visualization helpers
│
└── examples/
    └── typescript-workflow/
        ├── workflow.ts                  # Example workflow
        ├── package.json                 # With tsx dependency
        ├── workflow-viewer.html         # Basic viewer
        └── workflow-viewer-enhanced.html # NEW: Enhanced viewer
```

---

## Migration Guide

### For YAML Users (No Changes Required)

Existing YAML workflows continue to work:

```yaml
# workflow.yml - Still works!
steps:
  - id: step1
    tool_name: run_command
    arguments:
      run: echo "Hello"
```

### For TypeScript Users

Convert YAML to TypeScript:

**Before (YAML):**
```yaml
steps:
  - id: login
    name: Login to SAP
    tool_name: click_element
    arguments:
      selector: "role:button|name:Login"
```

**After (TypeScript):**
```typescript
const login = createStep({
  id: 'login',
  name: 'Login to SAP',
  execute: async ({ desktop }) => {
    await desktop.locator('role:button|name:Login').click();
  },
});
```

---

## Summary

### What Works Out of the Box

✅ **State caching** - Existing `.workflow_state` mechanism works
✅ **Start/stop at any step** - Existing `start_from_step`/`end_at_step` works
✅ **YAML backward compatibility** - Zero changes to YAML loading
✅ **Step execution** - Convert TS steps to `run_typescript_step` tool calls

### What Needs Implementation

🔨 **Format detection** - Detect TS vs YAML from URL/file extension
🔨 **TypeScript executor** - Execute workflow via Node.js with `tsx`
🔨 **Workflow runner** - TypeScript class to handle start/stop/state
🔨 **State integration** - Reuse existing save/load functions
🔨 **Metadata extraction** - Call `.getMetadata()` for visualization

### Timeline

- **Week 1:** Core adapter + state caching
- **Week 2:** Visualization metadata + integration testing
- **Total:** 2 weeks to MVP

### Benefits

✅ Unified execution engine (both YAML and TS use same `execute_sequence`)
✅ Incremental migration (can mix YAML and TS workflows)
✅ Full feature parity (caching, start/stop, visualization)
✅ Type safety for TS workflows
✅ Zero breaking changes for existing users
