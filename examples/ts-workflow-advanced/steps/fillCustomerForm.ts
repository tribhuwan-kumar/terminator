import { Desktop } from '@mediar/terminator';

/**
 * Step 4: Fill Customer Form
 */
export async function fillCustomerForm(
  desktop: Desktop,
  variables: {
    customerName: string;
    email: string;
    phone?: string;
  }
) {
  console.log('📝 Step 4: Filling customer form...');

  // Fill customer name
  await desktop.locator('role:textbox|name:Customer Name').fill(variables.customerName);

  // Fill email
  await desktop.locator('role:textbox|name:Email').fill(variables.email);

  // Fill phone if provided
  if (variables.phone) {
    await desktop.locator('role:textbox|name:Phone').fill(variables.phone);
  }

  // Select customer status
  await desktop.locator('role:combobox|name:Status').selectOption('Active');

  console.log('✅ Customer form filled');
}
