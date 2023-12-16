extern crate rocket;
use crate::scan_client::ScanClient;
use primitives::ScanAddress;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

#[get("/scan/address/<address>")]
pub async fn get_scan_address(
    address: &str,
    client: &State<Mutex<ScanClient>>,
) -> Json<ScanAddress> {
    let address = client.lock().await.get_scan_address(address).unwrap();
    Json(address)
}
