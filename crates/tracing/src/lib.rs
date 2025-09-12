use sentry::ClientInitGuard;
use std::sync::{Arc, OnceLock};
use tracing_subscriber::FmtSubscriber;

static TRACING_SUBSCRIBER: OnceLock<Arc<FmtSubscriber>> = OnceLock::new();

fn get_subscriber() -> Arc<FmtSubscriber> {
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

pub fn error_with_context<E: std::error::Error + ?Sized>(message: &str, error: &E, context: &[(&str, &str)]) {
    let subscriber = get_subscriber();
    tracing::subscriber::with_default(subscriber, || {
        let mut fields = vec![];
        for (key, value) in context {
            fields.push(format!("{}={}", key, value));
        }
        if fields.is_empty() {
            tracing::error!("{}: {}", message, error);
        } else {
            tracing::error!("{}: {} {}", message, error, fields.join(" "));
        }
    });

    sentry::configure_scope(|scope| {
        for (key, value) in context {
            scope.set_tag(key, value);
        }
    });
    sentry::capture_message(message, sentry::Level::Error);
}

pub fn info_with_context(message: &str, context: &[(&str, &str)]) {
    let subscriber = get_subscriber();
    tracing::subscriber::with_default(subscriber, || {
        let mut fields = vec![];
        for (key, value) in context {
            fields.push(format!("{}={}", key, value));
        }
        if fields.is_empty() {
            tracing::info!("{}", message);
        } else {
            tracing::info!("{} {}", message, fields.join(" "));
        }
    });
}

pub fn error<E: std::error::Error + ?Sized>(message: &str, error: &E) {
    let subscriber = get_subscriber();
    tracing::subscriber::with_default(subscriber, || {
        tracing::error!("{}: {}", message, error);
    });
    sentry::capture_error(error);
}
