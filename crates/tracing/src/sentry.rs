#[cfg(feature = "sentry")]
mod real_sentry {
    use sentry::ClientInitGuard;

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

    pub use sentry::{capture_message, configure_scope, start_transaction, Level, TransactionContext, capture_error};
}

#[cfg(not(feature = "sentry"))]
mod mock_sentry {
    pub struct SentryConfig {
        pub dsn: String,
        pub sample_rate: f32,
    }

    pub struct SentryTracing;

    impl SentryTracing {
        pub fn init(_config: Option<&SentryConfig>, _service: &str) -> Self {
            Self
        }
    }

    pub enum Level {
        Error,
    }

    pub struct TransactionContext;

    pub fn capture_message(_message: &str, _level: Level) {}

    pub fn configure_scope<F>(_f: F)
    where
        F: FnOnce(&mut Scope),
    {
    }

    pub fn start_transaction(_ctx: TransactionContext, _name: &str) -> Transaction {
        Transaction
    }

    pub fn capture_error<E: std::error::Error + ?Sized>(_error: &E) {}

    pub struct Scope;

    impl Scope {
        pub fn set_tag(&mut self, _key: &str, _value: String) {}
    }

    pub struct Transaction;

    impl Transaction {
        pub fn finish(self) {}
    }
}

#[cfg(feature = "sentry")]
pub use real_sentry::*;

#[cfg(not(feature = "sentry"))]
pub use mock_sentry::*;