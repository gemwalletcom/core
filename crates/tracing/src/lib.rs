use sentry::ClientInitGuard;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tracing_subscriber::FmtSubscriber;

static TRACING_SUBSCRIBER: OnceLock<Arc<FmtSubscriber>> = OnceLock::new();

pub fn get_subscriber() -> Arc<FmtSubscriber> {
    TRACING_SUBSCRIBER
        .get_or_init(|| Arc::new(tracing_subscriber::fmt().with_target(false).finish()))
        .clone()
}

pub struct SentryConfig {
    pub dsn: String,
    pub sample_rate: f32,
}

pub struct SentryTracing {
    _guard: Option<ClientInitGuard>,
}

impl SentryTracing {
    pub fn init(config: Option<&SentryConfig>, service: &str) -> Self {
        let _guard = config.and_then(|sentry| {
            if sentry.dsn.is_empty() {
                return None;
            }

            let options = sentry::ClientOptions {
                dsn: Some(sentry.dsn.parse().ok()?),
                send_default_pii: false,
                sample_rate: sentry.sample_rate,
                attach_stacktrace: true,
                ..Default::default()
            };

            let guard = sentry::init(options);

            sentry::configure_scope(|scope| {
                scope.set_tag("service", service);
            });

            Some(guard)
        });

        Self { _guard }
    }
}

pub use sentry::{capture_message, configure_scope, start_transaction, Level, TransactionContext};

pub struct DurationMs(pub Duration);

impl std::fmt::Display for DurationMs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}ms", self.0.as_millis())
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
            tracing::error!("{}: {} {}", message, error, field_pairs.join(" "));
        }
    });

    sentry::configure_scope(|scope| {
        for (key, value) in fields {
            scope.set_tag(key, value.to_string());
        }
    });
    sentry::capture_message(message, sentry::Level::Error);
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
    sentry::capture_error(error);
}
