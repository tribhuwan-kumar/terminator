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

    // Note: The specific output will vary depending on the active application.
    println!("Looking for the last combobox...");

    // Here, we use the `nth` selector in a chain.
    // 1. It first finds all elements with the "combobox" role.
    // 2. Then, the `nth=-1` selector picks the last element from that list.
    // 3. The `.first()` method waits for that specific element to appear.
    let element = opened_app
        .locator("role:dialog >> role:edit >> nth=1")?
        .first(Some(Duration::from_millis(10000)))
        .await?;

    println!("Found last combobox: {:?}", element.attributes());

    // You can now interact with it, for example, click to open it.
    // element.click()?;
    element.set_value("hi")?;

    Ok(())
}
