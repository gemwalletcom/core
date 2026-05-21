use super::{
    constants::{MAYAN_FORWARDER, SDK_VERSION},
    wormhole_chain,
};
use crate::{SwapperError, amount_to_value, fees::default_referral_address};
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeSet, ops::Deref, str::FromStr};

use gem_evm::ethereum_address_checksum;
pub use gem_sui::tx_builder::transaction_json::TransactionArgument as SuiTransactionArgument;
use primitives::SolanaInstruction;
use primitives::swap::SwapStatus;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub quotes: Vec<MayanQuote>,
}

#[derive(Debug, Clone)]
pub struct QuoteParams {
    pub amount_in64: String,
    pub from_token: String,
    pub from_chain: String,
    pub to_token: String,
    pub to_chain: String,
    pub referrer: String,
    pub referrer_bps: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MayanQuote {
    Swift(Box<MayanSwiftQuote>),
    Mctp(Box<MayanMctpQuote>),
    FastMctp(Box<MayanFastMctpQuote>),
    MonoChain(Box<MayanMonoChainQuote>),
}

impl MayanQuote {
    pub fn common(&self) -> &MayanQuoteCommon {
        match self {
            Self::Swift(route) => &route.common,
            Self::Mctp(route) => &route.common,
            Self::FastMctp(route) => &route.common,
            Self::MonoChain(route) => &route.common,
        }
    }

    pub fn as_swift(&self) -> Option<&MayanSwiftQuote> {
        match self {
            Self::Swift(route) => Some(route.as_ref()),
            Self::Mctp(_) | Self::FastMctp(_) | Self::MonoChain(_) => None,
        }
    }

    pub fn as_mctp(&self) -> Option<&MayanMctpQuote> {
        match self {
            Self::Mctp(route) => Some(route.as_ref()),
            Self::Swift(_) | Self::FastMctp(_) | Self::MonoChain(_) => None,
        }
    }

    pub fn as_fast_mctp(&self) -> Option<&MayanFastMctpQuote> {
        match self {
            Self::FastMctp(route) => Some(route.as_ref()),
            Self::Swift(_) | Self::Mctp(_) | Self::MonoChain(_) => None,
        }
    }

