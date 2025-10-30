# New TypeScript Workflow Features

## Overview

The TypeScript workflow SDK now includes enhanced error handling, validation, and response types for better testing and UI integration.

## 1. `expect()` - Step Validation

Runs after `execute()` to verify the step completed successfully. Similar to testing frameworks.

```typescript
const login = createStep({
  id: 'login',
  name: 'Login to SAP',

  execute: async ({ desktop, input }) => {
    await desktop.locator('role:textbox|name:Username').typeText(input.username);
    await desktop.locator('role:button|name:Login').click();
  },

  expect: async ({ desktop, result, logger }) => {
    // Verify login succeeded - check for home screen
    const homeCheck = await desktop.locator('role:Window|name:SAP Home').validate(5000);

    if (!homeCheck.exists) {
      return {
        success: false,
        message: 'Login failed - home screen not found',
      };
    }

    return {
      success: true,
      message: 'Successfully logged in',
      data: { userId: '12345' },  // Optional custom data
    };
  },
});
```

### ExpectationResult Type

```typescript
interface ExpectationResult {
  success: boolean;       // Whether expectation was met
  message?: string;       // Optional description
  data?: any;            // Optional custom data
}
```

### ExpectationContext

```typescript
interface ExpectationContext<TInput, TOutput> {
  desktop: Desktop;      // Desktop automation instance
  input: TInput;         // Workflow input
  result: TOutput;       // Result from execute()
  context: WorkflowContext;  // Shared workflow context
  logger: Logger;        // Logger instance
}
```

## 2. Enhanced `onError()` - Step-Level Error Recovery

Handle specific error scenarios and retry or fail gracefully.

```typescript
const login = createStep({
  id: 'login',
  name: 'Login to SAP',

  execute: async ({ desktop, input }) => {
    // ... login logic
  },

  onError: async ({ error, desktop, retry, logger }) => {
    logger.warn(`Login error: ${error.message}`);

    // Check for session conflict
    const conflictDialog = await desktop
      .locator('role:Dialog|name:Session Conflict')
      .validate(1000);

    if (conflictDialog.exists) {
      logger.info('Closing existing session...');
      await desktop.locator('role:Button|name:Close Session').click();
      await desktop.delay(2000);

      // Retry the step
      return retry();
    }

    // Check for invalid credentials
    const errorDialog = await desktop
      .locator('role:Dialog|name:Invalid Credentials')
      .validate(1000);

    if (errorDialog.exists) {
      return {
        recoverable: false,
        reason: 'Invalid credentials - cannot retry',
      };
    }

    // Unknown error
    return {
      recoverable: false,
      reason: `Unknown error: ${error.message}`,
    };
  },
});
```

## 3. Workflow-Level `onError()`

Handle errors at the workflow level and categorize them for UI rendering.

```typescript
const workflow = createWorkflow({
  name: 'SAP Login',
  input: InputSchema,

  onError: async ({ error, step, logger }): Promise<ExecutionResponse> => {
    logger.error(`Workflow failed at: ${step.config.name}`);

    // Business logic error
    if (error.message.includes('Invalid credentials')) {
      return {
        status: 'error',
        error: {
          category: 'business',
          code: 'INVALID_CREDENTIALS',
          message: 'Please check your username and password',
        },
        message: 'Authentication failed',
      };
    }

    // Technical error
    if (error.message.includes('timeout')) {
      return {
        status: 'error',
        error: {
          category: 'technical',
          code: 'UI_ELEMENT_NOT_FOUND',
          message: 'Application may be slow or UI changed',
        },
        message: 'Technical error',
      };
    }

    // Unknown error
    return {
      status: 'error',
      error: {
        category: 'technical',
        code: 'UNKNOWN_ERROR',
        message: error.message,
      },
    };
  },
})
  .step(loginStep)
  .build();
```

## 4. ExecutionResponse - Structured Return Value

Workflows now return a structured response for better UI integration.

```typescript
interface ExecutionResponse<TData = any> {
  // Well-rendered in UI
  status: 'success' | 'error' | 'warning' | 'user_input_required';

  // Error info (only if status is 'error')
  error?: {
    category: 'business' | 'technical';
    code: string;
    message?: string;
  };

  // Custom data (less prominent in UI)
  data?: TData;

  // Optional user-facing message
  message?: string;
}
```

