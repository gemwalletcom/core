uniffi::include_scaffolding!("gemstone");
use async_std::future::{pending, timeout};

#[uniffi::export(callback_interface)]
pub trait GreetingDelegate: Send + Sync {
    fn greeting_called(&self, to: String);
}

pub struct GreetingLogger {
    delegate: Box<dyn GreetingDelegate>,
}

impl GreetingLogger {
    pub fn new(delegate: Box<dyn GreetingDelegate>) -> Self {
        Self { delegate }
    }

    pub fn greeting_called(&self, to: String) {
        self.delegate.greeting_called(to)
    }
}

static LOGGER_INSTANCE: once_cell::sync::OnceCell<GreetingLogger> =
    once_cell::sync::OnceCell::new();

#[uniffi::export]
pub fn set_logging_delegate(delegate: Box<dyn GreetingDelegate>) {
    let logger = GreetingLogger::new(delegate);
    let result = LOGGER_INSTANCE.set(logger);
    if result.is_err() {
        panic!("Logger already set");
    }
}

// #[uniffi::export]
pub fn rust_greeting(to: String) -> String {
    if let Some(logger) = LOGGER_INSTANCE.get() {
        logger.greeting_called(to.clone());
    }
    format!("Hello, {}!", to)
}

#[uniffi::export]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[uniffi::export]
pub async fn say_after(ms: u64, who: String) -> String {
    let never = pending::<()>();
    timeout(std::time::Duration::from_millis(ms), never)
        .await
        .unwrap_err();
    format!("Hello, {who}!")
}

#[derive(uniffi::Record)]
pub struct RustDemoObj {
    pub value: i64,
}

#[uniffi::export]
pub fn add_obj(a: &RustDemoObj, b: &RustDemoObj) -> RustDemoObj {
    RustDemoObj {
        value: a.value + b.value,
    }
}
