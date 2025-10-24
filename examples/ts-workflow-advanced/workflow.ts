#!/usr/bin/env tsx
/**
 * CRM Customer Entry Workflow
 *
 * Demonstrates advanced workflow patterns:
 * - Organized steps in separate files
 * - State sharing between steps via context
 * - Type-safe variables
 * - Error handling
 * - Conditional execution
 */

import { Desktop } from '@mediar/terminator';
import * as steps from './steps';

/**
 * Workflow context for sharing state between steps
 */
interface WorkflowContext {
  customerId?: string;
  formValid?: boolean;
  submissionTime?: Date;
}

/**
 * Workflow variables (matches workflow.yml)
 */
interface WorkflowVariables {
  customerName: string;
  email: string;
  phone?: string;
  sendWelcomeEmail: boolean;
}

/**
 * Main workflow execution
 */
export async function main(variables: Partial<WorkflowVariables> = {}) {
  // Merge with defaults
  const vars: WorkflowVariables = {
    customerName: variables.customerName || 'John Doe',
    email: variables.email || 'john@example.com',
    phone: variables.phone,
    sendWelcomeEmail: variables.sendWelcomeEmail ?? true,
  };

  console.log('='.repeat(60));
  console.log('CRM Customer Entry Workflow');
  console.log('Variables:', vars);
  console.log('='.repeat(60));
  console.log('');

  const desktop = new Desktop();
  const context: WorkflowContext = {};

  try {
    // Step 1: Open CRM
    await steps.openCRM(desktop);

    // Step 2: Navigate to customers
    await steps.navigateToCustomers(desktop);

    // Step 3: Click new customer button
    await steps.clickNewCustomer(desktop);

    // Step 4: Fill customer form
    await steps.fillCustomerForm(desktop, vars);

    // Step 5: Validate form
    await steps.validateForm(desktop, context);
    if (!context.formValid) {
      throw new Error('Form validation failed');
    }

    // Step 6: Submit form
    await steps.submitForm(desktop);

    // Step 7: Verify success
    await steps.verifySuccess(desktop, context);
    console.log(`✅ Customer created with ID: ${context.customerId}`);

    // Step 8: Send welcome email (conditional)
    if (vars.sendWelcomeEmail) {
      await steps.sendWelcomeEmail(desktop, { email: vars.email });
    }

    // Step 9: Close form
    await steps.closeForm(desktop);

    console.log('');
    console.log('='.repeat(60));
    console.log('✅ Workflow completed successfully!');
    console.log('='.repeat(60));

  } catch (error) {
    console.error('');
    console.error('='.repeat(60));
    console.error('❌ Workflow failed:', error);
    console.error('='.repeat(60));
    throw error;
  }
}

/**
 * Parse CLI arguments
 */
function parseCliArgs(): Partial<WorkflowVariables> {
  const args = process.argv.slice(2);
  const params: Record<string, any> = {};

  for (let i = 0; i < args.length; i += 2) {
    const key = args[i].replace(/^--/, '');
    const value = args[i + 1];

    if (key && value !== undefined) {
      // Parse booleans
      if (value === 'true') params[key] = true;
      else if (value === 'false') params[key] = false;
      else params[key] = value;
    }
  }

  return params;
}

// Execute if run directly
if (require.main === module) {
  const variables = parseCliArgs();
  main(variables).catch(error => {
    console.error('\n❌ Workflow execution failed');
    process.exit(1);
  });
}

// Export for external usage
export { WorkflowContext, WorkflowVariables };
