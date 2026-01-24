use crate::alien::AlienError;
use gem_jsonrpc::types::{ERROR_CLIENT_ERROR, JsonRpcError};
use std::{error::Error, fmt::Display};

/// Errors that can occur during gateway operations.
#[derive(Debug, Clone, uniffi::Error)]
pub enum GatewayError {
    /// Network-related errors such as timeouts, connection failures, or HTTP errors.
    NetworkError { msg: String },
    /// Non-network errors from platform code (Kotlin/Swift), allowing clients to
    /// distinguish and map back to original error types (e.g., BlockchainError.DustError).
    PlatformError { msg: String },
}

impl Display for GatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError { msg } => write!(f, "Network error: {}", msg),
            Self::PlatformError { msg } => write!(f, "Platform error: {}", msg),
        }
    }
}

impl Error for GatewayError {}

pub(crate) fn map_network_error(error: Box<dyn Error + Send + Sync>) -> GatewayError {
    if let Some(jsonrpc_error) = error.downcast_ref::<JsonRpcError>().filter(|candidate| candidate.code == ERROR_CLIENT_ERROR) {
        return GatewayError::NetworkError {
            msg: jsonrpc_error.message.clone(),
        };
    }

    let message = if let Some(status) = http_status_from_error(error.as_ref()) {
        let error_message = error.to_string();
        if error_message.is_empty() {
            format!("HTTP error: status {}", status)
        } else {
            error_message
        }
    } else {
        error.to_string()
    };

    GatewayError::NetworkError { msg: message }
}

fn http_status_from_error(error: &(dyn Error + 'static)) -> Option<u16> {
    let mut current_error: Option<&(dyn Error + 'static)> = Some(error);

    while let Some(err) = current_error {
        if let Some(AlienError::Http { status, .. }) = err.downcast_ref::<AlienError>() {
            return Some(*status);
        }

        if let Some(gem_client::ClientError::Http { status, .. }) = err.downcast_ref::<gem_client::ClientError>() {
            return Some(*status);
        }

        current_error = err.source();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_network_error_with_status_code() {
        let error = AlienError::Http { status: 404, body: Vec::new() };
        let mapped = map_network_error(Box::new(error));

        match mapped {
            GatewayError::NetworkError { msg } => assert_eq!(msg, "HTTP error: status 404"),
            GatewayError::PlatformError { .. } => panic!("Expected NetworkError"),
        }
    }

    #[test]
    fn test_map_network_error_with_jsonrpc_status_code() {
        let error = JsonRpcError {
            code: ERROR_CLIENT_ERROR,
            message: "HTTP error: status 404".to_string(),
        };
        let mapped = map_network_error(Box::new(error));

        match mapped {
            GatewayError::NetworkError { msg } => assert_eq!(msg, "HTTP error: status 404"),
            GatewayError::PlatformError { .. } => panic!("Expected NetworkError"),
        }
    }
}
