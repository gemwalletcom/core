extern crate rocket;
use primitives::node::NodesResponse;
use rocket::serde::json::Json;
use rocket::State;
use rocket::tokio::sync::Mutex;
use crate::node_client::Client as NodeClient;

#[get("/nodes")]
pub async fn get_nodes(
    node_client: &State<Mutex<NodeClient>>,
) -> Json<NodesResponse> {
    let response = node_client.lock().await.get_nodes().await;
    match response {
        Ok(result) => Json(result),
        Err(_) => Json(NodesResponse{version: 1, nodes: vec![]}),
    }
}