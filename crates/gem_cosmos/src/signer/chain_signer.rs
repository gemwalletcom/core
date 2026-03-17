use std::str::FromStr;

use base64::{Engine, engine::general_purpose::STANDARD};
use gem_hash::sha2::sha256;
use primitives::{ChainSigner, SignerError, TransactionLoadInput, chain_cosmos::CosmosChain};
use signer::{SignatureScheme, Signer};

use super::transaction::{CosmosTxParams, encode_auth_info, encode_sign_doc, encode_tx_body, encode_tx_raw};
use crate::models::{Coin, CosmosMessage};

const BASE_FEE_GAS_UNITS: u64 = 200_000;
const GAS_BUFFER_NUMERATOR: u64 = 13;
const GAS_BUFFER_DENOMINATOR: u64 = 10;

#[derive(Default)]
pub struct CosmosChainSigner;

impl ChainSigner for CosmosChainSigner {
    fn sign_swap(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap_data = input.input_type.get_swap_data().map_err(SignerError::invalid_input)?;
        let account_number = input.metadata.get_account_number().map_err(SignerError::from_display)?;
        let sequence = input.metadata.get_sequence().map_err(SignerError::from_display)?;
        let chain_id = input.metadata.get_chain_id().map_err(SignerError::from_display)?;
        let chain = CosmosChain::from_str(input.input_type.get_asset().chain.as_ref()).map_err(|_| SignerError::invalid_input("unsupported cosmos chain"))?;

        let message = CosmosMessage::parse(&swap_data.data.data)?;
        let body_bytes = encode_tx_body(&[message.encode_as_any()], input.memo.as_deref().unwrap_or(""));

        let gas_limit = swap_data
            .data
            .gas_limit
            .as_ref()
            .and_then(|g| g.parse::<u64>().ok())
            .filter(|&g| g > 0)
            .unwrap_or(BASE_FEE_GAS_UNITS);
        let gas_limit = gas_limit * GAS_BUFFER_NUMERATOR / GAS_BUFFER_DENOMINATOR;

        let base_fee: u64 = input
            .gas_price
            .gas_price()
            .to_string()
            .parse()
            .map_err(|_| SignerError::invalid_input("invalid gas price"))?;
        let fee_amount = ((gas_limit as u128 * base_fee as u128 / BASE_FEE_GAS_UNITS as u128) as u64).to_string();

        let params = CosmosTxParams {
            body_bytes,
            chain_id: &chain_id,
            account_number,
            sequence,
            fee_coins: vec![Coin {
                denom: chain.denom().as_ref().to_string(),
                amount: fee_amount,
            }],
            gas_limit,
        };

        Ok(vec![Self::encode_and_sign_tx(&params, private_key)?])
    }
}

impl CosmosChainSigner {
    pub fn encode_and_sign_tx(params: &CosmosTxParams, private_key: &[u8]) -> Result<String, SignerError> {
        let pubkey_bytes = signer::secp256k1_public_key(private_key)?;
        let auth_info_bytes = encode_auth_info(&pubkey_bytes, params.sequence, &params.fee_coins, params.gas_limit);
        let sign_doc_bytes = encode_sign_doc(&params.body_bytes, &auth_info_bytes, params.chain_id, params.account_number);

        let digest = sha256(&sign_doc_bytes);
        let mut signature = Signer::sign_digest(SignatureScheme::Secp256k1, digest.to_vec(), private_key.to_vec())?;
        if signature.len() < 64 {
            return Err(SignerError::signing_error("secp256k1 signature too short"));
        }
        signature.truncate(64);

        let tx_raw = encode_tx_raw(&params.body_bytes, &auth_info_bytes, &signature);
        let tx_base64 = STANDARD.encode(&tx_raw);
        Ok(serde_json::json!({
            "mode": "BROADCAST_MODE_SYNC",
            "tx_bytes": tx_base64,
        })
        .to_string())
    }
}
