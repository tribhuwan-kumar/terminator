// TypeScript automation for application analysis
// This file is meant to be called from a YAML workflow using script_file

declare var desktop: any;
declare var env: Record<string, any>;
declare var variables: Record<string, any>;
declare function sleep(ms: number): Promise<void>;

interface ApplicationInfo {
    name: string;
    id: string;
    pid: number;
    focused?: boolean;
}

async function analyzeApplications() {
    console.log('🔍 Starting Application Analysis (TypeScript)');
    console.log('='.repeat(50));

    try {
        // Get all running applications
        const apps: ApplicationInfo[] = await desktop.applications();

        console.log(`📊 Found ${apps.length} running applications`);

        // Find focused app
        const focusedApp = apps.find((app: ApplicationInfo) => app.focused === true);
        if (focusedApp) {
            console.log(`✨ Currently focused: ${focusedApp.name}`);
        } else {
            console.log('✨ No application currently has focus');
        }

        // Group apps by base name
        const appGroups: Record<string, ApplicationInfo[]> = {};
        for (const app of apps) {
            if (app.name && typeof app.name === 'string') {
                const baseName = app.name.split(' - ')[0].split(' — ')[0].trim();
                if (!appGroups[baseName]) {
                    appGroups[baseName] = [];
                }
                appGroups[baseName].push(app);
            }
        }

        console.log('\n📋 Applications Summary:');
        for (const [name, instances] of Object.entries(appGroups)) {
            const pids = instances.map(i => i.pid).join(', ');
            console.log(`  - ${name}: ${instances.length} instance(s) [PIDs: ${pids}]`);
        }

        // Return result as JSON string (TypeScript needs this for now)
        const result = {
            status: 'success',
            total_apps: apps.length,
            focused_app: focusedApp ? focusedApp.name : null,
            app_groups: Object.keys(appGroups).length,
            timestamp: new Date().toISOString()
        };

        return JSON.stringify(result);

    } catch (error: any) {
        console.error('❌ Error analyzing applications:', error);
        return JSON.stringify({
            status: 'error',
            error: error.message || String(error)
        });
    }
}

// Execute the main function
analyzeApplications().then(result => {
    // The result needs to be available to the workflow
    console.log('Result:', result);
});