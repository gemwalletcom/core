use cacher::CacheError;
use primitives::ResponseResult;
use rocket::response::{Responder, Response};
use rocket::serde::json::Json;
use rocket::{http::Status, Request};

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    InternalServerError(String),
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (status, message) = match self {
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
            match current_error.source() {
                Some(source) => current_error = source,
                None => break,
            }
        }

        ApiError::InternalServerError("Service error".to_string())
    }
}
