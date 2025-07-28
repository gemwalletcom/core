pub mod client;
pub use client::SubscriptionsClient;

use primitives::Subscription;
use rocket::{delete, get, post, serde::json::Json, tokio::sync::Mutex, State};

#[get("/subscriptions/<device_id>")]
pub async fn get_subscriptions(device_id: &str, client: &State<Mutex<SubscriptionsClient>>) -> Json<Vec<Subscription>> {
    let subscriptions = client.lock().await.get_subscriptions_by_device_id(device_id).await.unwrap();
    Json(subscriptions)
}

#[delete("/subscriptions/<device_id>", format = "json", data = "<subscriptions>")]
pub async fn delete_subscriptions(subscriptions: Json<Vec<Subscription>>, device_id: &str, client: &State<Mutex<SubscriptionsClient>>) -> Json<usize> {
    let result = client.lock().await.delete_subscriptions(device_id, subscriptions.0).await.unwrap();
    Json(result)
}

#[post("/subscriptions/<device_id>", format = "json", data = "<subscriptions>")]
pub async fn add_subscriptions(subscriptions: Json<Vec<Subscription>>, device_id: &str, client: &State<Mutex<SubscriptionsClient>>) -> Json<usize> {
    let subscriptions = client.lock().await.add_subscriptions(device_id, subscriptions.0).await.unwrap();
    Json(subscriptions)
}
