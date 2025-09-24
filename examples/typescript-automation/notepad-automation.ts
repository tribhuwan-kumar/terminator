// TypeScript Notepad Automation
// Opens Notepad and types demonstration text

declare var desktop: any;
declare var env: Record<string, any>;
declare var variables: Record<string, any>;
declare function sleep(ms: number): Promise<void>;

interface AutomationResult {
    status: string;
    message: string;
    [key: string]: any;
}

async function runNotepadAutomation(): Promise<string> {
    console.log('📝 Starting Notepad Automation (TypeScript)');

    try {
        // Step 1: Open Notepad
        console.log('Opening Notepad...');
        await desktop.openApplication('notepad.exe');
        await sleep(2000);
        console.log('✅ Notepad opened');

        // Step 2: Type demonstration text
        console.log('Typing demonstration text...');

        const lines = [
            'TYPESCRIPT AUTOMATION DEMO',
            '=' .repeat(40),
            `Date: ${new Date().toLocaleString()}`,
            '',
            'This text is being typed by TypeScript!',
            '',
            'Features demonstrated:',
            '- TypeScript execution in workflows',
            '- Desktop application control',
            '- Keyboard input simulation',
            '',
            'Each line appears automatically...'
        ];

        // Type each line with a delay
        for (const line of lines) {
            await desktop.pressKey(line);
            await desktop.pressKey('{Enter}');
            await sleep(200); // Pause between lines
        }

        console.log('✅ Text typed successfully');

        // Step 3: Save the file
        console.log('Saving file...');
        await desktop.pressKey('{Ctrl}s');
        await sleep(1000);

        const filename = `typescript-demo-${Date.now()}.txt`;
        await desktop.pressKey(filename);
        await sleep(500);

        await desktop.pressKey('{Enter}');
        await sleep(1000);

        console.log(`✅ File saved as: ${filename}`);

        const result: AutomationResult = {
            status: 'success',
            message: 'TypeScript automation completed successfully',
            filename: filename,
            lines_typed: lines.length
        };

        return JSON.stringify(result);

    } catch (error: any) {
        console.error('❌ Error in automation:', error);
        return JSON.stringify({
            status: 'error',
            message: error.message || String(error)
        });
    }
}

// Run the automation
runNotepadAutomation().then(result => {
    console.log('Final result:', result);
});