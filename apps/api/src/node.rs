extern crate rocket;
use primitives::node::NodesResponse;
use rocket::serde::json::Json;
#[get("/nodes")]
pub async fn get_nodes() -> Json<NodesResponse> {
    Json(NodesResponse {
        version: 1,
        nodes: vec![],
    })
}
