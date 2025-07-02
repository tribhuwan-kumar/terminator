// this test is not compelte
use crate::{AutomationError, Desktop};
use tracing::Level;

#[tokio::test]
async fn get_applications_test() -> Result<(), AutomationError> {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::INFO)
        .init();

    let desktop = Desktop::new(false, false)?;

    let applications = vec![
        "notepad",
        "firefox private",
        "edge",
        "paint",
    ];

    for app in applications {
        let opened_app = desktop.open_application(app)?;
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    let apps = desktop.applications()?;
    tracing::info!("total apps: {:?}", apps.len());

    for app in &apps {
        tracing::info!("App Name: {:?}", app.name_or_empty());
        let process_id = app.process_id().unwrap();
        let output = std::process::Command::new("powershell")
            .arg("-Command")
            .arg(format!(
                "Get-WmiObject Win32_Process | Where-Object {{ $_.ProcessId -eq {} }} | ForEach-Object {{ taskkill.exe /T /F /PID $_.ProcessId; Write-Output \"Process with PID $($_.ProcessId) has been terminated.\" }}",
                process_id
            ))
            .output()
            .unwrap();

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("Command Output:\n{}", stdout);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Command Error:\n{}", stderr);
        }
    }

    Ok(())
}
