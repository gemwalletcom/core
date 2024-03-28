use primitives::PriceFull;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub results: Vec<PriceFull>,
}
