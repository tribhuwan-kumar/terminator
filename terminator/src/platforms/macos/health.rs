//! macOS-specific health check implementation

use crate::health::{HealthCheckResult, PlatformHealthCheck};
use async_trait::async_trait;

/// macOS health checker
pub struct MacOSHealthChecker;

impl MacOSHealthChecker {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PlatformHealthCheck for MacOSHealthChecker {
    async fn check_health(&self) -> HealthCheckResult {
        // For now, return a healthy status for macOS
        // TODO: Implement actual Accessibility API checks
        let mut result = HealthCheckResult::healthy("macos");
        result.add_diagnostic("note", "Accessibility API health checks not yet implemented");
        result
    }
}