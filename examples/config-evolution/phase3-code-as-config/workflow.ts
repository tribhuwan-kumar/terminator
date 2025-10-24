#!/usr/bin/env tsx
/**
 * Phase 3: Code as Config
 *
 * Everything in one file - code IS config
 *
 * Like: Mastra, Inngest, Temporal
 */

import { createWorkflow, createStep, Desktop } from '@mediar/terminator';
import pkg from './package.json';

// Steps are defined inline
const openNotepad = createStep({
  id: 'open-notepad',
  name: 'Open Notepad',
  description: 'Launches Notepad application',

  execute: async ({ desktop, logger }) => {
    logger.info('📝 Opening Notepad...');
    desktop.openApplication('notepad');
    await new Promise(r => setTimeout(r, 2000));
  },
});

const typeGreeting = createStep({
  id: 'type-greeting',
  name: 'Type Greeting',
  description: 'Types personalized greeting',

  execute: async ({ desktop, variables, logger }) => {
    const userName = variables.userName || 'World';
    logger.info(`👋 Typing greeting for ${userName}...`);

    const textbox = desktop.locator('role:Edit');
    await textbox.type(`Hello, ${userName}!\n\n`);
  },
});

const addCustomMessage = createStep({
  id: 'add-message',
  name: 'Add Custom Message',
  description: 'Adds optional custom message',

  // Condition can be a function
  condition: ({ variables }) => Boolean(variables.message),

  execute: async ({ desktop, variables, logger }) => {
    logger.info(`💬 Adding custom message...`);

    const textbox = desktop.locator('role:Edit');
    await textbox.type(`${variables.message}\n`);
  },
});

const addEnvironment = createStep({
  id: 'add-environment',
  name: 'Add Environment Info',
  description: 'Adds environment information',

  execute: async ({ desktop, variables, logger }) => {
    logger.info(`🌍 Adding environment info...`);

    const env = variables.environment || process.env.NODE_ENV || 'development';

    const textbox = desktop.locator('role:Edit');
    await textbox.type(`\nEnvironment: ${env}\n`);
    await textbox.type(`Version: ${pkg.version}\n`);
    await textbox.type(`Node: ${process.version}\n`);
    await textbox.type(`Platform: ${process.platform}\n`);
    await textbox.type(`Time: ${new Date().toISOString()}\n`);
  },
});

// Workflow definition
export const workflow = createWorkflow({
  id: 'notepad-demo',
  name: 'Notepad Demo Workflow',
  description: 'Demonstrating Phase 3 Code as Config',

  // Can reference package.json
  version: pkg.version,

  // Can use computed values
  tags: [
    'demo',
    'phase3',
    'code-as-config',
    process.env.NODE_ENV || 'development'
  ],

  // Variables with computed defaults
  variables: {
    userName: {
      type: 'string',
      label: 'User Name',
      description: 'Name to greet',
      default: process.env.DEFAULT_USER || 'World',
      required: false,
    },

    message: {
      type: 'string',
      label: 'Custom Message',
      description: 'Optional custom message',
      required: false,
    },

    environment: {
      type: 'string',
      label: 'Environment',
      default: process.env.NODE_ENV || 'development',
      required: false,
    },
  },
})
  // Chain steps
  .step(openNotepad)
  .step(typeGreeting)
  .step(addCustomMessage) // Only runs if condition is met
  .step(addEnvironment)

  // Success handler
  .onSuccess(async ({ logger }) => {
    logger.success('✅ Workflow completed successfully!');
  })

  // Error handler
  .onError(async ({ error, step, logger }) => {
    logger.error(`❌ Failed at step: ${step.name}`);
    logger.error(`   Error: ${error.message}`);
  });

// Execute if run directly
if (require.main === module) {
  const args = process.argv.slice(2);
  const vars: Record<string, any> = {};

  for (let i = 0; i < args.length; i += 2) {
    const key = args[i].replace(/^--/, '');
    vars[key] = args[i + 1];
  }

  console.log('='.repeat(60));
  console.log('Phase 3: Code as Config');
  console.log('Variables:', vars);
  console.log('='.repeat(60));

  workflow.run(vars).catch(error => {
    console.error('\n❌ Workflow failed:', error);
    process.exit(1);
  });
}

// Export for external usage
export default workflow;
