use std::time::Duration;

use terminator::{platforms, AutomationError, Selector};
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), AutomationError> {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .init();

    let engine = platforms::create_engine(true, true)?;

    let opened_app = engine.get_focused_element()?;

    println!("Looking for the dropdown to the right of 'Results per page'...");

    // Here, we use the `RightOf` selector.
    // 1. It first finds the anchor element: a text element containing "Results per page".
    // 2. Then, it searches for elements to the right of that anchor.
    // 3. The `.first()` method gets the one closest to the anchor.
    let element = opened_app
        .locator(Selector::RightOf(Box::new(Selector::Name(
            "Results".to_string(),
        ))))?
        .first(Some(Duration::from_millis(10000)))
        .await?;

    println!("Found element: {:?}", element.attributes());

    // You can now interact with it, for example, click to open it.
    // element.click()?;

    element.click()?;

    Ok(())
}
