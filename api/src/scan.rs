extern crate rocket;
use primitives::ScanAddress;
use rocket::serde::json::Json;
use crate::scan_client::ScanClient;
use rocket::State;
use rocket::tokio::sync::Mutex;

#[get("/scan/address/<address>")]
pub async fn get_scan_address(
    address: &str,
    client: &State<Mutex<ScanClient>>,
) -> Json<ScanAddress> {
    let address = client.lock().await.get_scan_address(address).unwrap();
    Json(address)
}