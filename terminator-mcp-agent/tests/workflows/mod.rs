pub mod banking_workflows;
pub mod ecommerce_workflows;
pub mod government_workflows;
pub mod social_media_workflows;

// Re-export all workflow creation functions
pub use banking_workflows::*;
pub use ecommerce_workflows::*;
pub use government_workflows::*;
pub use social_media_workflows::*;
