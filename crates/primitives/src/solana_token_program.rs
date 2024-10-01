use serde::Serialize;
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Serialize, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SolanaTokenProgramId {
    Token,
    Token2022,
}
