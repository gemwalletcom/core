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
/// * `status_codes` - Optional list of HTTP status codes to retry on (e.g., [429, 502, 503, 504])
///   If None, defaults to clearly transient errors (429, 502, 503, 504, throttling)
///
/// # Example
/// ```rust
/// use gem_client::retry::retry;
///
/// // Retry on clearly transient errors (429, 502, 503, 504, throttling) - default behavior
/// let result = retry(
///     || async { api_client.get("/endpoint").await },
///     3,
///     None
/// ).await;
///
/// // Retry only on rate limiting (429)
/// let result = retry(
///     || async { api_client.get("/endpoint").await },
///     3,
///     Some(vec![429])
/// ).await;
///
/// // Retry on multiple specific status codes (including 500 if desired)
/// let result = retry(
///     || async { api_client.get("/endpoint").await },
///     3,
///     Some(vec![429, 500, 502, 503, 504])
/// ).await;
/// ```
pub async fn retry<T, E, F, Fut>(operation: F, max_retries: u32, status_codes: Option<Vec<u16>>) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                if should_retry(&err, &status_codes) && attempt < max_retries {
                    attempt += 1;
                    // Exponential backoff: 2^attempt seconds (2s, 4s, 8s, ...)
                    let delay = Duration::from_secs(2_u64.pow(attempt));
                    println!(
                        "Retrying after error: {} (attempt {}/{}, waiting {}s)",
                        err,
                        attempt,
                        max_retries,
                        delay.as_secs()
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

/// Check if an error should trigger a retry based on status codes
fn should_retry<E: std::fmt::Display>(error: &E, status_codes: &Option<Vec<u16>>) -> bool {
    let error_str = error.to_string().to_lowercase();

    match status_codes {
        Some(codes) => {
            // Check for specific status codes only
            codes.iter().any(|code| error_str.contains(&code.to_string()))
        }
        None => {
            error_str.contains("429") ||                    // Too Many Requests
            error_str.contains("502") ||                    // Bad Gateway
            error_str.contains("503") ||                    // Service Unavailable
            error_str.contains("504") ||                    // Gateway Timeout
            error_str.contains("too many requests") ||      // Rate limiting messages
            error_str.contains("throttled") // Throttling messages
        }
    }
}
