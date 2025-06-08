pub mod desktop;
pub mod element;
pub mod locator;
pub mod types;
pub mod exceptions;

// Main types first
pub use desktop::Desktop;
pub use element::{Element};
pub use locator::Locator;
pub use types::{
    Bounds,
    Coordinates,
    ClickResult,
    CommandOutput,
    ScreenshotResult,
    UIElementAttributes,
    UINode,
    TreeBuildConfig,
    PropertyLoadingMode,
    ExploreResponse,
    ExploredElementDetail,
};

// Error handling - see exceptions.rs for detailed architecture
pub use exceptions::map_error;
