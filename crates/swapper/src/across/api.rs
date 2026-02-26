use crate::SwapperError;
use alloy_primitives::U256;
use alloy_sol_types::SolEvent;
use gem_evm::{across::contracts::V3SpokePoolInterface, parse_u256, rpc::model::Log};

pub(crate) struct ParsedDeposit {
    pub deposit_id: u64,
    pub origin_chain_id: u64,
    pub destination_chain_id: Option<u64>,
    pub input_token: Option<String>,
    pub output_token: Option<String>,
    pub input_amount: Option<String>,
    pub output_amount: Option<String>,
}

fn parse_topic_u64(topic: &str) -> Option<u64> {
    parse_u256(topic).map(|v| v.to::<u64>())
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

pub(crate) fn parse_deposit_from_logs(logs: &[Log], origin_chain_id: u64) -> Result<ParsedDeposit, SwapperError> {
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

    let deposit_id = parse_topic_u64(&log.topics[2]).ok_or_else(|| SwapperError::TransactionError("failed to parse deposit ID".into()))?;

    let (origin, destination) = if is_fill {
        let origin = parse_topic_u64(&log.topics[1]).ok_or_else(|| SwapperError::TransactionError("failed to parse origin chain ID".into()))?;
        (origin, None)
    } else {
        let destination = parse_topic_u64(&log.topics[1]);
        (origin_chain_id, destination)
    };

    let (input_token, output_token, input_amount, output_amount) = alloy_primitives::hex::decode(&log.data)
        .ok()
        .and_then(|d| decode_token_amounts(&d))
        .map(|(a, b, c, d)| (Some(a), Some(b), Some(c), Some(d)))
        .unwrap_or_default();

    Ok(ParsedDeposit {
        deposit_id,
        origin_chain_id: origin,
        destination_chain_id: destination,
        input_token,
        output_token,
        input_amount,
        output_amount,
    })
}

pub(crate) fn filled_relay_topic() -> String {
    format!("{:#x}", V3SpokePoolInterface::FilledRelay::SIGNATURE_HASH)
}
