use crate::{
    SwapperError,
    alien::{RpcProvider, Target},
    client_factory::create_eth_client,
};
use alloy_primitives::U256;
use alloy_sol_types::SolEvent;
use gem_evm::{across::contracts::V3SpokePoolInterface, rpc::model::Log};
use primitives::{Chain, swap::SwapStatus};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AcrossApi {
    pub url: String,
    pub provider: Arc<dyn RpcProvider>,
}

impl AcrossApi {
    pub fn new(url: String, provider: Arc<dyn RpcProvider>) -> Self {
        Self { url, provider }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositStatus {
    pub status: String,
    pub deposit_id: String,
    pub deposit_tx_hash: String,
    pub fill_tx: Option<String>,
    pub destination_chain_id: u64,
    pub deposit_refund_tx_hash: Option<String>,
    #[serde(skip)]
    pub origin_chain_id: Option<u64>,
    #[serde(skip)]
    pub input_token: Option<String>,
    #[serde(skip)]
    pub output_token: Option<String>,
    #[serde(skip)]
    pub input_amount: Option<String>,
    #[serde(skip)]
    pub output_amount: Option<String>,
}

impl DepositStatus {
    pub fn swap_status(&self) -> SwapStatus {
        match self.status.as_str() {
            "filled" => SwapStatus::Completed,
            "refunded" => SwapStatus::Failed,
            _ => SwapStatus::Pending,
        }
    }
}

pub(crate) struct ParsedDeposit {
    pub deposit_id: String,
    pub origin_chain_id: String,
    pub input_token: Option<String>,
    pub output_token: Option<String>,
    pub input_amount: Option<String>,
    pub output_amount: Option<String>,
}

fn parse_topic_u256(topic: &str) -> Option<U256> {
    U256::from_str_radix(topic.strip_prefix("0x").unwrap_or(topic), 16).ok()
}

fn decode_token_amounts(data: &[u8]) -> Option<(String, String, String, String)> {
    if data.len() < 128 {
        return None;
    }
    let input_token = format!("0x{}", alloy_primitives::hex::encode(&data[12..32]));
    let output_token = format!("0x{}", alloy_primitives::hex::encode(&data[44..64]));
    let input_amount = U256::from_be_slice(&data[64..96]).to_string();
    let output_amount = U256::from_be_slice(&data[96..128]).to_string();
    Some((input_token, output_token, input_amount, output_amount))
}

pub(crate) fn parse_deposit_from_logs(logs: &[Log], origin_chain_id: &str) -> Result<ParsedDeposit, SwapperError> {
    let event_topics = [
        (format!("{:#x}", V3SpokePoolInterface::FundsDeposited::SIGNATURE_HASH), false),
        (format!("{:#x}", V3SpokePoolInterface::V3FundsDeposited::SIGNATURE_HASH), false),
        (format!("{:#x}", V3SpokePoolInterface::FilledRelay::SIGNATURE_HASH), true),
    ];

    let (log, is_fill) = event_topics
        .iter()
        .find_map(|(topic, is_fill)| logs.iter().find(|l| l.topics.first().is_some_and(|t| t == topic)).map(|l| (l, *is_fill)))
        .ok_or_else(|| SwapperError::TransactionError("FundsDeposited event not found".into()))?;

    if log.topics.len() < 3 {
        return Err(SwapperError::TransactionError("invalid event topics".into()));
    }

    let deposit_id = parse_topic_u256(&log.topics[2])
        .ok_or_else(|| SwapperError::TransactionError("failed to parse deposit ID".into()))?
        .to_string();

    let origin = if is_fill {
        parse_topic_u256(&log.topics[1])
            .ok_or_else(|| SwapperError::TransactionError("failed to parse origin chain ID".into()))?
            .to_string()
    } else {
        origin_chain_id.to_string()
    };

    let (input_token, output_token, input_amount, output_amount) = alloy_primitives::hex::decode(&log.data)
        .ok()
        .and_then(|d| decode_token_amounts(&d))
        .map(|(a, b, c, d)| (Some(a), Some(b), Some(c), Some(d)))
        .unwrap_or_default();

    Ok(ParsedDeposit {
        deposit_id,
        origin_chain_id: origin,
        input_token,
        output_token,
        input_amount,
        output_amount,
    })
}

impl AcrossApi {
    pub async fn deposit_status(&self, chain: Chain, tx_hash: &str) -> Result<DepositStatus, SwapperError> {
        let receipt = create_eth_client(self.provider.clone(), chain)?
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(SwapperError::from)?;

        let parsed = parse_deposit_from_logs(&receipt.logs, chain.network_id())?;

        let url = format!("{}/api/deposit/status?originChainId={}&depositId={}", self.url, parsed.origin_chain_id, parsed.deposit_id);
        let target = Target::get(&url);
        let response = self.provider.request(target).await?;
        let mut status: DepositStatus = serde_json::from_slice(&response.data).map_err(SwapperError::from)?;

        status.origin_chain_id = parsed.origin_chain_id.parse::<u64>().ok();
        status.input_token = parsed.input_token;
        status.output_token = parsed.output_token;
        status.input_amount = parsed.input_amount;
        status.output_amount = parsed.output_amount;

        Ok(status)
    }
}
