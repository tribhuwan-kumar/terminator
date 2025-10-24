import { Desktop } from '@mediar/terminator';

/**
 * Step 1: Open CRM Application
 */
export async function openCRM(desktop: Desktop) {
  console.log('🚀 Step 1: Opening CRM application...');

  desktop.openApplication('crm.exe');

  // Wait for CRM window to appear
  await desktop.locator('role:Window|name:CRM').waitFor('visible', 10000);

  console.log('✅ CRM application opened');
}
