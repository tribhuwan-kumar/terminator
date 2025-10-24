import { Desktop } from '@mediar/terminator';

/**
 * Step 8: Send Welcome Email (Conditional)
 */
export async function sendWelcomeEmail(
  desktop: Desktop,
  variables: { email: string }
) {
  console.log('📧 Step 8: Sending welcome email...');

  // Click Email button
  await desktop.locator('role:button|name:Send Email').click();

  // Wait for email dialog
  await desktop.locator('role:Window|name:Email').waitFor('visible', 3000);

  // Select welcome email template
  await desktop.locator('role:combobox|name:Template').selectOption('Welcome Email');

  // Verify recipient
  const recipientField = desktop.locator('role:textbox|name:To');
  await recipientField.fill(variables.email);

  // Send email
  await desktop.locator('role:button|name:Send').click();

  // Wait for confirmation
  await desktop.locator('text:Email sent').waitFor('visible', 5000);

  console.log(`✅ Welcome email sent to ${variables.email}`);
}
