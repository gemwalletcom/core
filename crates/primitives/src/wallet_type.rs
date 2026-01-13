use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
#[allow(non_camel_case_types)]
pub enum WalletType {
    multicoin,
    single,
    private_key,
    view,
}
