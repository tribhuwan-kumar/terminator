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

    let cases = vec![
        ("https://www.mediar.ai/", Browser::Default),
        ("https://www.reddit.com/", Browser::Chrome),
        ("https://stackoverflow.com/", Browser::Firefox),
        ("https://github.com/mediar-ai/terminator/issues/180", Browser::Edge),
        ("https://practicetestautomation.com/practice-test-login/", Browser::Brave),
        ("https://docs.screenpi.pe/terminator/introduction", Browser::Opera),
        ("https://thisisatest.com/", Browser::Default), // invalid url
    ];

    for (url, browser) in cases {
        let result = engine.open_url(url, Some(browser.clone()));
        tokio::time::sleep(Duration::from_secs(5)).await;

        if url == "https://thisisatest.com/" {
            assert!(result.is_err(), "expected failure for invalid url");
        } else {
            assert!(result?.name().is_some(), "expected name for ui element");
        }
    }
    Ok(())
}
