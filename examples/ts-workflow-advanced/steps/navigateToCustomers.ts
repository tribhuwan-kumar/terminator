import { Desktop } from '@mediar/terminator';

/**
 * Step 2: Navigate to Customers Section
 */
export async function navigateToCustomers(desktop: Desktop) {
  console.log('📂 Step 2: Navigating to Customers section...');

  // Click Customers menu item
  await desktop.locator('role:button|name:Customers').click();

  // Wait for customer list to load
  await desktop.locator('role:button|name:New Customer').waitFor('visible', 5000);

  console.log('✅ Navigated to Customers section');
}
