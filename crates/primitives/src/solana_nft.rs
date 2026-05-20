use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum SolanaNftStandard {
    NonFungible,
    ProgrammableNonFungible { rule_set: Option<String> },
    Core { collection: Option<String> },
}
