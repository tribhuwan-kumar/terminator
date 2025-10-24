# Advanced DX Proposal: TypeScript Workflows with AI Recovery

## Overview

This proposal outlines an advanced DX for Terminator workflows, inspired by Vercel, Mastra.ai, and Inngest, but optimized specifically for:

✅ **AI Code Generation** - Structure that AI can easily write and understand
✅ **Reliability** - Built-in error handling and recovery patterns
✅ **AI Recovery** - Automatic AI-powered error recovery
✅ **Production-Ready** - Patterns from real production workflows

## Inspiration Sources

### Vercel Functions
- Simple `export default` patterns
- Clear function signatures
- Built-in error handling

### Mastra.ai
- `createStep()` pattern for composability
- Type-safe step definitions
- Pipeline composition

### Inngest
- Event-driven workflow execution
- Step isolation and retries
- Built-in observability

## Core API Design

### 1. `createStep()` - The Building Block

```typescript
import { createStep } from '@mediar/terminator-workflow';

const loginStep = createStep({
  id: 'login',
  name: 'Login to Application',
  description: 'Authenticate with credentials',
  timeout: 30000, // Optional timeout

  // Main execution function
  execute: async ({ desktop, variables, context, logger }) => {
    logger.info('🔐 Logging in...');

    await desktop.locator('role:textbox|name:Username').fill(variables.username);
    await desktop.locator('role:textbox|name:Password').fill(variables.password);
    await desktop.locator('role:button|name:Sign In').click();

    const success = await desktop.locator('role:heading|name:Dashboard').exists({ timeout: 10000 });

    if (!success) {
      throw new Error('Login failed - dashboard not found');
    }

    logger.success('✅ Login successful');

    // Return value is stored in context for next steps
    return { sessionId: 'abc123', loginTime: Date.now() };
  },

  // AI-powered error recovery
  onError: async ({ error, desktop, context, logger, retry, attempt }) => {
    logger.warn(`⚠️ Login failed: ${error.message}`);

    // Check for known error patterns
    if (error.message.includes('Session conflict')) {
      logger.info('🔄 Session conflict - closing existing session...');

      await desktop.locator('role:button|name:Close Other Session').click();
      await desktop.wait(2000);

      return retry(); // Retry the execute() function
    }

    // Use AI to analyze and recover
    const screenshot = await desktop.screenshot();
    const tree = await desktop.getAccessibilityTree();

    const aiAnalysis = await context.ai.analyze({
      screenshot,
      tree,
      goal: 'Login to the application',
      error: error.message,
      variables: context.variables,
    });

    if (aiAnalysis.canRecover) {
      logger.info(`🤖 AI suggests: ${aiAnalysis.explanation}`);

      // Execute AI's recovery steps
      for (const step of aiAnalysis.steps) {
        await desktop.locator(step.selector)[step.action](step.value);
      }

      return retry();
    }

    // Cannot recover
    return {
      recoverable: false,
      reason: error.message,
      aiAttempted: true,
    };
  },

  // Simple heuristic fallback (when AI is not available)
  fallback: async ({ desktop, logger }) => {
    logger.info('🔧 Trying heuristic fallback: restart app...');

    await desktop.closeApplication('app.exe');
    await desktop.wait(2000);
    await desktop.openApplication('app.exe');
    await desktop.wait(5000);

    // Try login again
    await desktop.locator('role:textbox|name:Username').fill(variables.username);
    await desktop.locator('role:textbox|name:Password').fill(variables.password);
    await desktop.locator('role:button|name:Sign In').click();

    return { success: true, method: 'fallback' };
  },

  // Error classification for smart retry logic
  classifyError: (error: Error) => {
    const permanentPatterns = [
      'Invalid credentials',
      'Account locked',
      'Permission denied',
    ];

    const temporaryPatterns = [
      'Session conflict',
      'Network error',
      'Timeout',
      'Element not found',
    ];

    for (const pattern of permanentPatterns) {
      if (error.message.includes(pattern)) {
        return {
          type: 'permanent',
          shouldRetry: false,
          reason: error.message,
        };
      }
    }

    for (const pattern of temporaryPatterns) {
      if (error.message.includes(pattern)) {
        return {
          type: 'temporary',
          shouldRetry: true,
          maxRetries: 3,
          backoff: 'exponential', // or 'linear', 'constant'
          reason: error.message,
        };
      }
    }

    return { type: 'unknown', shouldRetry: true, maxRetries: 1 };
  },
});
```

### 2. `createWorkflow()` - Composing Steps

