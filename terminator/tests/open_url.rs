use tracing::Level;
use std::time::Duration;
use terminator::Browser;
use terminator::{platforms, AutomationError};

#[tokio::test]
async fn test_open_url() -> Result<(), AutomationError> {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .init();

    let engine = platforms::create_engine(false, false)?;

    let url = "https://github.com/mediar-ai/terminator/issues/180";

    let browsers = vec![
        Browser::Default,
        Browser::Chrome,
        Browser::Firefox,
        Browser::Edge,
        Browser::Brave,
        Browser::Opera,
        Browser::Vivaldi,
    ];

    for app in browsers {
        let result = engine.open_url(url, Some(app))?;
        tokio::time::sleep(Duration::from_secs(5)).await;

        assert!(result.name().is_some() , "Expected failure when opening URL with an invalid custom browser");
    }
    Ok(())
}
