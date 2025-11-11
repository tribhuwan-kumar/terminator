const assert = require('assert');
const path = require('path');
const fs = require('fs');

const { Desktop } = require('../wrapper.js');

async function testFunctionInput() {
  let capturedScript = null;
  const expected = { greeting: 'hello', answer: 42 };
  const fake = {
    _originalExecuteBrowserScript: async (script) => {
      capturedScript = script;
      return JSON.stringify(expected);
    },
  };

  const result = await Desktop.prototype.executeBrowserScript.call(
    fake,
    ({ greeting, answer }) => ({ greeting, answer }),
    { greeting: 'hello', answer: 42 }
  );

  assert.deepStrictEqual(result, expected);
  assert.ok(
    capturedScript.includes('"greeting":"hello"') &&
      capturedScript.includes('"answer":42'),
    'env payload should be embedded in generated script'
  );
  assert.ok(
    capturedScript.trim().startsWith('(async function()'),
    'generated script should be wrapped in async IIFE'
  );
}

async function testStringInput() {
  const fake = {
    _originalExecuteBrowserScript: async () => 'raw-result',
  };
  const script = '(() => "ignored")()';
  const result = await Desktop.prototype.executeBrowserScript.call(fake, script);
  assert.strictEqual(result, 'raw-result');
}

async function testFileInput() {
  const fixturePath = path.join(__dirname, 'fixtures', 'sample-browser-script.js');
  const expectedScript = fs.readFileSync(fixturePath, 'utf8');
  let receivedScript = null;
  const fake = {
    _originalExecuteBrowserScript: async (script) => {
      receivedScript = script;
      return 'file-result';
    },
  };

  const result = await Desktop.prototype.executeBrowserScript.call(fake, {
    file: fixturePath,
    env: { unused: true },
  });

  assert.strictEqual(result, 'file-result');
  assert.strictEqual(receivedScript, expectedScript);
}

async function run() {
  try {
    await testFunctionInput();
    console.log('✅ executeBrowserScript handles function input');

    await testStringInput();
    console.log('✅ executeBrowserScript preserves string behavior');

    await testFileInput();
    console.log('✅ executeBrowserScript loads scripts from files');

    console.log('🎉 All executeBrowserScript wrapper tests passed');
  } catch (err) {
    console.error('❌ executeBrowserScript wrapper test failed:', err);
    process.exitCode = 1;
  }
}

run();


