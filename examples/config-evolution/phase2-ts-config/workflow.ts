#!/usr/bin/env tsx
/**
 * Phase 2: TypeScript Config
 *
 * Implementation file - similar to Phase 1
 * But config is now in terminator.config.ts with type safety
 */

import { Desktop } from '@mediar/terminator';

export async function openNotepad(desktop: Desktop) {
  console.log('📝 Opening Notepad...');
  desktop.openApplication('notepad');
  await new Promise(r => setTimeout(r, 2000));
}

export async function typeGreeting(
  desktop: Desktop,
  variables: { userName: string }
) {
  console.log(`👋 Typing greeting for ${variables.userName}...`);

  const textbox = desktop.locator('role:Edit');
  await textbox.type(`Hello, ${variables.userName}!\n\n`);
}

export async function addCustomMessage(
  desktop: Desktop,
  variables: { message: string }
) {
  console.log(`💬 Adding custom message...`);

  const textbox = desktop.locator('role:Edit');
  await textbox.type(`${variables.message}\n`);
}

export async function addEnvironment(
  desktop: Desktop,
  variables: { environment: string }
) {
  console.log(`🌍 Adding environment info...`);

  const textbox = desktop.locator('role:Edit');
  await textbox.type(`\nEnvironment: ${variables.environment}\n`);
  await textbox.type(`Build time: ${new Date().toISOString()}\n`);
}

// Main entry point
export async function main(variables: Record<string, any> = {}) {
  const {
    userName = process.env.DEFAULT_USER || 'World',
    message = '',
    environment = process.env.NODE_ENV || 'development'
  } = variables;

  console.log('='.repeat(60));
  console.log('Phase 2: TypeScript Config');
  console.log('Variables:', { userName, message, environment });
  console.log('='.repeat(60));

  const desktop = new Desktop();

  await openNotepad(desktop);
  await typeGreeting(desktop, { userName });

  if (message) {
    await addCustomMessage(desktop, { message });
  }

  await addEnvironment(desktop, { environment });

  console.log('✅ Done!');
}

if (require.main === module) {
  const args = process.argv.slice(2);
  const vars: Record<string, any> = {};

  for (let i = 0; i < args.length; i += 2) {
    const key = args[i].replace(/^--/, '');
    vars[key] = args[i + 1];
  }

  main(vars).catch(console.error);
}
