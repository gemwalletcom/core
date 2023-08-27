extern crate rocket;
use primitives::chain::Chain;
use rocket::tokio::sync::Mutex;
use rocket::serde::json::Json;
use rocket::State;
use name_resolver::client::Client as NameClient;
use primitives::name::NameRecord;

#[get("/name/resolve/<name>?<chain>")]
pub async fn get_name_resolve(
    name: &str,
    chain: &str,
    name_client: &State<Mutex<NameClient>>,
) -> Json<NameRecord> {
    let chain = Chain::new(chain).unwrap();
    let name = name_client.lock().await.resolve(name, chain).await.unwrap();
    Json(name)
}
