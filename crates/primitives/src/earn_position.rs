use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::earn_position_base::EarnPositionBase;
use crate::earn_provider::EarnProvider;
use crate::Price;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct EarnPosition {
    pub base: EarnPositionBase,
    pub provider: EarnProvider,
    pub price: Option<Price>,
}
