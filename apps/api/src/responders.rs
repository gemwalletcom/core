use cacher::CacheError;
use fiat::error::FiatQuoteError;
use primitives::ResponseResult;
use rocket::response::{Responder, Response};
use rocket::serde::json::Json;
use rocket::{Request, http::Status};
use serde::Serialize;
use storage::DatabaseError;
use strum::ParseError;

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
            CacheError::NotFound { .. } | CacheError::ResourceNotFound(_) => ApiError::NotFound(error.to_string()),
            CacheError::KeyNotFound(_) => ApiError::InternalServerError("Unexpected cache miss".to_string()),
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
            DatabaseError::NotFound { .. } => ApiError::NotFound(error.to_string()),
            DatabaseError::Error(msg) => ApiError::InternalServerError(msg),
        }
    }
}

impl From<swapper::SwapperError> for ApiError {
    fn from(error: swapper::SwapperError) -> Self {
        ApiError::InternalServerError(error.to_string())
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
                return cache_error.clone().into();
            }
            if let Some(db_error) = current_error.downcast_ref::<DatabaseError>() {
                return db_error.clone().into();
            }
            if let Some(fiat_error) = current_error.downcast_ref::<FiatQuoteError>() {
                return fiat_error.clone().into();
            }
            match current_error.source() {
                Some(source) => current_error = source,
                None => break,
            }
        }

        ApiError::InternalServerError(format!("{}", error))
    }
}

#[cfg(test)]
mod tests {
    use super::ApiError;
    use cacher::CacheError;
    use storage::DatabaseError;

    #[test]
    fn test_cache_not_found_maps_to_public_not_found() {
        let error = ApiError::from(CacheError::not_found("FiatQuote", "abc"));
        match error {
            ApiError::NotFound(message) => assert_eq!(message, "FiatQuote abc not found"),
            _ => panic!("expected not found"),
        }
    }

    #[test]
    fn test_cache_key_not_found_maps_to_internal_server_error() {
        let error = ApiError::from(CacheError::KeyNotFound("fiat:quote:abc".to_string()));
        match error {
            ApiError::InternalServerError(message) => assert_eq!(message, "Unexpected cache miss"),
            _ => panic!("expected internal server error"),
        }
    }

    #[test]
    fn test_boxed_database_not_found_hides_internal_lookup() {
        let error: Box<dyn std::error::Error + Send + Sync> = Box::new(DatabaseError::not_found_internal("Device", "1"));
        match ApiError::from(error) {
            ApiError::NotFound(message) => assert_eq!(message, "Device not found"),
            _ => panic!("expected not found"),
        }
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
