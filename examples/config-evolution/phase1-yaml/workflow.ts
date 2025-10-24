#!/usr/bin/env tsx
/**
 * Phase 1: YAML + TypeScript
 *
 * Config: workflow.yml (static YAML)
 * Code: workflow.ts (TypeScript implementation)
 *
 * Like: Vercel (vercel.json + code)
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

// Main entry point
export async function main(variables: Record<string, any> = {}) {
  const { userName = 'World', message = '' } = variables;

  console.log('='.repeat(60));
  console.log('Phase 1: YAML + TypeScript');
  console.log('Variables:', { userName, message });
  console.log('='.repeat(60));

  const desktop = new Desktop();

  await openNotepad(desktop);
  await typeGreeting(desktop, { userName });

  if (message) {
    await addCustomMessage(desktop, { message });
  }

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
