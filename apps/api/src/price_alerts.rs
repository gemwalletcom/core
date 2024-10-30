extern crate rocket;
use primitives::PriceAlerts;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

pub use pricer::PriceAlertClient;

#[get("/price_alerts/<device_id>")]
pub async fn get_price_alerts(device_id: &str, client: &State<Mutex<PriceAlertClient>>) -> Json<PriceAlerts> {
    let values = client.lock().await.get_price_alerts(device_id).await.unwrap();
    Json(values)
}

#[post("/price_alerts/<device_id>", format = "json", data = "<subscriptions>")]
pub async fn add_price_alerts(
    device_id: &str,
    subscriptions: Json<PriceAlerts>,
    client: &State<Mutex<PriceAlertClient>>,
) -> Json<usize> {
    let result = client.lock().await.add_price_alerts(device_id, subscriptions.0).await.unwrap();
    Json(result)
}

#[delete("/price_alerts/<device_id>", format = "json", data = "<subscriptions>")]
pub async fn delete_price_alerts(
    device_id: &str,
    subscriptions: Json<PriceAlerts>,
    client: &State<Mutex<PriceAlertClient>>,
) -> Json<usize> {
    let result = client.lock().await.delete_price_alerts(device_id, subscriptions.0).await.unwrap();
    Json(result)
}
