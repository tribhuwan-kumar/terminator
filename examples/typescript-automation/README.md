# TypeScript Automation Example

This folder demonstrates TypeScript support in terminator workflows.

## Current Status

⚠️ **Note**: TypeScript execution is supported but result capture from TypeScript scripts is currently not working properly in the MCP agent. The TypeScript files compile and run, but results are not returned to the workflow.

## Files

### TypeScript Files (Ready for when result capture is fixed)
- `analyze-apps.ts` - TypeScript application analysis
- `notepad-automation.ts` - TypeScript Notepad automation

### Workflow Files
- `run-typescript-analysis.yml` - Runs analyze-apps.ts
- `run-typescript-notepad.yml` - Runs notepad-automation.ts

## Running the Examples

```bash
# Try running TypeScript analysis (currently doesn't return results)
terminator mcp run examples/typescript-automation/run-typescript-analysis.yml

# Try running TypeScript Notepad automation (currently doesn't return results)
terminator mcp run examples/typescript-automation/run-typescript-notepad.yml
```

## Known Issues

1. **TypeScript Result Capture**: The MCP agent successfully compiles and executes TypeScript files using Bun's native TypeScript support, but the result capture mechanism doesn't work. The scripts run but return "No result received from TypeScript process".

2. **Workaround**: For now, use `engine: javascript` instead of `engine: typescript` for working automation.

## TypeScript Features Demonstrated

When result capture is fixed, these examples will demonstrate:
- Native TypeScript execution
- Type safety with interfaces
- Loading TypeScript from external `.ts` files
- Desktop automation with terminator.js
- Async/await patterns
- Error handling with try/catch

## Technical Details

The MCP agent uses:
- Bun for native TypeScript execution (no transpilation needed)
- terminator.js for desktop automation
- Automatic dependency installation

The issue appears to be that TypeScript execution doesn't use the same wrapper mechanism as JavaScript, so results aren't captured with the `__RESULT__` markers.