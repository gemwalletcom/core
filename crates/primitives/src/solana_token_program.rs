use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SolanaTokenProgramId {
    Token,
    Token2022,
}
