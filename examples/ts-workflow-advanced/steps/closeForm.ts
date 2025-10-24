import { Desktop } from '@mediar/terminator';

/**
 * Step 9: Close Customer Form
 */
export async function closeForm(desktop: Desktop) {
  console.log('🔚 Step 9: Closing customer form...');

  // Click Close or Cancel button
  await desktop.locator('role:button|name:Close').click();

  // Wait for main customer list to be visible again
  await desktop.locator('role:button|name:New Customer').waitFor('visible', 3000);

  console.log('✅ Customer form closed');
}
