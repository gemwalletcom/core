use rocket::Request;
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::{FromRequest, Outcome};

use crate::responders::cache_error;

const AUTHORIZATION_HEADER: &str = "Authorization";
const BEARER_PREFIX: &str = "Bearer ";

fn error_outcome<T>(req: &Request<'_>, status: Status, message: &str) -> Outcome<T, String> {
    cache_error(req, message);
    Error((status, message.to_string()))
}

#[derive(Debug, Clone)]
pub struct AdminConfig {
    pub token: String,
}

pub struct AdminAuthorized;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminAuthorized {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        let Success(config) = req.guard::<&rocket::State<AdminConfig>>().await else {
            return error_outcome(req, Status::InternalServerError, "Admin config not available");
        };

        if config.token.is_empty() {
            return error_outcome(req, Status::InternalServerError, "Admin token is not configured");
        }

        let Some(auth_value) = req.headers().get_one(AUTHORIZATION_HEADER) else {
            return error_outcome(req, Status::Unauthorized, "Missing Authorization header");
        };

        if !auth_value.starts_with(BEARER_PREFIX) {
            return error_outcome(req, Status::Unauthorized, "Invalid authorization format");
        }

        if auth_value[BEARER_PREFIX.len()..] != config.token {
            return error_outcome(req, Status::Unauthorized, "Invalid admin token");
        }

        Success(AdminAuthorized)
    }
}

#[cfg(test)]
mod tests {
    use rocket::http::{Header, Status};
    use rocket::local::asynchronous::Client;
    use rocket::{Build, Rocket, get, routes};

    use super::{AdminAuthorized, AdminConfig};

    #[get("/protected")]
    async fn protected(_admin: AdminAuthorized) -> &'static str {
        "ok"
    }

    fn rocket(config: AdminConfig) -> Rocket<Build> {
        rocket::build().manage(config).mount("/", routes![protected])
    }

    fn bearer_header(token: &str) -> Header<'static> {
        Header::new("Authorization", format!("Bearer {token}"))
    }

    #[rocket::async_test]
    async fn test_invalid_token_returns_unauthorized() {
        let client = Client::tracked(rocket(AdminConfig { token: "secret".to_string() })).await.unwrap();

        let response = client.get("/protected").header(bearer_header("wrong")).dispatch().await;

        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[rocket::async_test]
    async fn test_correct_token_returns_ok() {
        let client = Client::tracked(rocket(AdminConfig { token: "secret".to_string() })).await.unwrap();

        let response = client.get("/protected").header(bearer_header("secret")).dispatch().await;

        assert_eq!(response.status(), Status::Ok);
    }

    #[rocket::async_test]
    async fn test_enabled_with_empty_token_returns_internal_server_error() {
        let client = Client::tracked(rocket(AdminConfig { token: String::new() })).await.unwrap();

        let response = client.get("/protected").dispatch().await;

        assert_eq!(response.status(), Status::InternalServerError);
    }
}
