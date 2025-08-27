use serde::{Deserialize, Serialize};

use crate::models::AssetDetails;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetResponse {
    pub asset: AssetDetails,
}
