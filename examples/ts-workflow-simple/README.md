# Simple TypeScript Workflow Example

This demonstrates the proposed TypeScript workflow approach with YAML metadata config.

## Structure

- `workflow.yml` - Metadata config for UI parsing (steps, variables, etc.)
- `workflow.ts` - TypeScript implementation with exported step functions
- `package.json` - Node.js project config
- `tsconfig.json` - TypeScript config

## Key Concepts

### 1. YAML Config (workflow.yml)
Defines the workflow structure for UI parsing:
- Workflow metadata (id, name, description)
- Variables (inputs the user can configure)
- Steps (ordered list of actions with function names)

### 2. TypeScript Implementation (workflow.ts)
Simple exported functions matching the YAML steps:
- Each step is an exported async function
- Functions receive `desktop` and `variables` as parameters
- No complex builders or abstractions
- Just regular TypeScript code

### 3. Function Mapping
```yaml
# workflow.yml
steps:
  - id: open-app
    function: openNotepad  # Maps to export async function openNotepad()
```

```typescript
// workflow.ts
export async function openNotepad(desktop: Desktop) {
  // Step implementation
}
```

## Running

### Install dependencies
```bash
npm install
```

### Run with default variables
```bash
npm start
```

### Run with custom variables
```bash
tsx workflow.ts --userName "Alice" --includeDate false
```

### Run via Terminator CLI
```bash
terminator-cli execute-workflow --url file://./workflow.yml
```

## Benefits

### For Developers
✅ Simple, clear code structure
✅ Standard TypeScript - no magic
✅ Easy to test individual functions
✅ Full IDE support and type safety
✅ No build step required (tsx runs directly)

### For mediar-app UI
✅ YAML is easy to parse
✅ Clear step structure for rendering
✅ Variable definitions for form inputs
✅ Function names map to implementations
✅ Conditional steps clearly marked

## Next Steps

This example demonstrates the basic pattern. Future enhancements could include:

1. **Step dependencies** - Express which steps depend on others
2. **Error handling** - Standard patterns for retry and fallback
3. **State sharing** - Pass context between steps
4. **Validation** - Validate YAML structure and function exports
5. **Testing** - Unit test individual step functions
