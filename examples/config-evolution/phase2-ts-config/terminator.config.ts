#!/usr/bin/env tsx
/**
 * Phase 2: TypeScript Config
 *
 * Config: terminator.config.ts (TypeScript with logic)
 * Compiled: terminator.config.json (for UI parsing)
 * Code: workflow.ts (implementation)
 *
 * Like: Next.js (next.config.ts)
 */

import { defineConfig } from '@mediar/terminator';
import pkg from './package.json';

export default defineConfig({
  id: 'notepad-demo',
  name: 'Notepad Demo Workflow',
  description: 'Demonstrating Phase 2 TypeScript config',

  // Can reference package.json!
  version: pkg.version,

  // Can use computed values!
  tags: [
    'demo',
    'phase2',
    'typescript',
    process.env.NODE_ENV || 'development'
  ],

  variables: {
    userName: {
      type: 'string',
      label: 'User Name',
      description: 'Name to greet',
      // Can use environment variables!
      default: process.env.DEFAULT_USER || 'World',
      required: false,
    },

    message: {
      type: 'string',
      label: 'Custom Message',
      description: 'Optional custom message',
      required: false,
    },

    // Can compute defaults!
    environment: {
      type: 'string',
      label: 'Environment',
      default: process.env.NODE_ENV || 'development',
      required: false,
    },
  },

  steps: [
    {
      id: 'open-notepad',
      name: 'Open Notepad',
      function: 'openNotepad',
      description: 'Launches Notepad application',
    },
    {
      id: 'type-greeting',
      name: 'Type Greeting',
      function: 'typeGreeting',
      description: 'Types personalized greeting',
      inputs: ['userName'],
    },
    {
      id: 'add-message',
      name: 'Add Custom Message',
      function: 'addCustomMessage',
      description: 'Adds optional custom message',
      // Can use logic in conditions!
      condition: (vars) => Boolean(vars.message),
      inputs: ['message'],
    },
    {
      id: 'add-environment',
      name: 'Add Environment Info',
      function: 'addEnvironment',
      description: 'Adds environment information',
      inputs: ['environment'],
    },
  ],

  // Can add computed metadata!
  metadata: {
    buildTime: new Date().toISOString(),
    nodeVersion: process.version,
    platform: process.platform,
  },
});
