extern crate rocket;
use crate::scan_client::ScanClient;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;
use security_provider::{ScanRequest, ScanResult};

#[post("/scan/security", data = "<scan_request>")]
pub async fn scan_security(scan_request: Json<ScanRequest>, client: &State<Mutex<ScanClient>>) -> Json<Vec<ScanResult>> {
    let result = client.lock().await.scan_security(scan_request.0).await.unwrap();
    Json(result)
}
