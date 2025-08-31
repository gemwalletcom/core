use reqwest::{retry, StatusCode};
use std::future::Future;
use std::time::Duration;

#[cfg(feature = "reqwest")]
use tokio::time::sleep;

/// Create a retry policy for API requests that handles common HTTP error scenarios
///
/// NOTE: This uses reqwest's built-in retry mechanism which does NOT implement exponential backoff.
/// It will retry immediately without delays, which may not be suitable for rate limiting scenarios.
/// For rate limiting with proper backoff, use the `retry()` function instead.
pub fn retry_policy<S>(host: S, max_retries: u32) -> retry::Builder
where
    S: for<'a> PartialEq<&'a str> + Send + Sync + 'static,
{
    retry::for_host(host).max_retries_per_request(max_retries).classify_fn(|req_rep| {
        match req_rep.status() {
            Some(StatusCode::TOO_MANY_REQUESTS)
            | Some(StatusCode::INTERNAL_SERVER_ERROR)
            | Some(StatusCode::BAD_GATEWAY)
            | Some(StatusCode::SERVICE_UNAVAILABLE)
            | Some(StatusCode::GATEWAY_TIMEOUT) => req_rep.retryable(),
            None => req_rep.retryable(), // Network errors
            _ => req_rep.success(),
        }
    })
}

/// Retry policy with exponential backoff for rate limiting and transient errors
///
/// This function provides proper exponential backoff (2^attempt seconds) for handling
/// HTTP errors and other transient failures. Uses async sleep when reqwest
/// feature is enabled, otherwise falls back to blocking sleep.
///
/// # Arguments
/// * `operation` - A closure that returns a Future to be retried
/// * `max_retries` - Maximum number of retry attempts
/// * `should_retry_fn` - Optional predicate function to determine if error should trigger retry
///   If None, defaults to clearly transient errors (429, 502, 503, 504, throttling)
///
/// # Example
/// ```rust
/// use gem_client::retry::{retry, default_should_retry};
///
/// // Retry on clearly transient errors (429, 502, 503, 504, throttling) - default behavior
/// let result = retry(
///     || async { api_client.get("/endpoint").await },
///     3,
///     None
/// ).await;
///
/// // Custom retry logic
/// let result = retry(
///     || async { api_client.get("/endpoint").await },
///     3,
///     Some(|error| error.to_string().contains("429"))
/// ).await;
///
/// // Use explicit predefined function
/// let result = retry(
///     || async { api_client.get("/endpoint").await },
///     3,
///     Some(default_should_retry)
/// ).await;
/// ```
pub async fn retry<T, E, F, Fut, P>(operation: F, max_retries: u32, should_retry_fn: Option<P>) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
    P: Fn(&E) -> bool,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                let should_retry_error = match &should_retry_fn {
                    Some(predicate) => predicate(&err),
                    None => default_should_retry(&err),
                };

                if should_retry_error && attempt < max_retries {
                    attempt += 1;
                    // Exponential backoff: 2^attempt seconds (2s, 4s, 8s, ...) with max cap
                    let delay = Duration::from_secs(2_u64.saturating_pow(attempt).min(1800)); // Cap at 30 minutes
                    tracing::warn!(
                        error = %err,
                        attempt = attempt,
                        max_retries = max_retries,
                        delay_secs = delay.as_secs(),
                        "Retrying after error"
                    );

                    #[cfg(feature = "reqwest")]
                    sleep(delay).await;

                    #[cfg(not(feature = "reqwest"))]
                    std::thread::sleep(delay);

                    continue;
                }

                return Err(err);
            }
        }
    }
}

/// Default retry predicate for clearly transient errors
///
/// Retries on:
/// - 429 (Too Many Requests)
/// - 502 (Bad Gateway)
/// - 503 (Service Unavailable)
/// - 504 (Gateway Timeout)
/// - "too many requests" and "throttled" messages
pub fn default_should_retry<E: std::fmt::Display>(error: &E) -> bool {
    let error_str = error.to_string().to_lowercase();

    error_str.contains("429") ||                    // Too Many Requests
    error_str.contains("502") ||                    // Bad Gateway
    error_str.contains("503") ||                    // Service Unavailable
    error_str.contains("504") ||                    // Gateway Timeout
    error_str.contains("too many requests") ||      // Rate limiting messages
    error_str.contains("throttled") // Throttling messages
}
