# TypeScript Workflow Proposal

## Goal
Create a simple, parseable way to define workflows in TypeScript projects with a YAML metadata config file for UI rendering.

## Key Requirements
1. ✅ TypeScript projects with clear function/step definitions
2. ✅ YAML config file with metadata for mediar-app UI parsing
3. ✅ Simple DX - no complex builders or abstractions
4. ✅ Easy to understand workflow structure
5. ✅ UI can parse and render workflow steps properly

## Proposed Structure

### Directory Layout
```
my-workflow/
├── workflow.yml          # Metadata config for UI
├── workflow.ts           # Main workflow logic
├── steps/                # Optional: organize steps
│   ├── login.ts
│   ├── fillForm.ts
│   └── submit.ts
├── package.json
└── tsconfig.json
```

### 1. Metadata Config (workflow.yml)
Simple YAML file that describes the workflow for UI parsing:

```yaml
id: customer-onboarding
name: Customer Onboarding Workflow
description: Automates customer data entry in CRM
version: 1.0.0
tags:
  - crm
  - onboarding
  - data-entry

variables:
  customerName:
    type: string
    label: Customer Name
    description: Full name of the customer
    required: true

  email:
    type: string
    label: Email Address
    required: true

  sendWelcomeEmail:
    type: boolean
    label: Send Welcome Email
    default: true

steps:
  - id: open-crm
    name: Open CRM Application
    function: openCRM
    description: Launches the CRM and waits for it to be ready

  - id: navigate-to-customers
    name: Navigate to Customers
    function: navigateToCustomers
    description: Opens the customer management section

  - id: fill-customer-form
    name: Fill Customer Details
    function: fillCustomerForm
    description: Populates the new customer form
    inputs:
      - customerName
      - email

  - id: submit-form
    name: Submit Form
    function: submitForm
    description: Saves the customer record

  - id: send-welcome-email
    name: Send Welcome Email
    function: sendWelcomeEmail
    condition: variables.sendWelcomeEmail === true
    description: Optionally sends welcome email
    inputs:
      - email
```

### 2. Workflow Implementation (workflow.ts)

Simple TypeScript file with exported functions matching the YAML config:

```typescript
import { Desktop } from '@mediar/terminator';

// Each function corresponds to a step in workflow.yml
export async function openCRM(desktop: Desktop) {
  console.log('🚀 Opening CRM...');
  desktop.openApplication('crm.exe');
  await desktop.locator('role:Window|name:CRM').waitFor('visible', 5000);
}

export async function navigateToCustomers(desktop: Desktop) {
  console.log('📂 Navigating to Customers...');
  await desktop.locator('role:button|name:Customers').click();
  await desktop.locator('role:button|name:New Customer').waitFor('visible', 3000);
}

export async function fillCustomerForm(
  desktop: Desktop,
  variables: { customerName: string; email: string }
) {
  console.log('📝 Filling customer form...');

  await desktop.locator('role:textbox|name:Name').fill(variables.customerName);
  await desktop.locator('role:textbox|name:Email').fill(variables.email);
  await desktop.locator('role:combobox|name:Status').selectOption('Active');
}

export async function submitForm(desktop: Desktop) {
  console.log('💾 Submitting form...');
  await desktop.locator('role:button|name:Save').click();
  await desktop.locator('text:Customer saved successfully').waitFor('visible', 5000);
}

export async function sendWelcomeEmail(
  desktop: Desktop,
  variables: { email: string }
) {
  console.log('📧 Sending welcome email...');
  await desktop.locator('role:button|name:Send Email').click();
  console.log(`✅ Welcome email sent to ${variables.email}`);
}

// Main entry point for direct execution
export async function main(variables: Record<string, any> = {}) {
  const desktop = new Desktop();

  await openCRM(desktop);
  await navigateToCustomers(desktop);
  await fillCustomerForm(desktop, variables);
  await submitForm(desktop);

  if (variables.sendWelcomeEmail) {
    await sendWelcomeEmail(desktop, variables);
  }

  console.log('✅ Workflow completed successfully!');
}

if (require.main === module) {
  const args = parseCliArgs();
  main(args).catch(console.error);
}

function parseCliArgs(): Record<string, any> {
  // Simple arg parsing
  const args = process.argv.slice(2);
  const params: Record<string, any> = {};
  for (let i = 0; i < args.length; i += 2) {
    const key = args[i].replace(/^--/, '');
    params[key] = args[i + 1];
  }
  return params;
}
```

## Benefits

### For Developers
- ✅ **Simple & Clear**: Just export functions, no complex builders
- ✅ **Standard TypeScript**: Use regular async functions
- ✅ **Type Safety**: Full TypeScript support
- ✅ **Easy Testing**: Each function can be tested independently
- ✅ **No Magic**: No decorators, no build steps, no abstractions

### For mediar-app UI
- ✅ **Easy Parsing**: YAML is simple to parse
- ✅ **Clear Structure**: Steps, variables, and functions are explicit
- ✅ **UI Rendering**: All metadata available for rendering
- ✅ **Step Mapping**: Function names map directly to step IDs
- ✅ **Variable Inputs**: UI knows which steps need which variables

## Execution

### From CLI
```bash
# Direct execution
tsx workflow.ts --customerName "John Doe" --email "john@example.com"

# Via Terminator CLI
terminator-cli execute-workflow --url file://./workflow.yml
```

### From mediar-app
1. Parse `workflow.yml` to display workflow info
2. Show variables as form inputs
3. Render steps from YAML metadata
4. Execute by running `tsx workflow.ts` with variables

## Implementation Plan

1. **YAML Parser** - Parse workflow.yml to get metadata
2. **Function Resolver** - Map YAML steps to exported TS functions
3. **Variable Injection** - Pass variables to functions that need them
4. **Step Executor** - Execute functions in order with proper context
5. **UI Integration** - mediar-app displays and runs workflows

## Alternative: Simpler Function Naming Convention

Instead of explicit YAML mapping, use naming convention:

```typescript
// workflow.ts
export async function step1_openCRM(desktop: Desktop) { }
export async function step2_navigateToCustomers(desktop: Desktop) { }
export async function step3_fillCustomerForm(desktop: Desktop, vars: any) { }
```

```yaml
# workflow.yml
steps:
  - openCRM
  - navigateToCustomers
  - fillCustomerForm
```

## Questions to Explore

1. Should steps be in separate files or one file?
2. Should variables be passed to all functions or only those that need them?
3. Should we support conditions in YAML or handle in TS?
4. Should we have a standard interface for step functions?
5. How to handle step dependencies and state sharing?

## Next Steps

- [ ] Create minimal working example
- [ ] Build YAML parser
- [ ] Build function executor
- [ ] Test with mediar-app UI
- [ ] Document patterns and best practices
