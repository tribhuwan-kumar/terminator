#!/usr/bin/env tsx
/**
 * Production-Ready Workflow with AI Recovery
 *
 * Demonstrates the proposed DX inspired by Vercel/Mastra/Inngest
 * Optimized for:
 * - AI code generation
 * - Reliability and error recovery
 * - Heuristic fallbacks
 * - State management
 */

import { createWorkflow, createStep, Desktop } from '@mediar/terminator-workflow';

// ============================================================================
// Step 1: Login with AI Recovery
// ============================================================================

const loginToApp = createStep({
  id: 'login-to-app',
  name: 'Login to Application',
  description: 'Authenticate with the application',
  timeout: 30000,

  execute: async ({ desktop, context, logger }) => {
    logger.info('🔐 Attempting login...');

    await desktop.locator('role:textbox|name:Username').fill('user@example.com');
    await desktop.locator('role:textbox|name:Password').fill('password123');
    await desktop.locator('role:button|name:Sign In').click();

    // Wait for dashboard or error message
    const result = await desktop.waitForAny([
      { selector: 'role:heading|name:Dashboard', name: 'dashboard' },
      { selector: 'text:Invalid credentials', name: 'error' },
      { selector: 'text:Session conflict', name: 'conflict' },
    ], { timeout: 10000 });

    if (result.matched === 'dashboard') {
      logger.success('✅ Login successful');
      return { success: true, sessionId: 'abc123' };
    }

    // Failed - throw error for recovery
    throw new Error(`Login failed: ${result.matched}`);
  },

  // AI Recovery Function - called when execute() fails
  onError: async ({ error, desktop, context, logger, retry }) => {
    logger.warn(`⚠️ Login failed: ${error.message}`);

    // Classify error type
    if (error.message.includes('conflict')) {
      logger.info('🔄 Session conflict detected - attempting recovery...');

      // Try to close existing session
      const closeBtn = desktop.locator('role:button|name:Close Other Session');
      if (await closeBtn.exists()) {
        await closeBtn.click();
        await desktop.wait(2000);

        // Retry login
        return retry();
      }
    }

    if (error.message.includes('Invalid credentials')) {
      // Permanent error - don't retry
      logger.error('❌ Invalid credentials - cannot recover');
      return {
        recoverable: false,
        reason: 'Invalid credentials'
      };
    }

    // Try AI-powered recovery
    logger.info('🤖 Attempting AI-powered recovery...');

    const screenshot = await desktop.screenshot();
    const tree = await desktop.getAccessibilityTree();

    const aiSuggestion = await context.ai.analyze({
      screenshot,
      tree,
      goal: 'Login to the application',
      error: error.message,
    });

    if (aiSuggestion.canRecover) {
      logger.info(`🤖 AI suggests: ${aiSuggestion.action}`);

      // Execute AI's suggested recovery steps
      for (const step of aiSuggestion.steps) {
        await desktop.locator(step.selector)[step.action](step.value);
      }

      return retry();
    }

    // Cannot recover
    return {
      recoverable: false,
      reason: error.message,
      aiAttempted: true
    };
  },

  // Heuristic fallback - simpler recovery logic
  fallback: async ({ desktop, logger }) => {
    logger.info('🔧 Trying heuristic fallback...');

    // Simple heuristic: close app and reopen
    await desktop.closeApplication('app.exe');
    await desktop.wait(2000);
    await desktop.openApplication('app.exe');
    await desktop.wait(5000);

    // Try login again
    await desktop.locator('role:textbox|name:Username').fill('user@example.com');
    await desktop.locator('role:textbox|name:Password').fill('password123');
    await desktop.locator('role:button|name:Sign In').click();

    const success = await desktop.locator('role:heading|name:Dashboard').exists({ timeout: 10000 });

    return { success, method: 'heuristic_restart' };
  },
});

// ============================================================================
// Step 2: Data Processing with Classification
// ============================================================================

