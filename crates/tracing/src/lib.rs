use sentry::ClientInitGuard;
use settings::Settings;

pub struct SentryTracing {
    _guard: Option<ClientInitGuard>,
}

impl SentryTracing {
    pub fn init(settings: &Settings, service: &str) -> Self {
        tracing_subscriber::fmt().with_target(false).init();
        let _guard = settings.sentry.as_ref().and_then(|sentry| {
            if sentry.dsn.is_empty() {
                return None;
            }

            let options = sentry::ClientOptions {
                dsn: Some(sentry.dsn.parse().ok()?),
                send_default_pii: false,
                sample_rate: sentry.sample_rate,
                traces_sample_rate: 0.0,
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

pub use sentry::{capture_message, configure_scope, Level};

pub fn error_with_context<E: std::error::Error + ?Sized>(message: &str, error: &E, context: &[(&str, &str)]) {
    tracing::error!("{}: {}", message, error);
    sentry::configure_scope(|scope| {
        scope.set_tag("error_type", error.to_string());
        for (key, value) in context {
            scope.set_tag(key, value);
        }
    });
    sentry::capture_message(message, sentry::Level::Error);
}

pub fn info_with_context(message: &str, context: &[(&str, &str)]) {
    let mut fields = vec![];
    for (key, value) in context {
        fields.push(format!("{}={}", key, value));
    }
    if fields.is_empty() {
        tracing::info!("{}", message);
    } else {
        tracing::info!("{} {}", message, fields.join(" "));
    }
}

pub fn warn_with_context(message: &str, context: &[(&str, &str)]) {
    let mut fields = vec![];
    for (key, value) in context {
        fields.push(format!("{}={}", key, value));
    }
    if fields.is_empty() {
        tracing::warn!("{}", message);
    } else {
        tracing::warn!("{} {}", message, fields.join(" "));
    }
    sentry::configure_scope(|scope| {
        for (key, value) in context {
            scope.set_tag(key, value);
        }
    });
    sentry::capture_message(message, sentry::Level::Warning);
}

#[macro_export]
macro_rules! warn_ctx {
    ($message:expr, $($var:ident),* $(,)?) => {
        {
            let context = &[
                $(
                    (stringify!($var), &$var.to_string()),
                )*
            ];
            $crate::warn_with_context($message, context);
        }
    };
}


pub fn error<E: std::error::Error + ?Sized>(message: &str, error: &E) {
    tracing::error!("{}: {}", message, error);
    sentry::capture_error(error);
}
