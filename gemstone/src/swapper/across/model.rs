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

// "estimatedFillTimeSec": 8,
// "capitalFeePct": "78750000000001",
// "capitalFeeTotal": "78750000000001",
// "relayGasFeePct": "1537279254383",
// "relayGasFeeTotal": "1537279254383",
// "relayFeePct": "80287279254384",
// "relayFeeTotal": "80287279254384",
// "lpFeePct": "0",
// "timestamp": "1732711367",
// "isAmountTooLow": false,
// "quoteBlock": "21279160",
// "exclusiveRelayer": "0x0000000000000000000000000000000000000000",
// "exclusivityDeadline": 0,
// "spokePoolAddress": "0x5c7BCd6E7De5423a257D81B442095A1a6ced35C5",
// "destinationSpokePoolAddress": "0x6f26Bf09B1C792e3228e5467807a900A503c0281",
// "totalRelayFee": {
// "pct": "80287279254384",
// "total": "80287279254384"
// },
// "relayerCapitalFee": {
// "pct": "78750000000001",
// "total": "78750000000001"
// },
// "relayerGasFee": {
// "pct": "1537279254383",
// "total": "1537279254383"
// },
// "lpFee": {
// "pct": "0",
// "total": "0"
// },
// "limits": {
// "minDeposit": "144849863116879",
// "maxDeposit": "1342586751114515098801",
// "maxDepositInstant": "285583466574213585294",
// "maxDepositShortDelay": "1342586751114515098801",
// "recommendedDepositInstant": "285583466574213585294"
// }

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
