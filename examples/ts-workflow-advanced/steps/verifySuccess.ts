import { Desktop } from '@mediar/terminator';

interface WorkflowContext {
  customerId?: string;
  submissionTime?: Date;
}

/**
 * Step 7: Verify Successful Submission
 */
export async function verifySuccess(
  desktop: Desktop,
  context: WorkflowContext
) {
  console.log('🔍 Step 7: Verifying successful submission...');

  // Wait for success message
  await desktop
    .locator('text:Customer created successfully')
    .waitFor('visible', 5000);

  // Try to extract customer ID from success message
  const successMessage = await desktop
    .locator('role:text|name:Customer ID')
    .text();

  if (successMessage) {
    // Extract ID (assuming format like "Customer ID: 12345")
    const match = successMessage.match(/\d+/);
    if (match) {
      context.customerId = match[0];
    }
  }

  context.submissionTime = new Date();

  console.log('✅ Submission verified successfully');
}
