use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmosisMintParamsResponse {
    pub params: OsmosisMintParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmosisMintParams {
    pub epoch_identifier: String,
    pub distribution_proportions: OsmosisDistributionProportions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmosisDistributionProportions {
    pub staking: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmosisEpochProvisionsResponse {
    pub epoch_provisions: String,
}