//! Linux-specific health check implementation

use crate::health::{HealthCheckResult, PlatformHealthCheck};
use async_trait::async_trait;

/// Linux health checker
pub struct LinuxHealthChecker;

impl LinuxHealthChecker {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PlatformHealthCheck for LinuxHealthChecker {
    async fn check_health(&self) -> HealthCheckResult {
        // For now, return a healthy status for Linux
        // TODO: Implement actual AT-SPI or X11 accessibility checks
        let mut result = HealthCheckResult::healthy("linux");
        result.add_diagnostic("note", "AT-SPI health checks not yet implemented");
        result
    }
}