const processData = createStep({
  id: 'process-data',
  name: 'Process JSON Data',
  description: 'Read and validate JSON file',

  execute: async ({ desktop, variables, context, logger }) => {
    logger.info('📄 Reading JSON file...');

    const filePath = variables.jsonFile as string;
    const fs = await import('fs/promises');

    try {
      const content = await fs.readFile(filePath, 'utf-8');
      const data = JSON.parse(content);

      // Validate required fields
      if (!data.outlet_code || !data.date || !data.entries) {
        throw new Error('Missing required fields in JSON');
      }

      // Store in context for next steps
      context.data = data;

      logger.success(`✅ Loaded ${data.entries.length} entries`);

      return {
        success: true,
        entriesCount: data.entries.length,
        outletCode: data.outlet_code
      };

    } catch (error: any) {
      if (error instanceof SyntaxError) {
        // JSON parse error - permanent
        throw new Error('JSON_PARSE_ERROR: Invalid JSON format');
      }
      throw error;
    }
  },

  // Error classification
  classifyError: (error: Error) => {
    const permanentPatterns = [
      'JSON_PARSE_ERROR',
      'Missing required fields',
      'Invalid outlet',
      'Permission denied',
    ];

    const temporaryPatterns = [
      'ENOENT', // File not found - might appear later
      'EACCES', // Permission - might be temporary
      'Network error',
    ];

    for (const pattern of permanentPatterns) {
      if (error.message.includes(pattern)) {
        return {
          type: 'permanent',
          shouldRetry: false,
          reason: error.message
        };
      }
    }

    for (const pattern of temporaryPatterns) {
      if (error.message.includes(pattern)) {
        return {
          type: 'temporary',
          shouldRetry: true,
          maxRetries: 3,
          reason: error.message
        };
      }
    }

    return {
      type: 'unknown',
      shouldRetry: true,
      maxRetries: 1,
      reason: error.message
    };
  },

  onError: async ({ error, context, logger, retry, attempt }) => {
    const classification = processData.classifyError!(error);

    logger.warn(`⚠️ Error classified as: ${classification.type}`);

    if (classification.type === 'permanent') {
      logger.error(`❌ Permanent error: ${classification.reason}`);
      return { recoverable: false, reason: classification.reason };
    }

    if (attempt < (classification.maxRetries || 1)) {
      logger.info(`🔄 Retrying (attempt ${attempt + 1}/${classification.maxRetries})...`);
      await context.wait(2000 * attempt); // Exponential backoff
      return retry();
    }

    logger.error(`❌ Max retries exceeded`);
    return { recoverable: false, reason: 'Max retries exceeded' };
  },
});

// ============================================================================
// Step 3: Fill Form with Smart Recovery
// ============================================================================

const fillForm = createStep({
  id: 'fill-form',
  name: 'Fill Entry Form',
  description: 'Populate form with data from JSON',

  execute: async ({ desktop, context, logger }) => {
    const data = context.data;
    logger.info(`📝 Filling form for ${data.outlet_code}...`);

    await desktop.locator('role:textbox|name:Outlet').fill(data.outlet_code);
    await desktop.locator('role:textbox|name:Date').fill(data.date);

    // Fill entries table
    for (let i = 0; i < data.entries.length; i++) {
      const entry = data.entries[i];
      logger.info(`  Row ${i + 1}/${data.entries.length}`);

      await desktop.locator(`role:cell[row:${i}][col:account]`).fill(entry.account);
      await desktop.locator(`role:cell[row:${i}][col:debit]`).fill(entry.debit);
      await desktop.locator(`role:cell[row:${i}][col:credit]`).fill(entry.credit);
    }

    logger.success('✅ Form filled');
    return { success: true, rowsFilled: data.entries.length };
  },

  onError: async ({ error, desktop, context, logger, retry }) => {
    logger.warn(`⚠️ Form fill error: ${error.message}`);

    // Check if element not found
    if (error.message.includes('not found')) {
      logger.info('🔍 Element not found - checking UI state...');

      // Take screenshot for AI analysis
      const screenshot = await desktop.screenshot();
      const tree = await desktop.getAccessibilityTree();

      // AI analyzes what went wrong
      const analysis = await context.ai.analyze({
        screenshot,
        tree,
        goal: 'Fill the form with data',
        error: error.message,
        context: { data: context.data }
      });

      if (analysis.canRecover) {
        logger.info(`🤖 AI recovery: ${analysis.explanation}`);

        // AI might suggest clicking a button first, closing a popup, etc.
        for (const step of analysis.recoverySteps) {
          await desktop.execute(step);
        }

        return retry();
      }
    }

    return { recoverable: false, reason: error.message };
  },
});