```typescript
import { createWorkflow } from '@mediar/terminator-workflow';

export const workflow = createWorkflow({
  id: 'sap-journal-entry',
  name: 'SAP Journal Entry Automation',
  description: 'Process JSON files and create journal entries in SAP',
  version: '1.0.0',

  variables: {
    jsonFile: {
      type: 'string',
      label: 'JSON File Path',
      required: true,
    },
    maxRetries: {
      type: 'number',
      label: 'Max Retry Attempts',
      default: 3,
    },
  },
})
  // Chain steps
  .step(checkDuplicates)
  .step(loginToSAP)
  .step(selectCompany)
  .step(navigateToJournalEntry)
  .step(fillJournalData)
  .step(verifyTotals)
  .step(submitForm)
  .step(verifySuccess)

  // Global error handler
  .onError(async ({ error, step, context, logger }) => {
    logger.error(`❌ Workflow failed at: ${step.name}`);

    // Move file to failed folder
    await moveToFailed(context.variables.jsonFile, {
      error: error.message,
      step: step.id,
      timestamp: new Date(),
    });
  })

  // Success handler
  .onSuccess(async ({ context, logger }) => {
    logger.success('✅ Workflow completed!');

    // Move file to processed folder
    await moveToProcessed(context.variables.jsonFile);
  });
```

### 3. Context & State Management

```typescript
// Context is passed to all steps
interface StepContext {
  // Workflow variables (from user input or config)
  variables: Record<string, any>;

  // Shared state between steps (mutable)
  data: any;
  state: Record<string, any>;

  // AI integration
  ai: {
    analyze: (params: AIAnalysisParams) => Promise<AIRecoverySuggestion>;
    classify: (error: Error) => Promise<ErrorClassification>;
  };

  // Utilities
  wait: (ms: number) => Promise<void>;
  logger: Logger;
}

// Example: Sharing data between steps
const readData = createStep({
  id: 'read-data',
  execute: async ({ context, logger }) => {
    const data = await readJSONFile(context.variables.jsonFile);

    // Store in context for next steps
    context.data = data;
    context.state.outletCode = data.outlet_code;
    context.state.entriesCount = data.entries.length;

    return { success: true };
  },
});

const processData = createStep({
  id: 'process-data',
  execute: async ({ context, logger }) => {
    // Access data from previous step
    const data = context.data;
    const outletCode = context.state.outletCode;

    logger.info(`Processing ${context.state.entriesCount} entries for ${outletCode}`);

    // Process the data...
  },
});
```

### 4. AI Recovery Patterns

```typescript
// Pattern 1: Screenshot + Tree Analysis
onError: async ({ error, desktop, context, retry }) => {
  const screenshot = await desktop.screenshot();
  const tree = await desktop.getAccessibilityTree();

  const suggestion = await context.ai.analyze({
    screenshot,
    tree,
    goal: 'Fill the form with data',
    error: error.message,
    context: context.data,
  });

  if (suggestion.canRecover) {
    for (const step of suggestion.steps) {
      await desktop.execute(step);
    }
    return retry();
  }

  return { recoverable: false };
}

// Pattern 2: Error Classification
classifyError: (error: Error) => {
  // Use AI to classify error
  const classification = await context.ai.classify(error);

  return {
    type: classification.isPermanent ? 'permanent' : 'temporary',
    shouldRetry: !classification.isPermanent,
    maxRetries: classification.suggestedRetries,
    reason: classification.reason,
  };
}

// Pattern 3: Heuristic Fallback Chain
fallback: async ({ desktop, logger, attempt }) => {
  // Try progressively more aggressive recovery
  const strategies = [
    () => desktop.refresh(), // Refresh page
    () => desktop.closePopups(), // Close popups
    () => desktop.restartApp(), // Restart app
    () => desktop.restartBrowser(), // Restart browser
  ];

  if (attempt < strategies.length) {
    await strategies[attempt]();
    return { success: true, strategy: attempt };
  }

  return { success: false };
}
```

### 5. Conditional Steps & Branching

```typescript
const workflow = createWorkflow({ ... })
  .step(readData)

  // Conditional step - only runs if condition is true
  .step(checkDuplicates, {
    condition: (context) => context.variables.checkDuplicates === true,
  })

  // Branch based on result
  .step(classifyData)
  .branch((context) => {
    if (context.state.isDuplicate) {
      return [skipProcessing, logDuplicate];
    } else {
      return [processData, submitData];
    }
  })

  // Or use switch-style branching
  .switch((context) => context.state.errorType, {
    'permanent': [logError, moveToFailed],
    'temporary': [retryProcessing],
    'unknown': [classifyWithAI, decide],
  });
```

### 6. Parallel Execution

```typescript
const workflow = createWorkflow({ ... })
  .step(prepareData)

  // Run steps in parallel
  .parallel([
    validateData,
    checkDuplicates,
    classifyOutlet,
  ])

  // Results from parallel steps available in context
  .step(processResults, {
    execute: async ({ context }) => {
      const [validation, duplicate, classification] = context.parallelResults;

      if (!validation.success) {
        throw new Error('Validation failed');
      }

      // Continue...
    },
  });
```

### 7. Observability & Logging

