use crate::alien::AlienError;
use gem_jsonrpc::types::{ERROR_CLIENT_ERROR, JsonRpcError};
use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, uniffi::Error)]
pub enum GatewayError {
    NetworkError { msg: String },
}

impl Display for GatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError { msg: message } => write!(f, "Network error: {}", message),
        }
    }
}

impl Error for GatewayError {}

pub(crate) fn map_network_error(error: Box<dyn Error + Send + Sync>) -> GatewayError {
    if let Some(jsonrpc_error) = error.downcast_ref::<JsonRpcError>()
        && jsonrpc_error.code == ERROR_CLIENT_ERROR
    {
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
        if let Some(alien_error) = err.downcast_ref::<AlienError>()
            && let AlienError::Http { status, .. } = alien_error
        {
            return Some(*status);
        }

        if let Some(client_error) = err.downcast_ref::<gem_client::ClientError>()
            && let gem_client::ClientError::Http { status, .. } = client_error
        {
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
        let error = AlienError::Http { status: 404, len: 0 };
        let mapped = map_network_error(Box::new(error));

        match mapped {
            GatewayError::NetworkError { msg } => {
                assert_eq!(msg, "HTTP error: status 404");
            }
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
        }
    }
}
