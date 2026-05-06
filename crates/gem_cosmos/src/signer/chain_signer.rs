use gem_encoding::encode_base64;
use gem_hash::{keccak::keccak256, sha2::sha256};
use k256::{PublicKey, elliptic_curve::sec1::ToEncodedPoint};
use primitives::{ChainSigner, SignerError, SignerInput, chain_cosmos::CosmosChain};
use signer::{SignatureScheme, Signer};

use super::transaction::{self, COSMOS_SECP256K1_PUBKEY_TYPE, CosmosTxParams, INJECTIVE_ETHSECP256K1_PUBKEY_TYPE};
use crate::models::{Coin, CosmosMessage};

const BASE_FEE_GAS_UNITS: u64 = 200_000;
const GAS_BUFFER_NUMERATOR: u64 = 13;
const GAS_BUFFER_DENOMINATOR: u64 = 10;

#[derive(Default)]
pub struct CosmosChainSigner;

impl ChainSigner for CosmosChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let chain = Self::chain(input)?;
        Self::sign_send(chain, input, chain.denom().as_ref(), private_key)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let chain = Self::chain(input)?;
        let denom = input.input_type.get_asset().id.get_token_id()?;
        Self::sign_send(chain, input, denom, private_key)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap_data = input.input_type.get_swap_data().map_err(SignerError::invalid_input)?;
        let chain = Self::chain(input)?;

        let messages = CosmosMessage::parse_array(&swap_data.data.data)?;
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

        Ok(vec![Self::sign_messages(chain, input, messages, gas_limit, fee_amount, private_key)?])
    }

    fn sign_stake(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let chain = Self::chain(input)?;
        let messages = transaction::stake_messages(input, chain)?;
        let gas_limit = Self::gas_limit(input, messages.len())?;
        let fee_amount = input.fee.fee.to_string();

        Ok(vec![Self::sign_messages(chain, input, messages, gas_limit, fee_amount, private_key)?])
    }
}

impl CosmosChainSigner {
    fn chain(input: &SignerInput) -> Result<CosmosChain, SignerError> {
        CosmosChain::from_chain(input.input_type.get_asset().chain).ok_or_else(|| SignerError::invalid_input("unsupported cosmos chain"))
    }

