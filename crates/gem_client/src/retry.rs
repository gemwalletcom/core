use reqwest::{retry, StatusCode};

/// Create a retry policy for API requests that handles common HTTP error scenarios
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

/// Standard retry policy for API requests (3 retries)
pub fn standard_retry_policy<S>(host: S) -> retry::Builder
where
    S: for<'a> PartialEq<&'a str> + Send + Sync + 'static,
{
    retry_policy(host, 3)
}

/// Aggressive retry policy for external APIs (10 retries)
pub fn aggressive_retry_policy<S>(host: S) -> retry::Builder
where
    S: for<'a> PartialEq<&'a str> + Send + Sync + 'static,
{
    retry_policy(host, 10)
}