// ============================================================================
// Step 4: Verify with Adjustments
// ============================================================================

const verifyAndAdjust = createStep({
  id: 'verify-and-adjust',
  name: 'Verify Totals',
  description: 'Check balance and add adjustments if needed',

  execute: async ({ desktop, context, logger }) => {
    logger.info('🔍 Verifying totals...');

    // Read table totals
    const debitTotal = await desktop.locator('role:cell|name:Debit Total').text();
    const creditTotal = await desktop.locator('role:cell|name:Credit Total').text();

    const debit = parseFloat(debitTotal);
    const credit = parseFloat(creditTotal);
    const difference = Math.abs(debit - credit);

    logger.info(`  Debit: ${debit}, Credit: ${credit}`);
    logger.info(`  Difference: ${difference}`);

    if (difference < 0.01) {
      logger.success('✅ Table balanced');
      return { balanced: true, difference: 0 };
    }

    // Need adjustment
    logger.warn(`⚠️ Table not balanced - adding adjustment entry...`);

    const adjustmentAmount = difference;
    const needsDebit = credit > debit;

    // Add adjustment row
    const lastRow = context.data.entries.length;
    await desktop.locator(`role:cell[row:${lastRow}][col:account]`).fill('441500');
    await desktop.locator(`role:cell[row:${lastRow}][col:${needsDebit ? 'debit' : 'credit'}]`).fill(adjustmentAmount.toFixed(2));
    await desktop.locator(`role:cell[row:${lastRow}][col:remarks]`).fill('GST Rounding Adjustment');

    logger.success(`✅ Added adjustment: ${adjustmentAmount.toFixed(2)} to ${needsDebit ? 'debit' : 'credit'}`);

    return {
      balanced: true,
      adjustmentAdded: true,
      adjustmentAmount
    };
  },
});

// ============================================================================
// Workflow Definition
// ============================================================================

export const workflow = createWorkflow({
  id: 'production-workflow',
  name: 'Production Workflow with AI Recovery',
  description: 'Demonstrates advanced error handling and AI recovery',
  version: '1.0.0',

  variables: {
    jsonFile: {
      type: 'string',
      label: 'JSON File Path',
      required: true,
    },
  },
})
  .step(loginToApp)
  .step(processData)
  .step(fillForm)
  .step(verifyAndAdjust)

  // Global error handler
  .onError(async ({ error, step, context, logger }) => {
    logger.error(`❌ Workflow failed at step: ${step.name}`);
    logger.error(`   Error: ${error.message}`);

    // Move file to failed folder
    const fs = await import('fs/promises');
    const path = await import('path');

    const fileName = path.basename(context.variables.jsonFile as string);
    const failedPath = path.join(path.dirname(context.variables.jsonFile as string), 'failed', fileName);

    await fs.mkdir(path.dirname(failedPath), { recursive: true });
    await fs.rename(context.variables.jsonFile as string, failedPath);

    // Write error metadata
    await fs.writeFile(failedPath + '_meta.json', JSON.stringify({
      error: error.message,
      step: step.id,
      timestamp: new Date().toISOString(),
      stack: error.stack,
    }, null, 2));

    logger.info(`📁 Moved to failed folder: ${failedPath}`);
  })

  // Success handler
  .onSuccess(async ({ context, logger }) => {
    logger.success('✅ Workflow completed successfully!');

    // Move file to processed folder
    const fs = await import('fs/promises');
    const path = await import('path');

    const fileName = path.basename(context.variables.jsonFile as string);
    const processedPath = path.join(path.dirname(context.variables.jsonFile as string), 'processed', fileName);

    await fs.mkdir(path.dirname(processedPath), { recursive: true });
    await fs.rename(context.variables.jsonFile as string, processedPath);

    logger.info(`📁 Moved to processed folder: ${processedPath}`);
  });

// ============================================================================
// Execute
// ============================================================================

if (require.main === module) {
  const jsonFile = process.argv[2] || './data.json';

  workflow.run({ jsonFile }).catch(error => {
    console.error('❌ Workflow execution failed:', error);
    process.exit(1);
  });
}
