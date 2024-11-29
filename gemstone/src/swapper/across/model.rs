use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaggestedFeesRequest {
    pub inputToken: String,
    pub outputToken: String,
    pub amount: String,
    pub originChainId: String,
    pub destinationChainId: String,
    pub recipient: Option<String>,
    pub message: Option<String>,
    pub relayer: Option<String>,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaggestedFeesResponse {
    pub estimatedFillTimeSec: u64,
    pub capitalFeePct: String,
    pub capitalFeeTotal: String,
    pub isAmountTooLow: bool,
    pub quoteBlock: String,
    pub destinationSpokePoolAddress: String,
    pub timestamp: String,
    pub spokePoolAddress: String,
    pub totalRelayFee: FeeStruct,
    pub relayerCapitalFee: FeeStruct,
    pub relayerGasFee: FeeStruct,
    pub lpFee: FeeStruct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeStruct {
    pub pct: String,
    pub total: String,
}
