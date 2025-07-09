use terminator::{Browser, Desktop};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Zoom Control Demo");
    println!("=================\n");

    // Initialize desktop automation
    let desktop = Desktop::new()?;

    // Open a browser to demonstrate zoom
    println!("Opening browser...");
    let browser = desktop
        .open_url("https://example.com", Some(Browser::Chrome))
        .await?;

    // Wait for page to load
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("\nDemonstrating zoom controls:");

    // 1. Traditional zoom in/out
    println!("\n1. Traditional incremental zoom:");
    println!("   - Zooming in 3 levels...");
    desktop.zoom_in(3).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    println!("   - Zooming out 2 levels...");
    desktop.zoom_out(2).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // 2. Set zoom to specific levels
    println!("\n2. Setting zoom to specific percentages:");

    println!("   - Setting zoom to 150%...");
    desktop.set_zoom(150).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("   - Setting zoom to 75%...");
    desktop.set_zoom(75).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("   - Resetting to 100%...");
    desktop.set_zoom(100).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 3. Advanced zoom scenarios
    println!("\n3. Advanced zoom scenarios:");

    // Zoom for accessibility
    println!("   - Setting high zoom (200%) for accessibility...");
    desktop.set_zoom(200).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Zoom out for overview
    println!("   - Setting low zoom (50%) for overview...");
    desktop.set_zoom(50).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Return to normal
    println!("   - Returning to normal (100%)...");
    desktop.set_zoom(100).await?;

    println!("\nZoom demo completed!");
    println!("\nNote: The exact zoom behavior depends on the application.");
    println!("Most browsers use 10% increments, but some applications may vary.");

    Ok(())
}
