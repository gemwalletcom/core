use crate::{
    network::{AlienProvider, AlienTarget},
    swapper::SwapperError,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct LayerZeroScanApi {
    pub url: String,
    pub provider: Arc<dyn AlienProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    pub data: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub pathway: MessagePathway,
    pub source: MessageSource,
    pub destination: MessageDestination,
    pub verification: MessageVerification,
    pub guid: String,
    pub config: MessageConfig,
    pub status: MessageStatus,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagePathway {
    pub src_eid: u32,
    pub dst_eid: u32,
    pub sender: MessageParticipant,
    pub receiver: MessageParticipant,
    pub id: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageParticipant {
    pub address: String,
    pub id: Option<String>,
    pub name: Option<String>,
    pub chain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSource {
    pub status: String,
    pub tx: SourceTransaction,
    #[serde(rename = "failedTx")]
    pub failed_tx: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceTransaction {
    pub tx_hash: String,
    pub block_hash: String,
    pub block_number: String,
    pub block_timestamp: u64,
    pub from: String,
    pub block_confirmations: u64,
    pub payload: String,
    pub value: String,
    pub readiness_timestamp: u64,
    pub resolved_payload: String,
    pub adapter_params: AdapterParams,
    pub options: TransactionOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdapterParams {
    pub version: String,
    pub dst_gas_limit: String,
    pub dst_native_gas_transfer_amount: String,
    pub dst_native_gas_transfer_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOptions {
    #[serde(rename = "lzReceive")]
    pub lz_receive: LzReceive,
    #[serde(rename = "nativeDrop")]
    pub native_drop: Vec<NativeDrop>,
    pub compose: Vec<Compose>,
    pub ordered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LzReceive {
    pub gas: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeDrop {
    pub amount: String,
    pub receiver: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compose {
    pub index: u64,
    pub gas: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDestination {
    pub status: String,
    pub tx: DestinationTransaction,
    #[serde(rename = "payloadStoredTx")]
    pub payload_stored_tx: String,
    #[serde(rename = "failedTx")]
    pub failed_tx: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DestinationTransaction {
    pub tx_hash: String,
    pub block_hash: String,
    pub block_number: u64,
    pub block_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageVerification {
    pub dvn: Dvn,
    pub sealer: Sealer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dvn {
    pub dvns: HashMap<String, DvnInfo>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DvnInfo {
    pub tx_hash: String,
    pub block_hash: String,
    pub block_number: u64,
    pub block_timestamp: u64,
    pub proof: DvnProof,
    pub optional: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DvnProof {
    #[serde(rename = "packetHeader")]
    pub packet_header: String,
    #[serde(rename = "payloadHash")]
    pub payload_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sealer {
    pub tx: SealerTransaction,
    #[serde(rename = "failedTx")]
    pub failed_tx: Vec<FailedTransaction>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SealerTransaction {
    pub tx_hash: String,
    pub block_hash: String,
    pub block_number: u64,
    pub block_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedTransaction {
    pub tx_hash: String,
    pub tx_error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageConfig {
    pub error: bool,
    pub error_message: String,
    pub dvn_config_error: bool,
    pub receive_library: Option<String>,
    pub send_library: Option<String>,
    pub inbound_config: InboundConfig,
    pub outbound_config: OutboundConfig,
    pub uln_send_version: String,
    pub uln_receive_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundConfig {
    pub confirmations: u64,
    pub required_dvn_count: u64,
    pub optional_dvn_count: u64,
    pub optional_dvn_threshold: u64,
    pub required_dvns: Vec<String>,
    pub required_dvn_names: Vec<String>,
    pub optional_dvns: Vec<String>,
    pub optional_dvn_names: Vec<String>,
    pub executor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboundConfig {
    pub confirmations: u64,
    pub required_dvn_count: u64,
    pub optional_dvn_count: u64,
    pub optional_dvn_threshold: u64,
    pub required_dvns: Vec<String>,
    pub required_dvn_names: Vec<String>,
    pub optional_dvns: Vec<String>,
    pub optional_dvn_names: Vec<String>,
    pub executor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageStatusName {
    Inflight,
    Confirming,
    Failed,
    Delivered,
    Blocked,
    PayloadStored,
    ApplicationBurned,
    ApplicationSkipped,
    UnresolvableCommand,
    MalformedCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStatus {
    pub name: MessageStatusName,
    pub message: Option<String>,
}

impl MessageStatus {
    pub fn is_delivered(&self) -> bool {
        matches!(self.name, MessageStatusName::Delivered)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.name, MessageStatusName::Failed)
    }

    pub fn is_pending(&self) -> bool {
        matches!(
            self.name,
            MessageStatusName::Inflight | MessageStatusName::Confirming | MessageStatusName::PayloadStored
        )
    }
}

impl LayerZeroScanApi {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self {
            url: "https://scan.layerzero-api.com/v1".into(),
            provider,
        }
    }

    pub async fn get_message_by_tx(&self, tx_hash: &str) -> Result<MessageResponse, SwapperError> {
        let url = format!("{}/messages/tx/{}", self.url, tx_hash);
        let target = AlienTarget::get(&url);
        let response = self.provider.request(target).await?;
        serde_json::from_slice(&response).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })
    }
}
