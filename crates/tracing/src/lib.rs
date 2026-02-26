use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tracing_subscriber::FmtSubscriber;

static TRACING_SUBSCRIBER: OnceLock<Arc<FmtSubscriber>> = OnceLock::new();

pub fn get_subscriber() -> Arc<FmtSubscriber> {
    TRACING_SUBSCRIBER.get_or_init(|| Arc::new(tracing_subscriber::fmt().with_target(false).finish())).clone()
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

pub fn info_with_fields_impl(message: &str, fields: &[(&str, &dyn std::fmt::Display)]) {
    let subscriber = get_subscriber();
    tracing::subscriber::with_default(subscriber, || {
        let mut field_pairs = vec![];
        for (key, value) in fields {
            field_pairs.push(format!("{}={}", key, value));
        }

        if field_pairs.is_empty() {
            tracing::info!("{}", message);
        } else {
            tracing::info!("{} {}", message, field_pairs.join(" "));
        }
    });
}

pub fn error_with_fields_impl<E: std::error::Error + ?Sized>(message: &str, error: &E, fields: &[(&str, &dyn std::fmt::Display)]) {
    let subscriber = get_subscriber();
    tracing::subscriber::with_default(subscriber, || {
        let mut field_pairs = vec![];
        for (key, value) in fields {
            field_pairs.push(format!("{}={}", key, value));
        }

        if field_pairs.is_empty() {
            tracing::error!("{}: {}", message, error);
        } else {
            tracing::error!("{}: {} {}", message, field_pairs.join(" "), error);
        }
    });
}

#[macro_export]
macro_rules! info_with_fields {
    ($message:expr, $($field:ident = $value:expr),* $(,)?) => {
        {
            let fields: &[(&str, &dyn std::fmt::Display)] = &[
                $((stringify!($field), &$value),)*
            ];
            $crate::info_with_fields_impl($message, fields);
        }
    };
}

#[macro_export]
macro_rules! error_with_fields {
    ($message:expr, $error:expr, $($field:ident = $value:expr),* $(,)?) => {
        {
            let fields: &[(&str, &dyn std::fmt::Display)] = &[
                $((stringify!($field), &$value),)*
            ];
            $crate::error_with_fields_impl($message, $error, fields);
        }
    };
}

pub fn error<E: std::error::Error + ?Sized>(message: &str, error: &E) {
    let subscriber = get_subscriber();
    tracing::subscriber::with_default(subscriber, || {
        tracing::error!("{}: {}", message, error);
    });
}
