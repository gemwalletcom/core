extern crate rocket;
use crate::DevicesClient;
use primitives::device::Device;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

#[post("/devices", format = "json", data = "<device>")]
pub async fn add_device(
    device: Json<Device>,
    client: &State<Mutex<DevicesClient>>,
) -> Json<Device> {
    let device = client.lock().await.add_device(device.0).unwrap();
    Json(device)
}

#[get("/devices/<device_id>")]
pub async fn get_device(device_id: &str, client: &State<Mutex<DevicesClient>>) -> Json<Device> {
    let device = client.lock().await.get_device(device_id).unwrap();
    Json(device)
}

#[put("/devices/<device_id>", format = "json", data = "<device>")]
pub async fn update_device(
    device: Json<Device>,
    #[allow(unused)] device_id: &str,
    client: &State<Mutex<DevicesClient>>,
) -> Json<Device> {
    let device = client.lock().await.update_device(device.0).unwrap();
    Json(device)
}

#[post("/devices/<device_id>/push-notification")]
pub async fn send_push_notification_device(
    device_id: &str,
    client: &State<Mutex<DevicesClient>>,
) -> Json<bool> {
    let result = client
        .lock()
        .await
        .send_push_notification_device(device_id)
        .await
        .unwrap();
    Json(result)
}

#[delete("/devices/<device_id>")]
pub async fn delete_device(device_id: &str, client: &State<Mutex<DevicesClient>>) -> Json<usize> {
    let result: usize = client.lock().await.delete_device(device_id).unwrap();
    Json(result)
}
