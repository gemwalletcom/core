extern crate rocket;
use std::str::FromStr;

use name_resolver::client::Client as NameClient;
use primitives::chain::Chain;
use primitives::name::NameRecord;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

#[get("/name/resolve/<name>?<chain>")]
pub async fn get_name_resolve(
    name: &str,
    chain: &str,
    name_client: &State<Mutex<NameClient>>,
) -> Json<NameRecord> {
    let chain = Chain::from_str(chain).unwrap();
    let name = name_client.lock().await.resolve(name, chain).await.unwrap();
    Json(name)
}
