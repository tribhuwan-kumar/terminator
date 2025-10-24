import { Desktop } from '@mediar/terminator';

/**
 * Step 6: Submit Customer Form
 */
export async function submitForm(desktop: Desktop) {
  console.log('💾 Step 6: Submitting customer form...');

  // Click Save button
  await desktop.locator('role:button|name:Save').click();

  // Wait for submission to complete
  await new Promise(r => setTimeout(r, 2000));

  console.log('✅ Form submitted');
}
