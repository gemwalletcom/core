use std::str::FromStr;

use name_resolver::client::Client as NameClient;
use primitives::{chain::Chain, name::NameRecord};
use rocket::{get, response::status::NotFound, serde::json::Json, tokio::sync::Mutex, State};

#[get("/name/resolve/<name>?<chain>")]
pub async fn get_name_resolve(name: &str, chain: &str, name_client: &State<Mutex<NameClient>>) -> Result<Json<NameRecord>, NotFound<String>> {
    let chain = Chain::from_str(chain).unwrap();
    let result = name_client.lock().await.resolve(name, chain).await;
    match result {
        Ok(name) => Ok(Json(name)),
        Err(err) => Err(NotFound(err.to_string())),
    }
}
