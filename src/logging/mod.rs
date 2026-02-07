// src/logging/mod.rs
// Centralized logging framework

mod subscriber;

use crate::config::CONFIG;

/// Initialize the logging/tracing framework.
/// Call this as early as possible in main().
///
/// This sets up the global tracing subscriber based on config:
/// - LOG_LEVEL: trace, debug, info, warn, error (default: info)
/// - LOG_FORMAT: pretty, json (default: pretty)
///
/// # Example
/// ```ignore
/// use kult_browser_backend_rust::logging;
///
/// fn main() {
///     logging::init();
///     tracing::info!("Application starting");
/// }
/// ```
pub fn init() {
    subscriber::init_subscriber(&CONFIG.log);
}

// Re-export tracing macros for convenience
// Usage: use kult_browser_backend_rust::logging::{info, warn, error, debug, trace};
pub use tracing::{debug, error, info, trace, warn};
