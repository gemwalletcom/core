extern crate rocket;
use crate::metrics_client::MetricsClient;
use rocket::State;
use rocket::tokio::sync::Mutex;

#[get("/")]
pub async fn get_metrics(
    client: &State<Mutex<MetricsClient>>,
) -> String {
    
    client.lock().await.get()
}