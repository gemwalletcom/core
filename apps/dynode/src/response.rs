use std::io::Cursor;

use crate::proxy::ProxyResponse;
use rocket::Request;
use rocket::http::{ContentType, Status};
use rocket::response::{Responder, Response};
use serde_json::json;

pub struct ErrorResponse {
    status: Status,
    message: String,
}

impl ErrorResponse {
    pub fn new(status: Status, message: String) -> Self {
        Self { status, message }
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for ErrorResponse {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        let body = json!({
            "error": self.status.reason_lossy(),
            "message": self.message,
            "code": self.status.code
        })
        .to_string();

        Response::build()
            .status(self.status)
            .header(ContentType::JSON)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

pub struct ProxyRocketResponse(pub ProxyResponse);

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for ProxyRocketResponse {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        let ProxyResponse { status, headers, body } = self.0;

        let mut builder = Response::build();
        let status = Status::from_code(status).unwrap_or(Status::Ok);
        builder.status(status);

        for (name, value) in headers.iter() {
            if let Ok(value_str) = value.to_str() {
                builder.raw_header(name.as_str().to_string(), value_str.to_string());
            }
        }

        let body_len = body.len();
        builder.sized_body(body_len, Cursor::new(body));
        Ok(builder.finalize())
    }
}
