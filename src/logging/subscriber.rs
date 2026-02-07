// src/logging/subscriber.rs
// Tracing subscriber configuration

use crate::config::log_config::{LogConfig, LogFormat};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter,
};

/// Initialize the tracing subscriber based on config.
pub fn init_subscriber(config: &LogConfig) {
    // Build the filter from LOG_LEVEL
    let filter = EnvFilter::try_new(&config.level).unwrap_or_else(|_| EnvFilter::new("info"));

    match config.format {
        LogFormat::Pretty => {
            // Pretty format for development
            let fmt_layer = fmt::layer()
                .with_target(config.include_target)
                .with_file(config.include_file_info)
                .with_line_number(config.include_file_info)
                .with_thread_ids(false)
                .with_span_events(FmtSpan::NONE)
                .pretty();

            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer)
                .init();
        }
        LogFormat::Json => {
            // JSON format for production (log aggregators)
            let fmt_layer = fmt::layer()
                .with_target(config.include_target)
                .with_file(config.include_file_info)
                .with_line_number(config.include_file_info)
                .with_thread_ids(false)
                .with_span_events(FmtSpan::NONE)
                .json();

            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer)
                .init();
        }
    }
}
