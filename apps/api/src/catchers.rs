use crate::responders::ErrorContext;
use gem_tracing::info_with_fields;
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
    let user_agent = req.headers().get_one("User-Agent").unwrap_or("unknown");
    let uri = req.uri().to_string();
    info_with_fields!("Request failed", uri = uri, status = status.code, error = message, user_agent = user_agent);
    (status, Json(ResponseResult::error(message)))
}
