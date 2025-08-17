use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRPTokenId {
    pub issuer: String,
    pub currency: String,
}
