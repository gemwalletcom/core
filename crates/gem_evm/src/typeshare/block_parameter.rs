use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable, Hashable")]
#[serde(rename_all = "lowercase")]
pub enum EthereumBlockParameter {
    Latest,
    Earliest,
    Pending,
    Finalized,
    Safe,
}
