# TypeScript Workflow Exploration - PR 318 Alternative

## Overview

This document explores an alternative, simpler DX for TypeScript workflows based on the original PR 318 goals but with a cleaner approach.

## Problem Statement

PR 318 aimed to add TypeScript workflow support but resulted in:
- 2476 additions, 7 deletions across 69 files
- Complex parser for JavaScript/TypeScript code
- Multiple framework compatibility layers (Mastra, Inngest)
- Builder pattern with abstractions

**We want something simpler.**

## Proposed Solution

### Core Concept
1. **YAML config file** - Metadata for UI parsing (steps, variables, etc.)
2. **TypeScript file** - Simple exported functions matching YAML steps
3. **No magic** - Just regular async functions with clear signatures

### Why This Approach?

#### For Developers
✅ **Simple & Clear** - Just export functions, no builders
✅ **Standard TypeScript** - Use regular async functions
✅ **Type Safety** - Full TypeScript support
✅ **Easy Testing** - Each function can be tested independently
✅ **No Build Steps** - Use `tsx` to run directly
✅ **No Abstractions** - No decorators, no builders, no magic

#### For mediar-app UI
✅ **Easy Parsing** - YAML is trivial to parse
✅ **Clear Structure** - Steps, variables, and functions are explicit
✅ **UI Rendering** - All metadata available for rendering
✅ **Step Mapping** - Function names map directly to implementations
✅ **Variable Inputs** - UI knows which steps need which variables

## Examples Created

### 1. Simple Example (`examples/ts-workflow-simple/`)
Basic workflow showing:
- `workflow.yml` - Metadata config
- `workflow.ts` - Exported step functions
- Direct execution with `tsx`
- CLI argument parsing

**Key Files:**
- `workflow.yml` - 3 steps, 2 variables
- `workflow.ts` - 3 exported functions (openNotepad, typeGreeting, addDate)

### 2. Advanced Example (`examples/ts-workflow-advanced/`)
Complex workflow demonstrating:
- Organized step files in `steps/` directory
- State sharing via context object
- Type-safe variables interface
- Conditional execution
- Error handling

**Key Files:**
- `workflow.yml` - 9 steps, 4 variables
- `workflow.ts` - Main orchestration
- `steps/*.ts` - Individual step implementations

## YAML Structure

```yaml
id: workflow-id
name: Workflow Name
description: What it does
version: 1.0.0
tags:
  - tag1
  - tag2

variables:
  varName:
    type: string
    label: Display Name
    description: Help text
    required: true
    default: default value

steps:
  - id: step-id
    name: Step Name
    function: functionName  # Maps to export async function functionName()
    description: What this step does
    inputs:              # Which variables this step needs
      - varName
    condition: variables.varName === true  # Optional conditional
    timeout: 5000        # Optional timeout in ms
```

## TypeScript Structure

```typescript
import { Desktop } from '@mediar/terminator';

// Simple pattern: export functions matching YAML
export async function stepOne(desktop: Desktop) {
  // Step implementation
}

export async function stepTwo(
  desktop: Desktop,
  variables: { varName: string }
) {
  // Step with variables
}

// Optional: Share state between steps
interface WorkflowContext {
  stepOneResult?: string;
}

export async function stepThree(
  desktop: Desktop,
  context: WorkflowContext
) {
  // Use context.stepOneResult
}

// Main entry point for direct execution
export async function main(variables: Record<string, any> = {}) {
  const desktop = new Desktop();
  const context: WorkflowContext = {};

  await stepOne(desktop);
  await stepTwo(desktop, variables);
  await stepThree(desktop, context);
}
```

## Function Signatures

Three patterns for step functions:

### 1. Basic Step (no inputs)
```typescript
export async function stepName(desktop: Desktop): Promise<void>
```

### 2. Step with Variables
```typescript
export async function stepName(
  desktop: Desktop,
  variables: { varName: string }
): Promise<void>
```

### 3. Step with Context (state sharing)
```typescript
export async function stepName(
  desktop: Desktop,
  context: WorkflowContext
): Promise<void>
```

## Execution Flow

### Direct Execution
```bash
# Run with tsx
tsx workflow.ts --varName "value"

# Or with npm
npm start -- --varName "value"
```

### Via Terminator CLI
```bash
# Execute workflow by pointing to YAML
terminator-cli execute-workflow --url file://./workflow.yml

# CLI reads YAML, loads .ts file, executes functions
```

### From mediar-app
1. Parse `workflow.yml` to display workflow info
2. Show variables as form inputs in UI
3. Render steps from YAML metadata
4. Execute by running `tsx workflow.ts` with user inputs

## Implementation Needed

To support this approach, we need to build:

### 1. YAML Parser
- Parse `workflow.yml` files
- Extract metadata, variables, steps
- Validate structure

### 2. TypeScript Module Loader
- Load `.ts` files with `tsx` or similar
- Resolve exported functions
- Map YAML step functions to TS exports

### 3. Function Executor
- Execute functions in step order
- Pass variables to functions that need them
- Handle context passing between steps
- Respect conditional execution

### 4. CLI Integration
- Detect workflow.yml files
- Load and execute workflows
- Pass CLI args as variables

### 5. mediar-app Integration
- Parse YAML for UI display
- Generate form from variables
- Execute workflow with user inputs
- Display step progress

## Comparison with PR 318 Approach

| Aspect | PR 318 | This Proposal |
|--------|--------|---------------|
| **Files Changed** | 69 files, 2476 additions | TBD (much smaller) |
| **Complexity** | High (parser, converters, builders) | Low (YAML + function loader) |
| **DX** | Builder pattern, abstractions | Simple exported functions |
| **Magic** | Some (export detection, parsing) | Minimal (just function mapping) |
| **Framework Compat** | Yes (Mastra, Inngest) | No (but simpler) |
| **Type Safety** | Yes | Yes |
| **Testing** | Harder (builder abstraction) | Easy (test functions directly) |
| **Learning Curve** | Medium | Low |

## Benefits Over PR 318

1. **Much Simpler** - No complex JS/TS parser, no builder pattern
2. **Clearer Mapping** - YAML explicitly lists function names
3. **Easier Testing** - Test individual functions without abstractions
4. **Better Separation** - Config (YAML) separate from code (TS)
5. **More Flexible** - Can organize steps however you want
6. **Less Code** - Smaller implementation, fewer files changed

## Tradeoffs

### What We Lose
- ❌ Framework compatibility (Mastra, Inngest patterns)
- ❌ Inline workflow definitions (need separate YAML)
- ❌ Export detection (must match YAML)

### What We Gain
- ✅ Simpler mental model
- ✅ Easier to implement
- ✅ Easier to debug
- ✅ Clear separation of concerns
- ✅ More explicit (less magic)

## Next Steps

1. **Gather Feedback** - Is this the right direction?
2. **Build Prototype** - Implement YAML parser + function executor
3. **Test Integration** - Try with mediar-app UI
4. **Document Patterns** - Best practices, common patterns
5. **Migration Path** - How to convert existing YAML workflows?

## Questions to Answer

1. Should we support ES modules or only CommonJS?
2. Should context passing be explicit or implicit?
3. Should we validate function signatures?
4. Should we support async generators for streaming progress?
5. Should we allow importing helper functions from other files?
6. Should we support workflow composition (calling other workflows)?

## Conclusion

This approach prioritizes **simplicity and clarity** over feature richness. It's easier to implement, easier to understand, and easier to maintain than the PR 318 approach.

The core insight: **YAML for structure, TypeScript for logic**. Keep them separate, keep them simple.
