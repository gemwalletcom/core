use crate::responders::ApiError;
use rocket::{
    Request,
    http::{ContentType, Status},
    response::{Responder, Response},
};
use std::io::Cursor;

pub struct MayanProxyResponse {
    status: Status,
    body: Vec<u8>,
}

impl MayanProxyResponse {
    pub(super) async fn from_response(response: reqwest::Response) -> Result<Self, ApiError> {
        let status = Status::from_code(response.status().as_u16()).unwrap_or(Status::InternalServerError);
        let body = response.bytes().await.map_err(ApiError::internal_server_error)?.to_vec();
        Ok(Self { status, body })
    }
}

impl<'r> Responder<'r, 'static> for MayanProxyResponse {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'static> {
        Response::build()
            .status(self.status)
            .header(ContentType::JSON)
            .sized_body(self.body.len(), Cursor::new(self.body))
            .ok()
    }
}
