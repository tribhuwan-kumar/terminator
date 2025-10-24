import { Desktop } from '@mediar/terminator';

interface WorkflowContext {
  formValid?: boolean;
}

/**
 * Step 5: Validate Form Data
 */
export async function validateForm(
  desktop: Desktop,
  context: WorkflowContext
) {
  console.log('🔍 Step 5: Validating form data...');

  // Check if required fields are filled
  const nameField = desktop.locator('role:textbox|name:Customer Name');
  const emailField = desktop.locator('role:textbox|name:Email');

  const nameValue = await nameField.value();
  const emailValue = await emailField.value();

  if (!nameValue || !emailValue) {
    context.formValid = false;
    throw new Error('Required fields are empty');
  }

  // Check for validation error messages
  const errorValidation = await desktop
    .locator('role:text|name:Error')
    .validate(1000);

  if (errorValidation.exists) {
    context.formValid = false;
    throw new Error('Form has validation errors');
  }

  context.formValid = true;
  console.log('✅ Form validation passed');
}
