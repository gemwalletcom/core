use crate::responders::ErrorContext;
use primitives::ResponseResult;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Request, catch};

#[catch(default)]
pub fn default_catcher(status: Status, req: &Request) -> (Status, Json<ResponseResult<()>>) {
    let context = req.local_cache(|| ErrorContext(String::new()));
    let message = if context.0.is_empty() {
        format!("{} {}", status.code, status.reason_lossy())
    } else {
        context.0.clone()
    };
    (status, Json(ResponseResult::error(message)))
}
