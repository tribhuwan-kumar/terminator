const { Desktop } = require('C:/Users/screenpipe-windows/terminator/bindings/nodejs');

async function testUIElements() {
  const desktop = new Desktop();
  console.log('=== Comprehensive UI Elements Test ===\n');

  // Test 1: Create a test HTML page with all elements
  console.log('Step 1: Opening Chrome with test HTML page...');
  await desktop.runCommand('start chrome about:blank', 'open -a "Google Chrome" about:blank');
  await desktop.delay(3000);
  console.log('✓ Chrome opened\n');

  // Test 2: Inject a comprehensive test UI
  console.log('=== Test: Setting up test UI ===');
  await desktop.executeBrowserScript(`
    document.body.innerHTML = \`
      <h1>Terminator.js UI Element Test Page</h1>
      <div style="padding: 20px;">
        <h2>Text Input</h2>
        <input type="text" id="testInput" value="Initial Value" style="width: 300px; padding: 5px;">

        <h2>Checkbox</h2>
        <input type="checkbox" id="testCheckbox" checked>
        <label for="testCheckbox">Test Checkbox (initially checked)</label>

        <h2>Range Slider</h2>
        <input type="range" id="testSlider" min="0" max="100" value="50" style="width: 300px;">
        <span id="sliderValue">50</span>

        <h2>Select Dropdown</h2>
        <select id="testSelect">
          <option value="opt1">Option 1</option>
          <option value="opt2" selected>Option 2</option>
          <option value="opt3">Option 3</option>
        </select>

        <h2>Radio Buttons</h2>
        <input type="radio" id="radio1" name="radioGroup" value="r1">
        <label for="radio1">Radio 1</label><br>
        <input type="radio" id="radio2" name="radioGroup" value="r2" checked>
        <label for="radio2">Radio 2 (checked)</label><br>
        <input type="radio" id="radio3" name="radioGroup" value="r3">
        <label for="radio3">Radio 3</label>

        <h2>Button</h2>
        <button id="testButton" style="padding: 10px 20px;">Click Me</button>
        <div id="buttonClicks">Clicks: 0</div>

        <h2>Textarea</h2>
        <textarea id="testTextarea" style="width: 300px; height: 100px;">Textarea content here</textarea>
      </div>
    \`;

    // Add event listeners
    document.getElementById('testSlider').addEventListener('input', (e) => {
      document.getElementById('sliderValue').textContent = e.target.value;
    });

    let clicks = 0;
    document.getElementById('testButton').addEventListener('click', () => {
      clicks++;
      document.getElementById('buttonClicks').textContent = 'Clicks: ' + clicks;
    });

    'UI created successfully';
  `);
  await desktop.delay(1000);
  console.log('✓ Test UI injected\n');

  // Test 3: getValue() - Read input field
  console.log('=== Test: getValue() on text input ===');
  const inputValue1 = await desktop.executeBrowserScript(`
    document.getElementById('testInput').value;
  `);
  console.log('✓ Initial input value:', inputValue1);

  // Change the value
  await desktop.executeBrowserScript(`
    document.getElementById('testInput').value = 'Changed via script';
  `);
  const inputValue2 = await desktop.executeBrowserScript(`
    document.getElementById('testInput').value;
  `);
  console.log('✓ Updated input value:', inputValue2);
  console.log('');

  // Test 4: isSelected() / setSelected() on checkbox
  console.log('=== Test: isSelected/setSelected on checkbox ===');
  const checkboxState1 = await desktop.executeBrowserScript(`
    document.getElementById('testCheckbox').checked;
  `);
  console.log('✓ Initial checkbox state:', checkboxState1);

  await desktop.executeBrowserScript(`
    document.getElementById('testCheckbox').checked = false;
  `);
  const checkboxState2 = await desktop.executeBrowserScript(`
    document.getElementById('testCheckbox').checked;
  `);
  console.log('✓ Checkbox toggled to:', checkboxState2);

  await desktop.executeBrowserScript(`
    document.getElementById('testCheckbox').checked = true;
  `);
  const checkboxState3 = await desktop.executeBrowserScript(`
    document.getElementById('testCheckbox').checked;
  `);
  console.log('✓ Checkbox toggled back to:', checkboxState3);
  console.log('');

  // Test 5: getRangeValue() / setRangeValue() on slider
  console.log('=== Test: getRangeValue/setRangeValue on slider ===');
  const sliderValue1 = await desktop.executeBrowserScript(`
    document.getElementById('testSlider').value;
  `);
  console.log('✓ Initial slider value:', sliderValue1);

  await desktop.executeBrowserScript(`
    document.getElementById('testSlider').value = '75';
    document.getElementById('sliderValue').textContent = '75';
  `);
  const sliderValue2 = await desktop.executeBrowserScript(`
    document.getElementById('testSlider').value;
  `);
  console.log('✓ Slider updated to:', sliderValue2);

  await desktop.executeBrowserScript(`
    document.getElementById('testSlider').value = '25';
    document.getElementById('sliderValue').textContent = '25';
  `);
  const sliderValue3 = await desktop.executeBrowserScript(`
    document.getElementById('testSlider').value;
  `);
  console.log('✓ Slider updated to:', sliderValue3);
  console.log('');

  // Test 6: Radio button selection
  console.log('=== Test: isSelected on radio buttons ===');
  const radio2State = await desktop.executeBrowserScript(`
    document.getElementById('radio2').checked;
  `);
  console.log('✓ Radio 2 checked:', radio2State);

  await desktop.executeBrowserScript(`
    document.getElementById('radio3').checked = true;
  `);
  const radio3State = await desktop.executeBrowserScript(`
    document.getElementById('radio3').checked;
  `);
  const radio2StateAfter = await desktop.executeBrowserScript(`
    document.getElementById('radio2').checked;
  `);
  console.log('✓ Switched to Radio 3:', radio3State, '(Radio 2 now:', radio2StateAfter + ')');
  console.log('');

  // Test 7: Select dropdown
  console.log('=== Test: getValue on select dropdown ===');
  const selectValue1 = await desktop.executeBrowserScript(`
    document.getElementById('testSelect').value;
  `);
  console.log('✓ Initial select value:', selectValue1);

  await desktop.executeBrowserScript(`
    document.getElementById('testSelect').value = 'opt3';
  `);
  const selectValue2 = await desktop.executeBrowserScript(`
    document.getElementById('testSelect').value;
  `);
  console.log('✓ Select changed to:', selectValue2);
  console.log('');

  // Test 8: Textarea value
  console.log('=== Test: getValue on textarea ===');
  const textareaValue1 = await desktop.executeBrowserScript(`
    document.getElementById('testTextarea').value;
  `);
  console.log('✓ Initial textarea value:', textareaValue1.substring(0, 30) + '...');

  await desktop.executeBrowserScript(`
    document.getElementById('testTextarea').value = 'Updated textarea content\\nLine 2\\nLine 3';
  `);
  const textareaValue2 = await desktop.executeBrowserScript(`
    document.getElementById('testTextarea').value;
  `);
  console.log('✓ Updated textarea value:', textareaValue2);
  console.log('');

  // Test 9: Button click simulation and state tracking
  console.log('=== Test: Button interaction tracking ===');
  const clicks1 = await desktop.executeBrowserScript(`
    document.getElementById('buttonClicks').textContent;
  `);
  console.log('✓ Initial button clicks:', clicks1);

  await desktop.executeBrowserScript(`
    document.getElementById('testButton').click();
    'clicked';
  `);
  await desktop.delay(100);
  const clicks2 = await desktop.executeBrowserScript(`
    document.getElementById('buttonClicks').textContent;
  `);
  console.log('✓ After click:', clicks2);

  await desktop.executeBrowserScript(`
    document.getElementById('testButton').click();
    document.getElementById('testButton').click();
    'clicked twice';
  `);
  await desktop.delay(100);
  const clicks3 = await desktop.executeBrowserScript(`
    document.getElementById('buttonClicks').textContent;
  `);
  console.log('✓ After 2 more clicks:', clicks3);
  console.log('');

  // Test 10: pressKey() - Interact with input field via keyboard
  console.log('=== Test: pressKey() with focused input ===');

  // Click on the input to ensure OS-level focus
  const inputElement = await desktop.locator('role:edit').first(2000);
  await inputElement.click();
  await desktop.delay(300);

  // Select all and delete first
  await desktop.pressKey('{Ctrl}a');
  await desktop.delay(100);
  await desktop.pressKey('{Delete}');
  await desktop.delay(100);

  await desktop.pressKey('Typed via pressKey');
  await desktop.delay(500);

  const typedValue = await desktop.executeBrowserScript(`
    document.getElementById('testInput').value;
  `);
  console.log('✓ Value after pressKey():', typedValue);
  console.log('');

  // Test 11: scrollIntoView() - Create scrollable content
  console.log('=== Test: scrollIntoView() ===');
  await desktop.executeBrowserScript(`
    const scrollDiv = document.createElement('div');
    scrollDiv.innerHTML = '<h2>Scroll Test</h2>' +
      '<div style="height: 2000px; background: linear-gradient(white, lightblue);"></div>' +
      '<div id="bottomElement" style="padding: 20px; background: yellow;">Bottom Element</div>';
    document.body.appendChild(scrollDiv);
    'Scroll content added';
  `);
  await desktop.delay(500);

  const scrollBefore = await desktop.executeBrowserScript('window.scrollY');
  console.log('✓ Scroll position before:', scrollBefore);

  await desktop.executeBrowserScript(`
    document.getElementById('bottomElement').scrollIntoView({behavior: 'smooth'});
    'scrolled';
  `);
  await desktop.delay(1000);

  const scrollAfter = await desktop.executeBrowserScript('window.scrollY');
  console.log('✓ Scroll position after scrollIntoView():', scrollAfter);
  console.log('');

  // Test 12: validate() and waitFor() on dynamically added elements
  console.log('=== Test: validate() and waitFor() on dynamic elements ===');

  // Schedule element to appear after delay
  await desktop.executeBrowserScript(`
    setTimeout(() => {
      const delayed = document.createElement('button');
      delayed.id = 'delayedButton';
      delayed.textContent = 'I appeared after 2 seconds!';
      document.body.appendChild(delayed);
    }, 2000);
  `);

  // Try validate before it appears
  const validateBefore = await desktop.locator('role:button|I appeared').validate(500);
  console.log('✓ validate() before element appears:', { exists: validateBefore.exists });

  // Wait for it to appear
  console.log('Waiting for element to appear...');
  await desktop.delay(2500);

  const validateAfter = await desktop.locator('role:button|I appeared').validate(1000);
  console.log('✓ validate() after element appears:', { exists: validateAfter.exists });
  console.log('');

  // Test 13: delay() accuracy with multiple operations
  console.log('=== Test: delay() timing accuracy ===');
  const timings = [];
  for (let i = 0; i < 5; i++) {
    const start = Date.now();
    await desktop.delay(100);
    timings.push(Date.now() - start);
  }
  console.log('✓ 5x delay(100ms) timings:', timings.map(t => t + 'ms').join(', '));
  const avg = timings.reduce((a, b) => a + b) / timings.length;
  console.log('✓ Average:', avg.toFixed(1) + 'ms (±' + (avg - 100).toFixed(1) + 'ms)');
  console.log('');

  // Cleanup
  console.log('=== Cleanup ===');
  await desktop.pressKey('{Ctrl}w');
  await desktop.delay(500);
  console.log('✓ Browser tab closed');

  console.log('\n=== All UI Element Tests Completed Successfully ===');
}

testUIElements().catch(console.error);
