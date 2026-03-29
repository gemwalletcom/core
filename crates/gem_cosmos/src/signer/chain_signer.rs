use std::str::FromStr;

use base64::{Engine, engine::general_purpose::STANDARD};
use gem_hash::{keccak::keccak256, sha2::sha256};
use primitives::{ChainSigner, SignerError, SignerInput, chain_cosmos::CosmosChain};
use signer::{SignatureScheme, Signer};

use super::transaction::{COSMOS_SECP256K1_PUBKEY_TYPE, CosmosTxParams, INJECTIVE_ETHSECP256K1_PUBKEY_TYPE};
use crate::models::{Coin, CosmosMessage};

const BASE_FEE_GAS_UNITS: u64 = 200_000;
const GAS_BUFFER_NUMERATOR: u64 = 13;
const GAS_BUFFER_DENOMINATOR: u64 = 10;

#[derive(Default)]
pub struct CosmosChainSigner;

impl ChainSigner for CosmosChainSigner {
    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap_data = input.input_type.get_swap_data().map_err(SignerError::invalid_input)?;
        let account_number = input.metadata.get_account_number().map_err(SignerError::from_display)?;
        let sequence = input.metadata.get_sequence().map_err(SignerError::from_display)?;
        let chain_id = input.metadata.get_chain_id().map_err(SignerError::from_display)?;
        let chain = CosmosChain::from_str(input.input_type.get_asset().chain.as_ref()).map_err(|_| SignerError::invalid_input("unsupported cosmos chain"))?;

        let messages = CosmosMessage::parse_array(&swap_data.data.data)?;
        let encoded: Vec<Vec<u8>> = messages.iter().map(|m| m.encode_as_any()).collect();
        let body_bytes = CosmosTxParams::encode_tx_body(&encoded, input.memo.as_deref().unwrap_or(""));

        let gas_limit = swap_data
            .data
            .gas_limit
            .as_ref()
            .and_then(|g| g.parse::<u64>().ok())
            .filter(|&g| g > 0)
            .ok_or_else(|| SignerError::invalid_input("missing or invalid gas_limit"))?;
        let gas_limit = gas_limit * GAS_BUFFER_NUMERATOR / GAS_BUFFER_DENOMINATOR;

        let base_fee = input.fee.gas_price_u64()?;
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
            pubkey_type: Self::pubkey_type(chain),
        };

        Ok(vec![Self::encode_and_sign_tx(chain, &params, private_key)?])
    }
}

impl CosmosChainSigner {
    fn pubkey_type(chain: CosmosChain) -> &'static str {
        match chain {
            CosmosChain::Injective => INJECTIVE_ETHSECP256K1_PUBKEY_TYPE,
            _ => COSMOS_SECP256K1_PUBKEY_TYPE,
        }
    }

    fn sign_doc_digest(chain: CosmosChain, sign_doc_bytes: &[u8]) -> [u8; 32] {
        match chain {
            CosmosChain::Injective => keccak256(sign_doc_bytes),
            _ => sha256(sign_doc_bytes),
        }
    }

    pub fn encode_and_sign_tx(chain: CosmosChain, params: &CosmosTxParams, private_key: &[u8]) -> Result<String, SignerError> {
        let pubkey_bytes = signer::secp256k1_public_key(private_key)?;
        let auth_info_bytes = params.encode_auth_info(&pubkey_bytes);
        let sign_doc_bytes = params.encode_sign_doc(&params.body_bytes, &auth_info_bytes);

        let digest = Self::sign_doc_digest(chain, &sign_doc_bytes);
        let mut signature = Signer::sign_digest(SignatureScheme::Secp256k1, digest.to_vec(), private_key.to_vec())?;
        if signature.len() < 64 {
            return Err(SignerError::signing_error("secp256k1 signature too short"));
        }
        signature.truncate(64);

        let tx_raw = CosmosTxParams::encode_tx_raw(&params.body_bytes, &auth_info_bytes, &signature);
        let tx_base64 = STANDARD.encode(&tx_raw);
        Ok(serde_json::json!({
            "mode": "BROADCAST_MODE_SYNC",
            "tx_bytes": tx_base64,
        })
        .to_string())
    }
}
