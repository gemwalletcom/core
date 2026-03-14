use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tracing_subscriber::FmtSubscriber;

static TRACING_SUBSCRIBER: OnceLock<Arc<FmtSubscriber>> = OnceLock::new();

pub fn get_subscriber() -> Arc<FmtSubscriber> {
    TRACING_SUBSCRIBER.get_or_init(|| Arc::new(tracing_subscriber::fmt().with_target(false).finish())).clone()
}

struct GemLogger;

impl log::Log for GemLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let subscriber = get_subscriber();
        let message = record.args().to_string();
        tracing::subscriber::with_default(subscriber, || match record.level() {
            log::Level::Error => tracing::error!("{}", message),
            log::Level::Warn => tracing::warn!("{}", message),
            log::Level::Info => tracing::info!("{}", message),
            log::Level::Debug => tracing::debug!("{}", message),
            log::Level::Trace => tracing::trace!("{}", message),
        });
    }

    fn flush(&self) {}
}

/// Initializes gem_tracing as the global `log` backend.
/// Log level is read from the `ROCKET_LOG_LEVEL` environment variable
/// (values: `off`, `critical`, `normal`, `debug`; default: `normal`).
/// If a global logger is already registered the call is a no-op.
pub fn init() {
    let level_filter = match std::env::var("ROCKET_LOG_LEVEL").as_deref() {
        Ok("off") => log::LevelFilter::Off,
        Ok("critical") => log::LevelFilter::Warn,
        Ok("debug") => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,
    };
    let _ = log::set_boxed_logger(Box::new(GemLogger));
    log::set_max_level(level_filter);
}

pub fn human_duration(duration: Duration) -> String {
    if duration.is_zero() {
        return "0s".to_string();
    }

    let mut parts = Vec::new();
    let mut remaining = duration.as_secs();
    const UNITS: [(&str, u64); 4] = [("d", 86_400), ("h", 3_600), ("m", 60), ("s", 1)];

    for (label, unit) in UNITS {
        if remaining >= unit {
            let value = remaining / unit;
            remaining %= unit;
            parts.push(format!("{value}{label}"));
            if parts.len() == 2 {
                break;
            }
        }
    }

    if parts.is_empty() { format!("{}ms", duration.subsec_millis()) } else { parts.join(" ") }
}

pub struct DurationMs(pub Duration);

impl std::fmt::Display for DurationMs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&human_duration(self.0))
    }
}

fn format_fields(fields: &[(&str, &dyn std::fmt::Display)]) -> String {
    fields.iter().map(|(key, value)| format!("{key}={value}")).collect::<Vec<_>>().join(" ")
}

pub fn info_with_fields_impl(message: &str, fields: &[(&str, &dyn std::fmt::Display)]) {
    let subscriber = get_subscriber();
    tracing::subscriber::with_default(subscriber, || {
        let pairs = format_fields(fields);
        if pairs.is_empty() {
            tracing::info!("{}", message);
        } else {
            tracing::info!("{} {}", message, pairs);
        }
    });
}

pub fn error_fields_impl(message: &str, fields: &[(&str, &dyn std::fmt::Display)]) {
    let subscriber = get_subscriber();
    tracing::subscriber::with_default(subscriber, || {
        let pairs = format_fields(fields);
        if pairs.is_empty() {
            tracing::error!("{}", message);
        } else {
            tracing::error!("{} {}", message, pairs);
        }
    });
}

pub fn error_with_fields_impl<E: std::error::Error + ?Sized>(message: &str, error: &E, fields: &[(&str, &dyn std::fmt::Display)]) {
    let subscriber = get_subscriber();
    tracing::subscriber::with_default(subscriber, || {
        let pairs = format_fields(fields);
        if pairs.is_empty() {
            tracing::error!("{} error={}", message, error);
        } else {
            tracing::error!("{} {} error={}", message, pairs, error);
        }
    });
}

#[macro_export]
macro_rules! info_with_fields {
    ($message:expr $(, $field:ident = $value:expr)* $(,)?) => {
        {
            let fields: &[(&str, &dyn std::fmt::Display)] = &[
                $((stringify!($field), &$value),)*
            ];
            $crate::info_with_fields_impl($message, fields);
        }
    };
}

#[macro_export]
macro_rules! error_fields {
    ($message:expr $(, $field:ident = $value:expr)* $(,)?) => {
        {
            let fields: &[(&str, &dyn std::fmt::Display)] = &[
                $((stringify!($field), &$value),)*
            ];
            $crate::error_fields_impl($message, fields);
        }
    };
}

#[macro_export]
macro_rules! error_with_fields {
    ($message:expr, $error:expr $(, $field:ident = $value:expr)* $(,)?) => {
        {
            let fields: &[(&str, &dyn std::fmt::Display)] = &[
                $((stringify!($field), &$value),)*
            ];
            $crate::error_with_fields_impl($message, $error, fields);
        }
    };
}

pub fn error<E: std::error::Error + ?Sized>(message: &str, error: &E) {
    error_with_fields_impl(message, error, &[]);
}