```typescript
const step = createStep({
  id: 'process-data',

  execute: async ({ logger, context }) => {
    logger.info('📊 Processing data...');
    logger.debug('Data:', context.data);

    try {
      const result = await processData(context.data);

      logger.success('✅ Processed successfully');
      logger.metric('entries_processed', result.count);

      return result;

    } catch (error) {
      logger.error('❌ Processing failed:', error);
      logger.metric('processing_errors', 1);

      throw error;
    }
  },

  // Step metrics/telemetry
  onComplete: async ({ result, duration, logger }) => {
    logger.metric('step_duration_ms', duration);
    logger.metric('entries_processed', result.count);
  },
});
```

## Why This DX is Better

### For AI Code Generation

✅ **Clear Structure** - AI can easily understand the pattern
✅ **Type Safety** - TypeScript provides hints for AI
✅ **Composable** - AI can build complex workflows from simple steps
✅ **Self-Documenting** - Function names and descriptions guide AI

Example: AI prompt
```
Create a workflow step that logs into SAP, handles session conflicts,
and uses AI recovery if login fails.
```

AI can generate:
```typescript
const loginToSAP = createStep({
  id: 'login-sap',
  name: 'Login to SAP',
  execute: async ({ desktop, variables }) => {
    await desktop.locator('role:textbox|name:Username').fill(variables.username);
    await desktop.locator('role:textbox|name:Password').fill(variables.password);
    await desktop.locator('role:button|name:Login').click();
    // ... rest generated by AI
  },
  onError: async ({ error, desktop, retry }) => {
    if (error.message.includes('Session conflict')) {
      await desktop.locator('role:button|name:Close Session').click();
      return retry();
    }
    // ... rest generated by AI
  },
});
```

### For Reliability

✅ **Built-in Retry Logic** - Automatic retries with backoff
✅ **Error Classification** - Distinguish permanent vs temporary errors
✅ **AI Recovery** - Automatic error analysis and recovery
✅ **Fallback Strategies** - Heuristic fallbacks when AI unavailable
✅ **State Management** - Share context between steps safely

### For Production Workflows

✅ **File Management** - Move to processed/failed folders
✅ **Error Metadata** - Save error details for debugging
✅ **Duplicate Detection** - Built-in duplicate checking
✅ **Observability** - Metrics and logging throughout
✅ **Conditional Execution** - Handle different scenarios

## Comparison with Production Workflow

Looking at `workflows/imperial_treasure_1/`, this DX addresses:

### Current Pain Points

❌ **40+ separate .js files** - Hard to maintain
❌ **Manual error classification** (classify_error.js has 188 lines)
❌ **Manual file management** (move_to_failed.js, move_to_processed.js)
❌ **Repetitive error handling** - Every file has similar try/catch
❌ **No type safety** - Variables passed as strings
❌ **Hard to test** - Tight coupling with file system

### With Proposed DX

✅ **Single workflow file** - All steps in one place
✅ **Automatic error classification** - Built into `createStep()`
✅ **Automatic file management** - `.onError()` and `.onSuccess()` handlers
✅ **Built-in error recovery** - `onError`, `fallback`, `classifyError`
✅ **Type safety** - Full TypeScript support
✅ **Easy to test** - Steps are pure functions

## Implementation Roadmap

### Phase 1: Core SDK
- [ ] Implement `createStep()` API
- [ ] Implement `createWorkflow()` API
- [ ] Context and state management
- [ ] Step execution engine

### Phase 2: Error Handling
- [ ] Retry logic with backoff
- [ ] Error classification
- [ ] Fallback strategies
- [ ] Error metadata collection

### Phase 3: AI Integration
- [ ] AI analysis API (`context.ai.analyze()`)
- [ ] AI error classification (`context.ai.classify()`)
- [ ] Screenshot + tree analysis
- [ ] Recovery suggestion generation

### Phase 4: Advanced Features
- [ ] Conditional steps
- [ ] Branching and switching
- [ ] Parallel execution
- [ ] Observability and metrics

### Phase 5: Production Features
- [ ] File management utilities
- [ ] Duplicate detection
- [ ] Workflow state persistence
- [ ] Workflow scheduling

## Next Steps

1. **Gather Feedback** - Is this the right direction?
2. **Build Prototype** - Implement core `createStep()` and `createWorkflow()`
3. **Test with Real Workflow** - Convert imperial_treasure_1 workflow
4. **Iterate** - Refine based on learnings
5. **Document** - Write comprehensive docs and examples

## Conclusion

This DX combines the best patterns from Vercel, Mastra, and Inngest while being optimized specifically for Terminator's needs:

- **AI-friendly** for code generation
- **Reliable** with built-in error recovery
- **Production-ready** with real-world patterns
- **Simple** for developers to understand and use

The key insight: **Make reliability and error recovery first-class citizens of the API**, not afterthoughts.
