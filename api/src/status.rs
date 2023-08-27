extern crate rocket;
use rocket::serde::json::Json;
use rocket::serde::Serialize;   
use std::time::{SystemTime, UNIX_EPOCH};

#[get("/")]
pub fn get_status(
    ip: std::net::IpAddr,
) -> Json<Status> {
   Json(Status { 
        time: get_epoch_ms(),
        ipv4: ip.to_string(),
    })
}

fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[derive(Serialize)]
pub struct Status {
    time: u128,
    ipv4: String,
}

