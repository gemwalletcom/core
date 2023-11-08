extern crate rocket;
use rocket::serde::json::Json;
use crate::SwapClient;
use rocket::State;
use rocket::tokio::sync::Mutex;

#[get("/swap/quote")]
pub async fn get_swap_quote(
    //device_id: &str,
    client: &State<Mutex<SwapClient>>,
) -> Json<bool> {
    Json(true)
    //let subscriptions = client.lock().await.get_subscriptions(device_id).unwrap();
    //Json(subscriptions)
}