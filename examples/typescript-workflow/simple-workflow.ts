#!/usr/bin/env tsx
/**
 * Simple TypeScript Workflow Example
 *
 * No YAML. Just TypeScript. Fully typed. AI-friendly.
 */

import { createStep, createWorkflow, Desktop } from '@mediar/terminator';
import { z } from 'zod';

// ============================================================================
// Input Schema (Using Zod)
// ============================================================================

const InputSchema = z.object({
  userName: z
    .string()
    .default('World')
    .describe('User name to greet'),

  includeDate: z
    .boolean()
    .default(true)
    .describe('Include current date in message'),
});

type Input = z.infer<typeof InputSchema>;

// ============================================================================
// Steps
// ============================================================================

const openNotepad = createStep({
  id: 'open-notepad',
  name: 'Open Notepad',
  description: 'Opens Notepad application',

  execute: async ({ desktop, logger }) => {
    logger.info('📝 Opening Notepad...');
    desktop.openApplication('notepad');
    await desktop.wait(2000);
    logger.success('✅ Notepad opened');
  },
});

const typeGreeting = createStep({
  id: 'type-greeting',
  name: 'Type Greeting Message',
  description: 'Types personalized greeting',

  execute: async ({ desktop, input, logger }: {
    desktop: Desktop;
    input: Input;
    logger: any;
  }) => {
    logger.info(`👋 Typing greeting for ${input.userName}...`);

    const textbox = desktop.locator('role:Edit');
    await textbox.type(`Hello, ${input.userName}!\n\n`);

    if (input.includeDate) {
      const date = new Date().toLocaleDateString();
      await textbox.type(`Date: ${date}\n`);
    }

    logger.success('✅ Greeting typed');
  },
});

// ============================================================================
// Workflow
// ============================================================================

export default createWorkflow({
  name: 'Simple Notepad Demo',
  description: 'Opens Notepad and types a personalized greeting',
  version: '1.0.0',

  // Type-safe input schema
  input: InputSchema,
})
  .step(openNotepad)
  .step(typeGreeting)
  .build();

// ============================================================================
// Execute (CLI)
// ============================================================================

if (require.main === module) {
  const input: Input = {
    userName: process.argv[2] || 'World',
    includeDate: true,
  };

  console.log('🚀 Running workflow...');
  console.log('Input:', input);

  workflow.run(input).then(() => {
    console.log('✅ Workflow completed!');
  }).catch(error => {
    console.error('❌ Workflow failed:', error);
    process.exit(1);
  });
}
