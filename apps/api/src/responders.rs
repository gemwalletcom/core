use std::time::{SystemTime, UNIX_EPOCH};

use cacher::CacheError;
use fiat::error::FiatQuoteError;
use gem_auth::verify_device_signature;
use primitives::ResponseResult;
use rocket::response::{Responder, Response};
use rocket::serde::json::Json;
use rocket::{Request, http::Status};
use serde::Serialize;
use storage::DatabaseError;
use strum::ParseError;

const SIGNATURE_TIMESTAMP_TOLERANCE_MS: u64 = 300_000;

pub fn verify_request_signature(req: &Request<'_>, public_key: &str) -> Result<(), (Status, String)> {
    let signature = req
        .headers()
        .get_one("x-device-signature")
        .ok_or((Status::Unauthorized, "Missing x-device-signature".to_string()))?;
    let timestamp_str = req
        .headers()
        .get_one("x-device-timestamp")
        .ok_or((Status::Unauthorized, "Missing x-device-timestamp".to_string()))?;
    let body_hash = req
        .headers()
        .get_one("x-device-body-hash")
        .ok_or((Status::Unauthorized, "Missing x-device-body-hash".to_string()))?;

    let timestamp_ms: u64 = timestamp_str.parse().map_err(|_| (Status::Unauthorized, "Invalid timestamp".to_string()))?;
    let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;

    if now_ms.abs_diff(timestamp_ms) > SIGNATURE_TIMESTAMP_TOLERANCE_MS {
        return Err((Status::Unauthorized, "Timestamp expired".to_string()));
    }

    let method = req.method().as_str();
    let path = req.uri().path().as_str();
    let message = format!("v1.{timestamp_str}.{method}.{path}.{body_hash}");

    if !verify_device_signature(public_key, &message, signature) {
        return Err((Status::Unauthorized, "Invalid signature".to_string()));
    }

    Ok(())
}

pub struct ErrorContext(pub String);

pub fn cache_error(req: &Request<'_>, message: &str) {
    req.local_cache(|| ErrorContext(message.to_string()));
}

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    InternalServerError(String),
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (Status::BadRequest, msg),
            ApiError::NotFound(msg) => (Status::NotFound, msg),
            ApiError::InternalServerError(msg) => (Status::InternalServerError, msg),
        };

        let error_response = ResponseResult::<()>::error(message);
        let json_response = Json(error_response);

        Response::build_from(json_response.respond_to(request)?).status(status).ok()
    }
}

impl From<CacheError> for ApiError {
    fn from(error: CacheError) -> Self {
        match error {
            CacheError::NotFound(key) => ApiError::NotFound(format!("Asset not found: {}", key)),
        }
    }
}

impl From<ParseError> for ApiError {
    fn from(error: ParseError) -> Self {
        ApiError::NotFound(format!("Invalid parameter: {}", error))
    }
}

impl From<DatabaseError> for ApiError {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::NotFound => ApiError::NotFound(error.to_string()),
            DatabaseError::Error(msg) => ApiError::InternalServerError(msg),
        }
    }
}

impl From<FiatQuoteError> for ApiError {
    fn from(error: FiatQuoteError) -> Self {
        ApiError::BadRequest(error.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for ApiError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        let mut current_error: &dyn std::error::Error = error.as_ref();
        loop {
            if let Some(cache_error) = current_error.downcast_ref::<CacheError>() {
                let reconstructed = match cache_error {
                    CacheError::NotFound(key) => CacheError::NotFound(key.clone()),
                };
                return reconstructed.into();
            }
            if let Some(db_error) = current_error.downcast_ref::<DatabaseError>() {
                return db_error.clone().into();
            }
            match current_error.source() {
                Some(source) => current_error = source,
                None => break,
            }
        }

        ApiError::InternalServerError(format!("{}", error))
    }
}

pub struct ApiResponse<T>(pub ResponseResult<T>);

impl<T> From<T> for ApiResponse<T> {
    fn from(data: T) -> Self {
        ApiResponse(ResponseResult::new(data))
    }
}

impl<'r, T: Serialize> Responder<'r, 'static> for ApiResponse<T> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        Json(self.0).respond_to(request)
    }
}