### Status Values

- **`success`**: Workflow completed successfully
- **`error`**: Workflow failed
- **`warning`**: Workflow completed with warnings
- **`user_input_required`**: Workflow paused, needs user input

### Error Categories

- **`business`**: Business logic error (invalid credentials, company not found, etc.)
- **`technical`**: Technical error (timeout, UI element not found, network error, etc.)

## 5. Complete Example

```typescript
const workflow = createWorkflow({
  name: 'SAP Data Entry',
  input: z.object({
    username: z.string(),
    invoices: z.array(z.object({
      number: z.string(),
      amount: z.number(),
    })),
  }),

  onError: async ({ error, step }) => {
    if (error.message.includes('credentials')) {
      return {
        status: 'error',
        error: {
          category: 'business',
          code: 'AUTH_FAILED',
        },
      };
    }

    return {
      status: 'error',
      error: {
        category: 'technical',
        code: 'UNKNOWN',
        message: error.message,
      },
    };
  },
})
  .step(createStep({
    id: 'login',
    name: 'Login',
    execute: async ({ desktop, input }) => {
      await desktop.locator('role:Edit|name:User').typeText(input.username);
      await desktop.locator('role:Button|name:Login').click();
    },
    expect: async ({ desktop }) => {
      const home = await desktop.locator('role:Window|name:SAP').validate(5000);
      return {
        success: home.exists,
        message: home.exists ? 'Login successful' : 'Login failed',
      };
    },
    onError: async ({ error, desktop, retry }) => {
      const conflict = await desktop.locator('role:Dialog|name:Conflict').validate(1000);
      if (conflict.exists) {
        await desktop.locator('role:Button|name:Close Session').click();
        return retry();
      }
      return { recoverable: false };
    },
  }))
  .step(createStep({
    id: 'enter-invoices',
    name: 'Enter Invoices',
    execute: async ({ desktop, input }) => {
      for (const invoice of input.invoices) {
        await desktop.locator('role:Edit|name:Invoice #').typeText(invoice.number);
        await desktop.locator('role:Edit|name:Amount').typeText(String(invoice.amount));
        await desktop.pressKey('{Enter}');
      }
      return { invoicesEntered: input.invoices.length };
    },
    expect: async ({ result }) => {
      return {
        success: result.invoicesEntered > 0,
        message: `Entered ${result.invoicesEntered} invoices`,
        data: result,
      };
    },
  }))
  .build();

// Run workflow
const response = await workflow.run({
  username: 'john_doe',
  invoices: [
    { number: '1001', amount: 1500 },
    { number: '1002', amount: 2800 },
  ],
});

console.log(response);
// {
//   status: 'success',
//   message: 'Workflow completed successfully in 12345ms',
//   data: { invoicesEntered: 2 }
// }
```

## Migration Guide

### Old Code (Still Works)

```typescript
const step = createStep({
  id: 'login',
  execute: async ({ desktop }) => {
    await desktop.locator('role:Button').click();
  },
});

const workflow = createWorkflow({ name: 'Test', input: z.object({}) })
  .step(step)
  .build();

await workflow.run({});  // Returns void (old behavior)
```

### New Code

```typescript
const step = createStep({
  id: 'login',
  execute: async ({ desktop }) => {
    await desktop.locator('role:Button').click();
  },
  expect: async ({ desktop }) => {
    const check = await desktop.locator('role:Window').validate(1000);
    return { success: check.exists };
  },
  onError: async ({ retry }) => {
    return retry();
  },
});

const workflow = createWorkflow({
  name: 'Test',
  input: z.object({}),
  onError: async () => ({
    status: 'error',
    error: { category: 'technical', code: 'FAIL' },
  }),
})
  .step(step)
  .build();

const response = await workflow.run({});  // Returns ExecutionResponse
console.log(response.status);  // 'success' or 'error'
```

## Benefits

1. **Better Testing**: `expect()` provides built-in validation similar to test frameworks
2. **Error Recovery**: Step-level `onError()` allows handling specific scenarios (session conflicts, retries)
3. **Error Categorization**: Distinguish business vs technical errors for better UI rendering
4. **Structured Responses**: `ExecutionResponse` provides consistent format for UI integration
5. **Type Safety**: All features are fully typed with TypeScript
