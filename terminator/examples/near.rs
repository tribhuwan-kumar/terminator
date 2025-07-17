use std::time::Duration;

use terminator::{platforms, AutomationError};
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), AutomationError> {
    use std::time::Instant;

    tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .init();

    let engine = platforms::create_engine(true, true)?;

    let opened_app = engine.get_focused_element()?;

    // Note: The specific output will vary depending on the active application.
    println!("Looking for the last combobox...");

    // // Measure time to find the element
    // let start = Instant::now();
    // let element = opened_app
    //     .locator("role:text|View Details")?
    //     .wait(Some(Duration::from_millis(100000)))
    //     .await?;
    // let elapsed = start.elapsed();
    // println!(
    //     "Time to find element: {:.2?} ms",
    //     elapsed.as_secs_f64() * 1000.0
    // );

    // println!("Found last combobox: {:?}", element.attributes());

    // get all checkboxes and toggle them
    let checkboxes = opened_app
        .locator("role:dialog >> role:checkbox")?
        .all(Some(Duration::from_millis(1000)), None)
        .await?;
    for checkbox in checkboxes {
        // checkbox.set_toggled(false)?;
        if checkbox.is_toggled()? {
            checkbox.click()?;
        }
    }

    // You can now interact with it, for example, click to open it.
    // element.click()?;

    Ok(())
}
