use crate::config::JwtConfig;
use crate::metrics::Metrics;
use gem_auth::verify_device_token;
use primitives::{AuthStatus, X_AUTH_STATUS};
use rocket::http::Status;
use rocket::request::FromRequest;
use rocket::response::{self, Responder, Response};
use rocket::{Request, State};

pub struct BearerToken(Option<String>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BearerToken {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, ()> {
        let token = request
            .headers()
            .get_one("Authorization")
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|t| t.to_string());
        rocket::request::Outcome::Success(BearerToken(token))
    }
}

pub struct AuthResponse(AuthStatus);

impl<'r> Responder<'r, 'static> for AuthResponse {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        Response::build().status(Status::Ok).raw_header(X_AUTH_STATUS, self.0.as_ref().to_string()).ok()
    }
}

#[rocket::get("/auth")]
pub async fn auth_endpoint(bearer: BearerToken, metrics: &State<Metrics>, jwt: &State<JwtConfig>) -> AuthResponse {
    let is_valid = bearer.0.as_deref().is_some_and(|token| verify_device_token(token, &jwt.secret).is_ok());
    let status = if is_valid { AuthStatus::Valid } else { AuthStatus::Invalid };
    metrics.add_auth_request(status.as_ref());
    AuthResponse(status)
}
