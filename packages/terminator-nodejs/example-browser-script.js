/**
 * Example: Enhanced executeBrowserScript API
 *
 * Shows 3 ways to use executeBrowserScript:
 * 1. Plain string (backward compatible)
 * 2. Function (new - auto IIFE wrapping)
 * 3. TypeScript file path (new - auto compilation)
 */

const { Desktop } = require('./wrapper.js');

async function main() {
  const desktop = new Desktop();

  // Get Chrome window
  const chromeWindow = await desktop.locator('role:Window|name:Chrome').first(5000);

  console.log('\n=== Example 1: Plain String (backward compatible) ===');
  const result1 = await chromeWindow.executeBrowserScript(`
    (function() {
      return JSON.stringify({ title: document.title });
    })()
  `);
  console.log('Result:', JSON.parse(result1));

  console.log('\n=== Example 2: Function (new!) ===');
  const result2 = await chromeWindow.executeBrowserScript(() => {
    // No IIFE needed, no JSON.stringify needed!
    return {
      title: document.title,
      url: window.location.href,
      links: document.querySelectorAll('a').length
    };
  });
  console.log('Result:', result2); // Already parsed!

  console.log('\n=== Example 3: Function with env variables ===');
  const result3 = await chromeWindow.executeBrowserScript((env) => {
    const selector = env.selector || 'a';
    const elements = document.querySelectorAll(selector);

    return {
      selector: selector,
      count: elements.length,
      first: elements[0]?.textContent
    };
  }, { selector: '.link' });
  console.log('Result:', result3);

  console.log('\n=== Example 4: TypeScript file (new!) ===');
  // Create a .ts file first (see example-script.ts)
  const result4 = await chromeWindow.executeBrowserScript('./example-script.ts');
  console.log('Result:', JSON.parse(result4));

  console.log('\n=== Example 5: File with env ===');
  const result5 = await chromeWindow.executeBrowserScript({
    file: './example-script.ts',
    env: { maxResults: 10 }
  });
  console.log('Result:', JSON.parse(result5));
}

main().catch(console.error);