    fn pubkey_type(chain: CosmosChain) -> &'static str {
        match chain {
            CosmosChain::Injective => INJECTIVE_ETHSECP256K1_PUBKEY_TYPE,
            _ => COSMOS_SECP256K1_PUBKEY_TYPE,
        }
    }

    fn public_key(chain: CosmosChain, private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
        let public_key = signer::secp256k1_public_key(private_key)?;
        match chain {
            CosmosChain::Injective => Self::uncompress_public_key(&public_key),
            CosmosChain::Cosmos | CosmosChain::Osmosis | CosmosChain::Celestia | CosmosChain::Thorchain | CosmosChain::Sei | CosmosChain::Noble => Ok(public_key),
        }
    }

    fn uncompress_public_key(public_key: &[u8]) -> Result<Vec<u8>, SignerError> {
        let public_key = PublicKey::from_sec1_bytes(public_key).map_err(|_| SignerError::invalid_input("invalid secp256k1 public key"))?;
        Ok(public_key.to_encoded_point(false).as_bytes().to_vec())
    }

    fn sign_doc_digest(chain: CosmosChain, sign_doc_bytes: &[u8]) -> [u8; 32] {
        match chain {
            CosmosChain::Injective => keccak256(sign_doc_bytes),
            _ => sha256(sign_doc_bytes),
        }
    }

    fn gas_limit(input: &SignerInput, message_count: usize) -> Result<u64, SignerError> {
        let message_count = u64::try_from(message_count).map_err(|_| SignerError::invalid_input("too many messages"))?;
        input
            .fee
            .gas_limit()?
            .checked_mul(message_count)
            .ok_or_else(|| SignerError::invalid_input("gas limit overflow"))
    }

    fn fee_coins(chain: CosmosChain, fee_amount: String) -> Vec<Coin> {
        match chain {
            CosmosChain::Thorchain => vec![],
            CosmosChain::Cosmos | CosmosChain::Osmosis | CosmosChain::Celestia | CosmosChain::Injective | CosmosChain::Sei | CosmosChain::Noble => vec![Coin {
                denom: chain.denom().as_ref().to_string(),
                amount: fee_amount,
            }],
        }
    }

    fn sign_send(chain: CosmosChain, input: &SignerInput, denom: &str, private_key: &[u8]) -> Result<String, SignerError> {
        let message = transaction::transfer_message(input, denom);
        let gas_limit = Self::gas_limit(input, 1)?;
        let fee_amount = input.fee.fee.to_string();
        Self::sign_messages(chain, input, vec![message], gas_limit, fee_amount, private_key)
    }

    fn sign_messages(chain: CosmosChain, input: &SignerInput, messages: Vec<CosmosMessage>, gas_limit: u64, fee_amount: String, private_key: &[u8]) -> Result<String, SignerError> {
        let account_number = input.metadata.get_account_number().map_err(SignerError::from_display)?;
        let sequence = input.metadata.get_sequence().map_err(SignerError::from_display)?;
        let chain_id = input.metadata.get_chain_id().map_err(SignerError::from_display)?;
        let encoded: Vec<Vec<u8>> = messages.iter().map(|m| m.encode_as_any(chain)).collect::<Result<Vec<_>, _>>()?;
        let body_bytes = CosmosTxParams::encode_tx_body(&encoded, input.memo.as_deref().unwrap_or(""));

        let params = CosmosTxParams {
            body_bytes,
            chain_id: &chain_id,
            account_number,
            sequence,
            fee_coins: Self::fee_coins(chain, fee_amount),
            gas_limit,
            pubkey_type: Self::pubkey_type(chain),
        };

        Self::encode_and_sign_tx(chain, &params, private_key)
    }

    pub fn encode_and_sign_tx(chain: CosmosChain, params: &CosmosTxParams, private_key: &[u8]) -> Result<String, SignerError> {
        let pubkey_bytes = Self::public_key(chain, private_key)?;
        let auth_info_bytes = params.encode_auth_info(&pubkey_bytes);
        let sign_doc_bytes = params.encode_sign_doc(&params.body_bytes, &auth_info_bytes);

        let digest = Self::sign_doc_digest(chain, &sign_doc_bytes);
        let mut signature = Signer::sign_digest(SignatureScheme::Secp256k1, digest.to_vec(), private_key.to_vec())?;
        if signature.len() < 64 {
            return Err(SignerError::signing_error("secp256k1 signature too short"));
        }
        signature.truncate(64);

        let tx_raw = CosmosTxParams::encode_tx_raw(&params.body_bytes, &auth_info_bytes, &signature);
        let tx_base64 = encode_base64(&tx_raw);
        Ok(serde_json::json!({
            "mode": "BROADCAST_MODE_SYNC",
            "tx_bytes": tx_base64,
        })
        .to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use num_bigint::BigInt;
    use primitives::{Asset, Chain, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata};
    use serde_json::Value;

    use super::*;

    #[test]
    fn test_sign_injective_transfer_matches_expected_tx_bytes() {
        // Source: https://github.com/trustwallet/wallet-core/blob/4.3.22/tests/chains/Cosmos/NativeInjective/SignerTests.cpp
        let private_key = hex::decode("9ee18daf8e463877aaf497282abc216852420101430482a28e246c179e2c5ef1").unwrap();
        let fee_amount = BigInt::from(100_000_000_000_000u64);
        let input = SignerInput::new(
            TransactionLoadInput {
                input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Injective)),
                sender_address: "inj13u6g7vqgw074mgmf2ze2cadzvkz9snlwcrtq8a".to_string(),
                destination_address: "inj1xmpkmxr4as00em23tc2zgmuyy2gr4h3wgcl6vd".to_string(),
                value: "10000000000".to_string(),
                gas_price: GasPriceType::regular(fee_amount.clone()),
                memo: None,
                is_max_value: false,
                metadata: TransactionLoadMetadata::Cosmos {
                    account_number: 17396,
                    sequence: 1,
                    chain_id: "injective-1".to_string(),
                },
            },
            TransactionFee::new_gas_price_type(GasPriceType::regular(fee_amount.clone()), fee_amount, BigInt::from(110_000u64), HashMap::new()),
        );

        let signed = CosmosChainSigner.sign_transfer(&input, &private_key).unwrap();
        let signed: Value = serde_json::from_str(&signed).unwrap();

        assert_eq!(signed["mode"], "BROADCAST_MODE_SYNC");
        assert_eq!(
            signed["tx_bytes"],
            "Co8BCowBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEmwKKmluajEzdTZnN3ZxZ3cwNzRtZ21mMnplMmNhZHp2a3o5c25sd2NydHE4YRIqaW5qMXhtcGtteHI0YXMwMGVtMjN0YzJ6Z211eXkyZ3I0aDN3Z2NsNnZkGhIKA2luahILMTAwMDAwMDAwMDASngEKfgp0Ci0vaW5qZWN0aXZlLmNyeXB0by52MWJldGExLmV0aHNlY3AyNTZrMS5QdWJLZXkSQwpBBFoMa4O4vZgn5QcnDK20mbfjqQlSRvaiITKB94PYd8mLJWdCdBsGOfMXdo/k9MJ2JmDCESKDp2hdgVUH3uMikXMSBAoCCAEYARIcChYKA2luahIPMTAwMDAwMDAwMDAwMDAwELDbBhpAx2vkplmzeK7n3puCFGPWhLd0l/ZC/CYkGl+stH+3S3hiCvIe7uwwMpUlNaSwvT8HwF1kNUp+Sx2m0Uo1x5xcFw=="
        );
    }
}
