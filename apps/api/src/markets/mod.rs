extern crate rocket;

use pricer::MarketsClient;
use primitives::{Markets, ResponseResult};
use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

#[get("/markets")]
pub async fn get_markets(client: &State<Mutex<MarketsClient>>) -> Result<Json<ResponseResult<Markets>>, NotFound<String>> {
    match client.lock().await.get_markets().await {
        Ok(data) => Ok(Json(ResponseResult::new(data))),
        Err(err) => Err(NotFound(err.to_string())),
    }
}
