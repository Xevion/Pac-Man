use tracing_subscriber::fmt::format::JsonFields;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::config::Config;
use crate::formatter;

/// Configure and initialize logging for the application
pub fn setup_logging(_config: &Config) {
    // Allow RUST_LOG to override levels; default to info for our crate and warn elsewhere
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn,pacman_server=info,pacman_server::auth=info"));

    // Default to pretty for local dev; switchable later if we add CLI
    let use_pretty = cfg!(debug_assertions);

    let subscriber: Box<dyn tracing::Subscriber + Send + Sync> = if use_pretty {
        Box::new(
            FmtSubscriber::builder()
                .with_target(true)
                .event_format(formatter::CustomPrettyFormatter)
                .with_env_filter(filter)
                .finish(),
        )
    } else {
        Box::new(
            FmtSubscriber::builder()
                .with_target(true)
                .event_format(formatter::CustomJsonFormatter)
                .fmt_fields(JsonFields::new())
                .with_env_filter(filter)
                .finish(),
        )
    };

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
