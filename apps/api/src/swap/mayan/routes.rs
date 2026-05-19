use super::client::{MayanProxyClient, MayanProxyResponse};
use crate::responders::ApiError;
use rocket::{
    Data, Request, State,
    data::ToByteUnit,
    request::{FromRequest, Outcome},
};
use std::path::PathBuf;

const MAYAN_PROXY_BODY_LIMIT_KIB: u64 = 64;

pub struct MayanProxyQuery(Option<String>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for MayanProxyQuery {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(Self(request.uri().query().map(|query| query.as_str().to_string())))
    }
}

#[rocket::get("/proxy/swap/mayan/<api>/<version>/<path..>")]
pub async fn get_mayan_proxy(api: &str, version: &str, path: PathBuf, query: MayanProxyQuery, client: &State<MayanProxyClient>) -> Result<MayanProxyResponse, ApiError> {
    client.get(api, version, &path, query.0.as_deref()).await
}

#[rocket::post("/proxy/swap/mayan/<api>/<version>/<path..>", data = "<data>")]
pub async fn post_mayan_proxy(
    api: &str,
    version: &str,
    path: PathBuf,
    query: MayanProxyQuery,
    data: Data<'_>,
    client: &State<MayanProxyClient>,
) -> Result<MayanProxyResponse, ApiError> {
    let body = read_body(data).await?;
    client.post(api, version, &path, query.0.as_deref(), body).await
}

async fn read_body(data: Data<'_>) -> Result<Vec<u8>, ApiError> {
    let bytes = data.open(MAYAN_PROXY_BODY_LIMIT_KIB.kibibytes()).into_bytes().await.map_err(ApiError::bad_request)?;
    if !bytes.is_complete() {
        return Err(ApiError::BadRequest("Mayan proxy request body is too large".to_string()));
    }
    Ok(bytes.into_inner())
}
