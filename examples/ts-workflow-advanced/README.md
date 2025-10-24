# Advanced TypeScript Workflow Example

Demonstrates advanced patterns for TypeScript workflows with organized step files.

## Structure

```
ts-workflow-advanced/
├── workflow.yml          # Metadata config for UI
├── workflow.ts           # Main workflow orchestration
├── steps/                # Individual step implementations
│   ├── index.ts         # Export all steps
│   ├── openCRM.ts
│   ├── navigateToCustomers.ts
│   ├── clickNewCustomer.ts
│   ├── fillCustomerForm.ts
│   ├── validateForm.ts
│   ├── submitForm.ts
│   ├── verifySuccess.ts
│   ├── sendWelcomeEmail.ts
│   └── closeForm.ts
├── package.json
└── tsconfig.json
```

## Key Patterns

### 1. Organized Steps
Each step is in its own file for better organization and testing:

```typescript
// steps/openCRM.ts
export async function openCRM(desktop: Desktop) {
  // Step implementation
}
```

### 2. State Sharing via Context
Pass a context object between steps to share state:

```typescript
interface WorkflowContext {
  customerId?: string;
  formValid?: boolean;
  submissionTime?: Date;
}

await validateForm(desktop, context);
if (!context.formValid) {
  throw new Error('Validation failed');
}
```

### 3. Type-Safe Variables
Define interface for workflow variables matching YAML:

```typescript
interface WorkflowVariables {
  customerName: string;
  email: string;
  phone?: string;
  sendWelcomeEmail: boolean;
}
```

### 4. Conditional Execution
Handle conditional steps in main workflow:

```typescript
if (vars.sendWelcomeEmail) {
  await steps.sendWelcomeEmail(desktop, { email: vars.email });
}
```

### 5. Error Handling
Wrap execution in try-catch for better error messages:

```typescript
try {
  await steps.validateForm(desktop, context);
  if (!context.formValid) {
    throw new Error('Form validation failed');
  }
} catch (error) {
  console.error('❌ Workflow failed:', error);
  throw error;
}
```

## Running

### Install dependencies
```bash
npm install
```

### Run with default variables
```bash
tsx workflow.ts
```

### Run with custom variables
```bash
tsx workflow.ts \
  --customerName "Alice Smith" \
  --email "alice@example.com" \
  --phone "555-1234" \
  --sendWelcomeEmail true
```

## Testing Individual Steps

Each step can be tested independently:

```typescript
// test/steps/openCRM.test.ts
import { openCRM } from '../steps/openCRM';

describe('openCRM', () => {
  it('should open CRM application', async () => {
    const desktop = new Desktop();
    await openCRM(desktop);
    // Assertions
  });
});
```

## Benefits of This Approach

### Code Organization
✅ Each step is self-contained and easy to find
✅ Clear separation of concerns
✅ Easy to test individual steps
✅ Simple to add new steps

### Type Safety
✅ Interfaces for variables and context
✅ Full TypeScript type checking
✅ Autocomplete for all functions

### Maintainability
✅ Changes to one step don't affect others
✅ Clear function signatures
✅ Easy to debug individual steps
✅ Simple error handling

### UI Integration
✅ YAML clearly defines step order
✅ Function names map to implementations
✅ Variable requirements are explicit
✅ Conditional steps are marked
