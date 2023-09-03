extern crate rocket;
use primitives::device::Device;
use rocket::serde::json::Json;
use self::client::DevicesClient;
use rocket::State;
use rocket::tokio::sync::Mutex;

pub mod client;

#[post("/devices", format = "json", data = "<device>")]
pub async fn add_device(
    device: Json<Device>, 
    client: &State<Mutex<DevicesClient>>,
) -> Json<Device> {
    let device = client.lock().await.add_device(device.0).unwrap();
    Json(device)
}

#[get("/devices/<device_id>")]
pub async fn get_device(
    device_id: &str,
    client: &State<Mutex<DevicesClient>>,
) -> Json<Device> {
    let device = client.lock().await.get_device(device_id).unwrap();
    Json(device)
}

#[put("/devices/<device_id>", format = "json", data = "<device>")]
pub async fn update_device(
    device: Json<Device>, 
    #[allow(unused)]
    device_id: &str,
    client: &State<Mutex<DevicesClient>>,
) -> Json<Device> {
    let device = client.lock().await.update_device(device.0).unwrap();
    Json(device)
}