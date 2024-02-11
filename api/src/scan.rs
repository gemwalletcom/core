extern crate rocket;
use crate::scan_client::ScanClient;
use primitives::ScanAddress;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

#[get("/scan/address/<chain>/<address>")]
pub async fn get_scan_address(
    chain: &str,
    address: &str,
    client: &State<Mutex<ScanClient>>,
) -> Json<ScanAddress> {
    let chain = Chain::from_str(chain).unwrap();
    let address = client
        .lock()
        .await
        .get_scan_address(chain, address)
        .unwrap();
    Json(address)
}
