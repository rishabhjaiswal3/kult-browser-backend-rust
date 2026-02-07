// src/config/log_config.rs
// Logging configuration

use std::env;

/// Log output format
#[derive(Debug, Clone, PartialEq)]
pub enum LogFormat {
    /// Human-readable colored output (development)
    Pretty,
    /// JSON structured output (production)
    Json,
}

impl LogFormat {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => LogFormat::Json,
            _ => LogFormat::Pretty,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log level filter (default: info)
    /// Options: trace, debug, info, warn, error
    pub level: String,
    /// Output format (default: pretty)
    pub format: LogFormat,
    /// Whether to include file/line info in logs (default: false in prod)
    pub include_file_info: bool,
    /// Whether to include target (module path) in logs (default: true)
    pub include_target: bool,
}

impl LogConfig {
    /// Load logging config from environment variables.
    pub fn from_env() -> Self {
        Self {
            level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),

            format: LogFormat::from_str(
                &env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string()),
            ),

            include_file_info: env::var("LOG_FILE_INFO")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),

            include_target: env::var("LOG_TARGET")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}
