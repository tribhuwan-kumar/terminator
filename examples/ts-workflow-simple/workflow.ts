#!/usr/bin/env tsx
/**
 * Simple Notepad Workflow
 *
 * This demonstrates the proposed TypeScript workflow approach:
 * - workflow.yml defines metadata and step structure
 * - workflow.ts exports functions matching the YAML steps
 * - Each function is a simple async function with Desktop + variables
 */

import { Desktop } from '@mediar/terminator';

/**
 * Step 1: Open Notepad
 */
export async function openNotepad(desktop: Desktop) {
  console.log('📝 Opening Notepad...');
  desktop.openApplication('notepad');
  await new Promise(r => setTimeout(r, 2000));
}

/**
 * Step 2: Type Greeting
 */
export async function typeGreeting(
  desktop: Desktop,
  variables: { userName: string }
) {
  console.log(`👋 Typing greeting for ${variables.userName}...`);

  const textbox = desktop.locator('role:Edit');
  await textbox.type(`Hello, ${variables.userName}!\n\n`);
  await textbox.type('This is a TypeScript workflow example.\n');
  await textbox.type('Simple functions, clear structure!\n\n');
}

/**
 * Step 3: Add Date (conditional)
 */
export async function addDate(desktop: Desktop) {
  console.log('📅 Adding date...');

  const textbox = desktop.locator('role:Edit');
  const date = new Date().toLocaleDateString();
  await textbox.type(`Date: ${date}\n`);
}

/**
 * Main entry point for direct execution
 */
export async function main(variables: Record<string, any> = {}) {
  const { userName = 'World', includeDate = true } = variables;

  console.log('='.repeat(60));
  console.log('Simple Notepad Workflow');
  console.log('Variables:', { userName, includeDate });
  console.log('='.repeat(60));
  console.log('');

  const desktop = new Desktop();

  await openNotepad(desktop);
  await typeGreeting(desktop, { userName });

  if (includeDate) {
    await addDate(desktop);
  }

  console.log('');
  console.log('✅ Workflow completed successfully!');
}

/**
 * Parse CLI arguments
 */
function parseCliArgs(): Record<string, any> {
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
    console.error('\n❌ Workflow failed:', error);
    process.exit(1);
  });
}
