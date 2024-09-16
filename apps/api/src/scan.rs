extern crate rocket;
use crate::scan_client::ScanClient;
use primitives::{Chain, ScanAddress};
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;
use security_provider::{ScanRequest, ScanResult};
use std::str::FromStr;

#[get("/scan/address/<chain>/<address>")]
pub async fn get_scan_address(chain: &str, address: &str, client: &State<Mutex<ScanClient>>) -> Json<ScanAddress> {
    let chain = Chain::from_str(chain).unwrap();
    let address = client.lock().await.get_scan_address(chain, address).unwrap();
    Json(address)
}

#[post("/scan/security", data = "<scan_request>")]
pub async fn scan_security(scan_request: Json<ScanRequest>, client: &State<Mutex<ScanClient>>) -> Json<ScanResult> {
    let result = client.lock().await.scan_security(scan_request.0).await.unwrap();
    Json(result)
}
