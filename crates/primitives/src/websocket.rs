use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetPrice, FiatRate};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct WebSocketPricePayload {
    pub prices: Vec<AssetPrice>,
    pub rates: Vec<FiatRate>,
}
