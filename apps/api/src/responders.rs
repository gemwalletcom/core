use cacher::CacheError;
use primitives::ResponseResult;
use rocket::response::{Responder, Response};
use rocket::serde::json::Json;
use rocket::{Request, http::Status};
use serde::Serialize;
use storage::DatabaseError;
use strum::ParseError;

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
            DatabaseError::Internal(msg) => ApiError::InternalServerError(msg),
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
            if let Some(db_error) = current_error.downcast_ref::<DatabaseError>() {
                return db_error.clone().into();
            }
            match current_error.source() {
                Some(source) => current_error = source,
                None => break,
            }
        }

        ApiError::InternalServerError("Service error".to_string())
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