    pub fn as_mono_chain(&self) -> Option<&MayanMonoChainQuote> {
        match self {
            Self::MonoChain(route) => Some(route.as_ref()),
            Self::Swift(_) | Self::Mctp(_) | Self::FastMctp(_) => None,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MayanQuoteCommon {
    pub effective_amount_in64: String,
    pub expected_amount_out: Value,
    pub min_amount_out: Value,
    pub gas_drop: Value,
    pub eta_seconds: u32,
    pub from_token: MayanToken,
    pub to_token: MayanToken,
    pub from_chain: String,
    pub to_chain: String,
    pub slippage_bps: u32,
    pub deadline64: Option<String>,
    pub referrer_bps: Option<u32>,
    pub expected_amount_out_base_units: Option<String>,
}

impl MayanQuoteCommon {
    pub(in crate::mayan) fn expected_output_value(&self, output_decimals: u32) -> Result<String, SwapperError> {
        let output_decimals = if output_decimals == 0 { self.to_token.decimals } else { output_decimals };
        if let Some(value) = &self.expected_amount_out_base_units {
            return BigUint::from_str(value)
                .map(|amount| rescale_base_units(amount, self.to_token.decimals, output_decimals).to_string())
                .map_err(SwapperError::from);
        }

        let amount = match &self.expected_amount_out {
            Value::Number(number) => number.to_string(),
            Value::String(value) => value.clone(),
            Value::Null | Value::Bool(_) | Value::Array(_) | Value::Object(_) => return Err(SwapperError::InvalidRoute),
        };
        BigNumberFormatter::value_from_amount(&amount, output_decimals).map_err(SwapperError::from)
    }
}

fn rescale_base_units(amount: BigUint, from_decimals: u32, to_decimals: u32) -> BigUint {
    match to_decimals.cmp(&from_decimals) {
        std::cmp::Ordering::Equal => amount,
        std::cmp::Ordering::Greater => amount * BigUint::from(10_u32).pow(to_decimals - from_decimals),
        std::cmp::Ordering::Less => amount / BigUint::from(10_u32).pow(from_decimals - to_decimals),
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MayanSwiftQuote {
    #[serde(flatten)]
    pub common: MayanQuoteCommon,
    pub refund_relayer_fee64: Option<String>,
    pub cancel_relayer_fee64: Option<String>,
    pub submit_relayer_fee64: Option<String>,
    pub protocol_bps: Option<u32>,
    pub min_middle_amount: Option<Value>,
    pub swift_mayan_contract: Option<String>,
    pub swift_auction_mode: Option<u8>,
    pub swift_input_contract: Option<String>,
    pub swift_input_decimals: Option<u32>,
    pub swift_input_contract_standard: Option<String>,
    pub swift_wrap_and_lock: Option<bool>,
    pub swift_version: Option<SwiftVersion>,
    pub hc_swift_deposit: Option<HcSwiftDeposit>,
    pub suggested_priority_fee: Option<u64>,
    pub gasless: Option<bool>,
    pub quote_id: Option<String>,
    pub max_swap_accounts: Option<u32>,
    pub max_swap_data_length: Option<u32>,
    pub memo_hex: Option<String>,
}

impl Deref for MayanSwiftQuote {
    type Target = MayanQuoteCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MayanMctpQuote {
    #[serde(flatten)]
    pub common: MayanQuoteCommon,
    pub min_middle_amount: Option<Value>,
    pub has_auction: Option<bool>,
    pub cheaper_chain: Option<String>,
    pub bridge_fee: Option<Value>,
    pub redeem_relayer_fee: Option<Value>,
    pub mctp_input_contract: Option<String>,
    pub mctp_verified_input_address: Option<String>,
    pub mctp_input_treasury: Option<String>,
    pub mctp_mayan_contract: Option<String>,
    pub solana_relayer_fee64: Option<String>,
    pub relayer: Option<String>,
    pub suggested_priority_fee: Option<u64>,
    pub max_swap_accounts: Option<u32>,
    pub max_swap_data_length: Option<u32>,
}

impl Deref for MayanMctpQuote {
    type Target = MayanQuoteCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MayanFastMctpQuote {
    #[serde(flatten)]
    pub common: MayanQuoteCommon,
    pub refund_relayer_fee64: Option<String>,
    pub redeem_relayer_fee64: Option<String>,
    pub redeem_relayer_fee: Option<Value>,
    pub min_middle_amount: Option<Value>,
    pub has_auction: Option<bool>,
    pub cheaper_chain: Option<String>,
    pub fast_mctp_mayan_contract: Option<String>,
    pub fast_mctp_input_contract: Option<String>,
    pub fast_mctp_min_finality: Option<u32>,
    pub circle_max_fee64: Option<String>,
    pub solana_relayer_fee64: Option<String>,
    pub relayer: Option<String>,
    pub suggested_priority_fee: Option<u64>,
    pub max_swap_accounts: Option<u32>,
    pub max_swap_data_length: Option<u32>,
}

impl Deref for MayanFastMctpQuote {
    type Target = MayanQuoteCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MayanMonoChainQuote {
    #[serde(flatten)]
    pub common: MayanQuoteCommon,
    pub mono_chain_mayan_contract: String,
    pub evm_swap_router_address: Option<String>,
    pub evm_swap_router_calldata: Option<String>,
}

impl Deref for MayanMonoChainQuote {
    type Target = MayanQuoteCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuoteType {
    #[default]
    Swift,
    Mctp,
    FastMctp,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum SwiftVersion {
    V1,
    V2,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MayanToken {
    pub contract: String,
    pub w_chain_id: u16,
    pub decimals: u32,
    pub verified_address: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HcSwiftDeposit {
    pub relayer_fee64: String,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSwapEvmParams {
    pub forwarder_address: &'static str,
    pub slippage_bps: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer_address: Option<String>,
    pub from_token: String,
    pub middle_token: String,
    pub chain_name: String,
    pub amount_in64: String,
    pub sdk_version: &'static str,
}

impl GetSwapEvmParams {
    pub fn swift(route: &MayanSwiftQuote, amount_in64: String, middle_token: String) -> Self {
        Self {
            forwarder_address: MAYAN_FORWARDER,
            slippage_bps: route.slippage_bps,
            referrer_address: wormhole_chain::chain_for_name(&route.from_chain)
                .ok()
                .map(default_referral_address)
                .filter(|address| !address.is_empty()),
            from_token: route.from_token.contract.clone(),
            middle_token,
            chain_name: route.from_chain.clone(),
            amount_in64,
            sdk_version: SDK_VERSION,
        }
    }

    pub fn mctp(route: &MayanMctpQuote, amount_in64: String, middle_token: String, referrer_address: Option<String>) -> Self {
        Self {
            forwarder_address: MAYAN_FORWARDER,
            slippage_bps: route.slippage_bps,
            referrer_address,
            from_token: route.from_token.contract.clone(),
            middle_token,
            chain_name: route.from_chain.clone(),
            amount_in64,
            sdk_version: SDK_VERSION,
        }
    }

    pub fn fast_mctp(route: &MayanFastMctpQuote, amount_in64: String, middle_token: String, referrer_address: Option<String>) -> Self {
        Self {
            forwarder_address: MAYAN_FORWARDER,
            slippage_bps: route.slippage_bps,
            referrer_address,
            from_token: route.from_token.contract.clone(),
            middle_token,
            chain_name: route.from_chain.clone(),
            amount_in64,
            sdk_version: SDK_VERSION,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSwapEvmResponse {
    pub swap_router_address: String,
    pub swap_router_calldata: String,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSwapSolanaParams {
    pub min_middle_amount: String,
    pub middle_token: String,
    pub user_wallet: String,
    pub slippage_bps: u32,
    pub from_token: String,
    pub amount_in64: String,
    pub deposit_mode: &'static str,
    pub fill_max_accounts: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tpm_token_account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer_address: Option<String>,
    pub chain_name: String,
    pub user_ledger: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_swap_accounts: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_swap_data_length: Option<u32>,
    pub sdk_version: &'static str,
}

impl GetSwapSolanaParams {
    pub fn swift(
        route: &MayanSwiftQuote,
        min_middle_amount: String,
        middle_token: String,
        user_wallet: String,
        amount_in64: String,
        referrer_address: Option<String>,
        user_ledger: String,
    ) -> Self {
        Self {
            min_middle_amount,
            middle_token,
            user_wallet,
            slippage_bps: route.slippage_bps,
            from_token: route.from_token.contract.clone(),
            amount_in64,
            deposit_mode: "SWIFT",
            fill_max_accounts: false,
            tpm_token_account: None,
            referrer_address,
            chain_name: route.from_chain.clone(),
            user_ledger,
            max_swap_accounts: route.max_swap_accounts,
            max_swap_data_length: route.max_swap_data_length,
            sdk_version: SDK_VERSION,
        }
    }

    pub fn mctp(
        route: &MayanMctpQuote,
        min_middle_amount: String,
        middle_token: String,
        user_wallet: String,
        amount_in64: String,
        deposit_mode: &'static str,
        referrer_address: Option<String>,
        user_ledger: String,
    ) -> Self {
        Self {
            min_middle_amount,
            middle_token,
            user_wallet,
            slippage_bps: route.slippage_bps,
            from_token: route.from_token.contract.clone(),
            amount_in64,
            deposit_mode,
            fill_max_accounts: false,
            tpm_token_account: None,
            referrer_address,
            chain_name: route.from_chain.clone(),
            user_ledger,
            max_swap_accounts: route.max_swap_accounts,
            max_swap_data_length: route.max_swap_data_length,
            sdk_version: SDK_VERSION,
        }
    }

    pub fn fast_mctp(
        route: &MayanFastMctpQuote,
        min_middle_amount: String,
        middle_token: String,
        user_wallet: String,
        amount_in64: String,
        deposit_mode: &'static str,
        referrer_address: Option<String>,
        user_ledger: String,
    ) -> Self {
        Self {
            min_middle_amount,
            middle_token,
            user_wallet,
            slippage_bps: route.slippage_bps,
            from_token: route.from_token.contract.clone(),
            amount_in64,
            deposit_mode,
            fill_max_accounts: false,
            tpm_token_account: None,
            referrer_address,
            chain_name: route.from_chain.clone(),
            user_ledger,
            max_swap_accounts: route.max_swap_accounts,
            max_swap_data_length: route.max_swap_data_length,
            sdk_version: SDK_VERSION,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaClientSwap {
    pub compute_budget_instructions: Option<Vec<SolanaInstruction>>,
    pub setup_instructions: Option<Vec<SolanaInstruction>>,
    pub swap_instruction: SolanaInstruction,
    pub cleanup_instruction: Option<SolanaInstruction>,
    pub address_lookup_table_addresses: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSwapSuiParams {
    pub amount_in64: String,
    pub input_coin_type: String,
    pub middle_coin_type: String,
    pub user_wallet: String,
    pub with_wh_fee: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer_address: Option<String>,
    pub slippage_bps: u32,
    pub chain_name: String,
    pub sdk_version: &'static str,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiClientSwap {
    pub tx: String,
    pub out_coin: SuiTransactionArgument,
    pub wh_fee_coin: Option<SuiTransactionArgument>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorData {
    pub min_amount_in: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorResponse {
    pub msg: Option<String>,
    pub message: Option<String>,
    pub data: Option<ErrorData>,
}

impl ErrorResponse {
    const AMOUNT_TOO_SMALL: &str = "amount too small";

    pub fn message(&self) -> Option<&str> {
        self.msg.as_deref().or(self.message.as_deref())
    }

    pub fn is_input_amount_error(&self) -> bool {
        if self.data.as_ref().and_then(|data| data.min_amount_in.as_ref()).is_some() {
            return true;
        }

        match self.message() {
            Some(message) => message.to_ascii_lowercase().contains(Self::AMOUNT_TOO_SMALL),
            None => false,
        }
    }

    pub fn min_amount_in(&self, decimals: u32) -> Option<String> {
        self.data
            .as_ref()
            .and_then(|data| data.min_amount_in.as_ref())
            .and_then(|amount| amount_value(amount, decimals))
            .or_else(|| self.message().and_then(|message| extract_min_amount(message, decimals)))
    }
}

fn amount_value(value: &Value, decimals: u32) -> Option<String> {
    match value {
        Value::Number(number) => amount_to_value(&number.to_string(), decimals),
        Value::String(value) => amount_to_value(value, decimals),
        Value::Null | Value::Bool(_) | Value::Array(_) | Value::Object(_) => None,
    }
}

fn extract_min_amount(message: &str, decimals: u32) -> Option<String> {
    let lowercased = message.to_ascii_lowercase();
    let start = lowercased.find("min")?;
    let amount = message[start + "min".len()..]
        .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == ',' || c == '_'))
        .find(|part| part.chars().any(|c| c.is_ascii_digit()))?;
    amount_to_value(amount, decimals)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MayanTransactionResult {
    pub from_token_address: String,
    pub to_token_address: String,
    pub from_token_chain: String,
    pub to_token_chain: String,
    pub from_amount64: Option<String>,
    pub to_amount64: Option<String>,
    pub client_status: MayanClientStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MayanClientStatus {
    Completed,
    InProgress,
    Refunded,
}

impl MayanClientStatus {
    pub fn swap_status(&self) -> SwapStatus {
        match self {
            MayanClientStatus::Completed => SwapStatus::Completed,
            MayanClientStatus::Refunded => SwapStatus::Failed,
            MayanClientStatus::InProgress => SwapStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MayanChain {
    pub mayan_address: String,
}

fn checksum_address(address: &str) -> String {
    ethereum_address_checksum(address).unwrap_or_else(|_| address.to_string())
}

impl MayanChain {
    pub fn unique_addresses(chains: Vec<MayanChain>) -> Vec<String> {
        chains
            .into_iter()
            .filter(|c| !c.mayan_address.is_empty())
            .map(|c| checksum_address(&c.mayan_address))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_quote_response() {
        let response: QuoteResponse = serde_json::from_str(include_str!("test/quote_response_swift.json")).unwrap();

        let value = serde_json::to_value(&response.quotes[0]).unwrap();
        assert_eq!(value.get("type").and_then(Value::as_str), Some("SWIFT"));

        let quote = response.quotes[0].as_swift().unwrap();
        assert_eq!(quote.swift_version, Some(SwiftVersion::V2));
        assert_eq!(quote.from_token.w_chain_id, 2);
        assert_eq!(quote.to_token.decimals, 9);
    }

    #[test]
    fn test_decode_fast_mctp_quote() {
        let route: MayanQuote = serde_json::from_str(include_str!("test/fast_mctp_quote.json")).unwrap();

        let route = route.as_fast_mctp().unwrap();
        assert_eq!(route.fast_mctp_input_contract.as_deref(), Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"));
        assert_eq!(route.fast_mctp_min_finality, Some(1000));
        assert_eq!(route.circle_max_fee64.as_deref(), Some("500"));
        assert_eq!(route.redeem_relayer_fee64.as_deref(), Some("100000"));
    }

    #[test]
    fn test_expected_output_value() {
        let mut route = MayanQuoteCommon {
            expected_amount_out_base_units: Some("1237897283".to_string()),
            to_token: MayanToken {
                decimals: 9,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(route.expected_output_value(9).unwrap(), "1237897283");

        route.expected_amount_out_base_units = None;
        route.expected_amount_out = serde_json::json!(1.237897283);
        route.to_token = MayanToken {
            decimals: 9,
            ..Default::default()
        };
        assert_eq!(route.expected_output_value(9).unwrap(), "1237897283");
    }

    #[test]
    fn test_expected_output_value_rescales_base_units_to_asset_decimals() {
        let route = MayanQuoteCommon {
            expected_amount_out_base_units: Some("6023337".to_string()),
            to_token: MayanToken {
                decimals: 6,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(route.expected_output_value(8).unwrap(), "602333700");
    }

    #[test]
    fn test_decode_sui_client_swap() {
        let response: SuiClientSwap = serde_json::from_str(include_str!("test/sui_client_swap.json")).unwrap();

        assert_eq!(response.out_coin, SuiTransactionArgument::Result { result: 4 });
        assert_eq!(response.wh_fee_coin, Some(SuiTransactionArgument::NestedResult { nested_result: [0, 0] }));
    }

    #[test]
    fn test_decode_mayan_transaction_result() {
        let result: MayanTransactionResult = serde_json::from_str(include_str!("test/eth_to_sui_swift.json")).unwrap();
        assert_eq!(result.client_status, MayanClientStatus::Completed);
        assert_eq!(result.from_amount64, Some("18124254".to_string()));
        assert!(result.to_amount64.is_none());

        let result: MayanTransactionResult = serde_json::from_str(include_str!("test/mctp_pending.json")).unwrap();
        assert_eq!(result.client_status, MayanClientStatus::InProgress);
        assert_eq!(result.from_amount64, Some("529066169".to_string()));
        assert!(result.to_amount64.is_none());

        let result: MayanTransactionResult = serde_json::from_str(include_str!("test/swift_refunded.json")).unwrap();
        assert_eq!(result.client_status, MayanClientStatus::Refunded);
        assert_eq!(result.from_amount64, Some("4000000000000000".to_string()));
        assert!(result.to_amount64.is_none());
    }

    #[test]
    fn test_token_chain_fields() {
        let result: MayanTransactionResult = serde_json::from_str(include_str!("test/eth_to_sui_swift.json")).unwrap();
        assert_eq!(result.from_token_chain, "2");
        assert_eq!(result.to_token_chain, "21");
        assert_eq!(result.from_token_address, "0x0000000000000000000000000000000000000000");
        assert_eq!(result.to_token_address, "0x2::sui::SUI");

        let result: MayanTransactionResult = serde_json::from_str(include_str!("test/mctp_pending.json")).unwrap();
        assert_eq!(result.from_token_chain, "30");
        assert_eq!(result.to_token_chain, "23");
        assert_eq!(result.from_token_address, "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913");
        assert_eq!(result.to_token_address, "0xaf88d065e77c8cc2239327c5edb3a432268e5831");

        let result: MayanTransactionResult = serde_json::from_str(include_str!("test/swift_refunded.json")).unwrap();
        assert_eq!(result.from_token_chain, "2");
        assert_eq!(result.to_token_chain, "1");
    }
}
