import { Desktop } from '@mediar/terminator';

/**
 * Step 3: Click New Customer Button
 */
export async function clickNewCustomer(desktop: Desktop) {
  console.log('➕ Step 3: Clicking New Customer button...');

  await desktop.locator('role:button|name:New Customer').click();

  // Wait for the form to appear
  await desktop.locator('role:textbox|name:Customer Name').waitFor('visible', 3000);

  console.log('✅ Customer form opened');
}
