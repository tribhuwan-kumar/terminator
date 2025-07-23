use terminator::Browser;
use terminator::{platforms, AutomationError};
use tracing::{info, Level};

#[tokio::test]
#[ignore] // TODO does not work in ci
async fn test_open_url() -> Result<(), AutomationError> {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .init();

    let engine = platforms::create_engine(false, false)?;

    let cases = vec![
        ("https://www.mediar.ai/", Browser::Default),
        ("https://www.reddit.com/", Browser::Chrome),
        ("https://stackoverflow.com/", Browser::Firefox),
        (
            "https://github.com/mediar-ai/terminator/issues/180",
            Browser::Edge,
        ),
        (
            "https://practicetestautomation.com/practice-test-login/",
            Browser::Brave,
        ),
        (
            "https://docs.screenpi.pe/terminator/introduction",
            Browser::Opera,
        ),
        ("https://thisisatest.com/", Browser::Default), // invalid url
    ];

    for (url, browser) in cases {
        let start = std::time::Instant::now();
        let result = engine.open_url(url, Some(browser.clone()));
        let elapsed = start.elapsed();

        if url == "https://thisisatest.com/" {
            assert!(result.is_err(), "expected failure for invalid url");
        } else {
            assert!(result?.name().is_some(), "expected name for ui element");
            info!(
                "opened url '{:?}' in '{:?}' in '{:?}'",
                url, browser, elapsed
            );
        }
    }
    Ok(())
}
