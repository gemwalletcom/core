use pricer::MarketsClient;
use primitives::{Markets, ResponseResult};
use rocket::response::status::NotFound;
use rocket::{get, serde::json::Json, tokio::sync::Mutex, State};

#[get("/markets")]
pub async fn get_markets(client: &State<Mutex<MarketsClient>>) -> Result<Json<ResponseResult<Markets>>, NotFound<String>> {
    match client.lock().await.get_markets().await {
        Ok(data) => Ok(Json(ResponseResult::new(data))),
        Err(err) => Err(NotFound(err.to_string())),
    }
}
