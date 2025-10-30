#!/usr/bin/env tsx
/**
 * SAP Login Workflow Example
 *
 * Demonstrates:
 * - expect() for validation
 * - onError() for step-level error recovery
 * - Workflow-level onError for business logic
 * - ExecutionResponse with proper status and error categorization
 */

import { createStep, createWorkflow, z, type ExecutionResponse } from '../../packages/terminator-workflow/src';

// ============================================================================
// Input Schema
// ============================================================================

const InputSchema = z.object({
  username: z.string().min(1, 'Username is required'),
  password: z.string().min(1, 'Password is required'),
  company: z.string().default('ACME_CORP'),
});

type Input = z.infer<typeof InputSchema>;

// ============================================================================
// Steps
// ============================================================================

const login = createStep({
  id: 'login',
  name: 'Login to SAP',
  description: 'Fills in credentials and clicks login',

  execute: async ({ desktop, input, logger }) => {
    logger.info(`🔐 Logging in as ${input.username}...`);

    // Fill username
    const usernameField = await desktop.locator('role:Edit|name:User').first(5000);
    await usernameField.typeText(input.username);

    // Fill password
    const passwordField = await desktop.locator('role:Edit|name:Password').first(5000);
    await passwordField.typeText(input.password);

    // Click login
    const loginButton = await desktop.locator('role:Button|name:Log On').first(5000);
    await loginButton.click();

    await desktop.delay(3000);
  },

  expect: async ({ desktop, logger }) => {
    // Check that login was successful - look for home screen element
    logger.info('🔍 Verifying login success...');

    const homeCheck = await desktop.locator('role:Window|name:SAP Business One').validate(5000);

    if (!homeCheck.exists) {
      return {
        success: false,
        message: 'Login failed - SAP home screen not found',
      };
    }

    return {
      success: true,
      message: 'Successfully logged in to SAP',
    };
  },

  onError: async ({ error, desktop, retry, logger }) => {
    logger.warn(`⚠️  Login error: ${error.message}`);

    // Check for session conflict dialog
    const sessionDialogCheck = await desktop
      .locator('role:Dialog|name:Session Conflict')
      .validate(1000);

    if (sessionDialogCheck.exists) {
      logger.info('🔄 Session conflict detected - closing existing session...');

      const closeButton = await desktop.locator('role:Button|name:Close Session').first(2000);
      await closeButton.click();
      await desktop.delay(2000);

      // Retry the login
      return retry();
    }

    // Check for invalid credentials
    const errorDialogCheck = await desktop
      .locator('role:Dialog|name:Invalid Credentials')
      .validate(1000);

    if (errorDialogCheck.exists) {
      return {
        recoverable: false,
        reason: 'Invalid credentials - cannot retry',
      };
    }

    // Unknown error - not recoverable
    return {
      recoverable: false,
      reason: `Unknown login error: ${error.message}`,
    };
  },
});

const selectCompany = createStep({
  id: 'select-company',
  name: 'Select Company',
  description: 'Selects the company database',

  execute: async ({ desktop, input, logger }) => {
    logger.info(`🏢 Selecting company: ${input.company}...`);

    // Open company dropdown
    const companyDropdown = await desktop.locator('role:ComboBox|name:Company').first(5000);
    await companyDropdown.click();
    await desktop.delay(500);

    // Select company from list
    const companyItem = await desktop.locator(`role:ListItem|name:${input.company}`).first(3000);
    await companyItem.click();

    await desktop.delay(2000);
  },

  expect: async ({ desktop, input, logger }) => {
    logger.info('🔍 Verifying company selection...');

    // Check that the main application window is now open
    const mainWindowCheck = await desktop
      .locator(`role:Window|name:${input.company} - SAP Business One`)
      .validate(5000);

    if (!mainWindowCheck.exists) {
      return {
        success: false,
        message: `Company ${input.company} window not found`,
      };
    }

    return {
      success: true,
      message: `Company ${input.company} selected successfully`,
    };
  },
});

// ============================================================================
// Workflow
// ============================================================================

const workflow = createWorkflow({
  name: 'SAP Login',
  description: 'Login to SAP Business One and select company',
  version: '1.0.0',
  input: InputSchema,

  // Workflow-level error handler
  onError: async ({ error, step, logger }): Promise<ExecutionResponse> => {
    logger.error(`🚨 Workflow failed at step: ${step.config.name}`);

    // Categorize the error
    const errorMessage = error.message.toLowerCase();

    if (errorMessage.includes('invalid credentials') || errorMessage.includes('login failed')) {
      return {
        status: 'error',
        error: {
          category: 'business',
          code: 'INVALID_CREDENTIALS',
          message: 'Login failed - please check your username and password',
        },
        message: 'Authentication failed',
      };
    }

    if (errorMessage.includes('company') || errorMessage.includes('database')) {
      return {
        status: 'error',
        error: {
          category: 'business',
          code: 'COMPANY_NOT_FOUND',
          message: 'Company database not found or inaccessible',
        },
        message: 'Company selection failed',
      };
    }

    if (errorMessage.includes('timeout') || errorMessage.includes('not found')) {
      return {
        status: 'error',
        error: {
          category: 'technical',
          code: 'UI_ELEMENT_NOT_FOUND',
          message: 'SAP UI element not found - application may be slow or changed',
        },
        message: 'Technical error - UI automation failed',
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
      message: 'An unexpected error occurred',
    };
  },
})
  .step(login)
  .step(selectCompany)
  .build();

// ============================================================================
// Execute (CLI)
// ============================================================================

if (require.main === module) {
  const input: Input = {
    username: process.argv[2] || 'demo_user',
    password: process.argv[3] || 'demo_pass',
    company: 'ACME_CORP',
  };

  workflow.run(input).then(response => {
    console.log('\n📊 Workflow Response:');
    console.log(JSON.stringify(response, null, 2));

    if (response.status === 'error') {
      console.error('\n❌ Workflow failed');
      process.exit(1);
    }
  });
}

export default workflow;
