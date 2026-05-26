use gem_encoding::encode_base64;
use gem_hash::{keccak::keccak256, sha2::sha256};
use k256::{PublicKey, elliptic_curve::sec1::ToEncodedPoint};
use primitives::{ChainSigner, SignerError, SignerInput, TransactionLoadMetadata, chain_cosmos::CosmosChain};
use signer::{SignatureScheme, Signer};

use super::transaction::{self, COSMOS_SECP256K1_PUBKEY_TYPE, CosmosTxParams, INJECTIVE_ETHSECP256K1_PUBKEY_TYPE};
use crate::models::{Coin, CosmosMessage};

const BASE_FEE_GAS_UNITS: u64 = 200_000;
const GAS_BUFFER_NUMERATOR: u64 = 13;
const GAS_BUFFER_DENOMINATOR: u64 = 10;
const DEFAULT_STAKE_MEMO: &str = "Stake via Gem Wallet";

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
        // Prefer the provider's gas limit (with buffer); fall back to the preloaded swap gas
        // limit scaled by message count when the provider omits it.
        let gas_limit = match swap_data.data.gas_limit.as_ref().and_then(|g| g.parse::<u64>().ok()).filter(|&g| g > 0) {
            Some(provider_gas) => (provider_gas as u128 * GAS_BUFFER_NUMERATOR as u128 / GAS_BUFFER_DENOMINATOR as u128) as u64,
            None => Self::gas_limit(input, messages.len())?,
        };
        let fee_amount = Self::scale_fee(gas_limit, input.fee.gas_price_u64()?);
        let memo = input.memo.as_deref().unwrap_or("");

        Ok(vec![Self::sign_messages(chain, &input.metadata, messages, gas_limit, fee_amount, memo, private_key)?])
    }

    fn sign_stake(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let chain = Self::chain(input)?;
        let messages = transaction::stake_messages(input, chain)?;
        let gas_limit = Self::gas_limit(input, messages.len())?;
        let fee_amount = input.fee.fee.to_string();
        let memo = input.memo.as_deref().filter(|m| !m.is_empty()).unwrap_or(DEFAULT_STAKE_MEMO);

        Ok(vec![Self::sign_messages(chain, &input.metadata, messages, gas_limit, fee_amount, memo, private_key)?])
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

    fn scale_fee(gas_limit: u64, base_fee: u64) -> String {
        ((gas_limit as u128 * base_fee as u128 / BASE_FEE_GAS_UNITS as u128) as u64).to_string()
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
        let memo = input.memo.as_deref().unwrap_or("");
        Self::sign_messages(chain, &input.metadata, vec![message], gas_limit, fee_amount, memo, private_key)
    }

    fn sign_messages(
        chain: CosmosChain,
        metadata: &TransactionLoadMetadata,
        messages: Vec<CosmosMessage>,
        gas_limit: u64,
        fee_amount: String,
        memo: &str,
        private_key: &[u8],
    ) -> Result<String, SignerError> {
        let account_number = metadata.get_account_number()?;
        let sequence = metadata.get_sequence()?;
        let chain_id = metadata.get_chain_id()?;
        let encoded: Vec<Vec<u8>> = messages.iter().map(|m| m.encode_as_any(chain)).collect::<Result<Vec<_>, _>>()?;
        let body_bytes = CosmosTxParams::encode_tx_body(&encoded, memo);

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
        let mut signature = Signer::sign_digest(SignatureScheme::Secp256k1, &digest, private_key)?;
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
    use primitives::{
        Asset, Chain, Delegation, DelegationValidator, GasPriceType, RedelegateData, StakeType, SwapProvider, TransactionFee, TransactionInputType, TransactionLoadInput,
        TransactionLoadMetadata, swap::SwapData,
    };
    use serde_json::Value;

    use super::*;

    // Derived from "seminar cruel gown pause law tortoise step stairs size amused pond weapon" via m/44'/118'/0'/0/0.
    const OSMO_PRIVATE_KEY_HEX: &str = "325f5eba4c6466ca5a88638c74db5b396edb624efced0924a10aeb897525923c";
    const OSMO_VALIDATOR: &str = "osmovaloper1pxphtfhqnx9ny27d53z4052e3r76e7qq495ehm";
    const OSMO_VALIDATOR_DST: &str = "osmovaloper1z0sh4s80u99l6y9d3vfy582p8jejeeu6tcucs2";

    fn signed_tx_bytes(signed: &str) -> String {
        let value: Value = serde_json::from_str(signed).unwrap();
        assert_eq!(value["mode"], "BROADCAST_MODE_SYNC");
        value["tx_bytes"].as_str().unwrap().to_string()
    }

    #[test]
    fn test_sign_thorchain_transfer() {
        // Source: https://github.com/trustwallet/wallet-core/blob/4.3.22/swift/Tests/Blockchains/THORChainTests.swift
        let private_key = hex::decode("7105512f0c020a1dd759e14b865ec0125f59ac31e34d7a2807a228ed50cb343e").unwrap();
        let fee_amount = BigInt::from(200u64);
        let input = SignerInput::new(
            TransactionLoadInput {
                input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Thorchain)),
                sender_address: "thor1z53wwe7md6cewz9sqwqzn0aavpaun0gw0exn2r".to_string(),
                destination_address: "thor1e2ryt8asq4gu0h6z2sx9u7rfrykgxwkmr9upxn".to_string(),
                value: "38000000".to_string(),
                gas_price: GasPriceType::regular(fee_amount.clone()),
                memo: None,
                is_max_value: false,
                metadata: TransactionLoadMetadata::Cosmos {
                    account_number: 593,
                    sequence: 21,
                    chain_id: "thorchain-mainnet-v1".to_string(),
                },
            },
            TransactionFee::new_gas_price_type(GasPriceType::regular(fee_amount.clone()), fee_amount, BigInt::from(2_500_000u64), HashMap::new()),
        );

        let signed = CosmosChainSigner.sign_transfer(&input, &private_key).unwrap();
        assert_eq!(
            signed_tx_bytes(&signed),
            "ClIKUAoOL3R5cGVzLk1zZ1NlbmQSPgoUFSLnZ9tusZcIsAOAKb+9YHvJvQ4SFMqGRZ+wBVHH30JUDF54aRksgzrbGhAKBHJ1bmUSCDM4MDAwMDAwElkKUApGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQPtmX45bPQpL1/OWkK7pBWZzNXZbjExVKfJ6nBJ3jF8dxIECgIIARgVEgUQoMuYARpAj4gtkfIP83fI0HHaCa95deqwo280CoLDVHJ6BkSGADxQaYoBWJW/NwaMU05d34AkUgesUjJHk1238cG9Am+J0g=="
        );
    }

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
        assert_eq!(
            signed_tx_bytes(&signed),
            "Co8BCowBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEmwKKmluajEzdTZnN3ZxZ3cwNzRtZ21mMnplMmNhZHp2a3o5c25sd2NydHE4YRIqaW5qMXhtcGtteHI0YXMwMGVtMjN0YzJ6Z211eXkyZ3I0aDN3Z2NsNnZkGhIKA2luahILMTAwMDAwMDAwMDASngEKfgp0Ci0vaW5qZWN0aXZlLmNyeXB0by52MWJldGExLmV0aHNlY3AyNTZrMS5QdWJLZXkSQwpBBFoMa4O4vZgn5QcnDK20mbfjqQlSRvaiITKB94PYd8mLJWdCdBsGOfMXdo/k9MJ2JmDCESKDp2hdgVUH3uMikXMSBAoCCAEYARIcChYKA2luahIPMTAwMDAwMDAwMDAwMDAwELDbBhpAx2vkplmzeK7n3puCFGPWhLd0l/ZC/CYkGl+stH+3S3hiCvIe7uwwMpUlNaSwvT8HwF1kNUp+Sx2m0Uo1x5xcFw=="
        );
    }

    #[test]
    fn test_sign_osmosis_messages() {
        let private_key = hex::decode(OSMO_PRIVATE_KEY_HEX).unwrap();
        let signer = CosmosChainSigner;

        let transfer = SignerInput::mock_osmosis(
            TransactionInputType::Transfer(Asset::from_chain(Chain::Osmosis)),
            "osmo1rcjvzz8wzktqfz8qjf0l9q45kzxvd0z0n7l5cf",
        );
        assert_eq!(
            signed_tx_bytes(&signer.sign_transfer(&transfer, &private_key).unwrap()),
            "CooBCocBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEmcKK29zbW8xa2dsZW11bXU4bW42NThqNmc0ejlqem4zemVmMnFkeXl2a2x3YTMSK29zbW8xcmNqdnp6OHd6a3RxZno4cWpmMGw5cTQ1a3p4dmQwejBuN2w1Y2YaCwoFdW9zbW8SAjEwEmgKUApGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQMslcYn7DhPe5b/8lM3FnPXhGBj5SdC15+XI1hZ1gYbBBIECgIIARgKEhQKDgoFdW9zbW8SBTEwMDAwEMCaDBpAVJkDxaS5ZaghmJ6ZtpC9yim7JA8duO8MwOODdJeHEHssH3PQN+4Yl+SVyLtNEW6+IDUKfkG1dfIYOvpRiFlOyg=="
        );

        let stake = SignerInput::mock_osmosis(
            TransactionInputType::Stake(Asset::from_chain(Chain::Osmosis), StakeType::Stake(DelegationValidator::mock_osmosis(OSMO_VALIDATOR))),
            "",
        );
        let signed = signer.sign_stake(&stake, &private_key).unwrap();
        assert_eq!(signed.len(), 1);
        assert_eq!(
            signed_tx_bytes(&signed[0]),
            "Cq4BCpUBCiMvY29zbW9zLnN0YWtpbmcudjFiZXRhMS5Nc2dEZWxlZ2F0ZRJuCitvc21vMWtnbGVtdW11OG1uNjU4ajZnNHo5anpuM3plZjJxZHl5dmtsd2EzEjJvc21vdmFsb3BlcjFweHBodGZocW54OW55MjdkNTN6NDA1MmUzcjc2ZTdxcTQ5NWVobRoLCgV1b3NtbxICMTASFFN0YWtlIHZpYSBHZW0gV2FsbGV0EmgKUApGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQMslcYn7DhPe5b/8lM3FnPXhGBj5SdC15+XI1hZ1gYbBBIECgIIARgKEhQKDgoFdW9zbW8SBTEwMDAwEMCaDBpAxh9uwNZvql2fODCEAp4XhucO1cxXYrz2oMEkat+wvJEP1VDlai4ZnLz+n9mRbgjF143EfsaonoEh36uQKYOWuQ=="
        );

        let undelegate = SignerInput::mock_osmosis(
            TransactionInputType::Stake(Asset::from_chain(Chain::Osmosis), StakeType::Unstake(Delegation::mock_osmosis(OSMO_VALIDATOR))),
            "",
        );
        // Auto-claims pending rewards before unstake.
        let signed = signer.sign_stake(&undelegate, &private_key).unwrap();
        assert_eq!(
            signed_tx_bytes(&signed[0]),
            "Cs8CCpwBCjcvY29zbW9zLmRpc3RyaWJ1dGlvbi52MWJldGExLk1zZ1dpdGhkcmF3RGVsZWdhdG9yUmV3YXJkEmEKK29zbW8xa2dsZW11bXU4bW42NThqNmc0ejlqem4zemVmMnFkeXl2a2x3YTMSMm9zbW92YWxvcGVyMXB4cGh0Zmhxbng5bnkyN2Q1M3o0MDUyZTNyNzZlN3FxNDk1ZWhtCpcBCiUvY29zbW9zLnN0YWtpbmcudjFiZXRhMS5Nc2dVbmRlbGVnYXRlEm4KK29zbW8xa2dsZW11bXU4bW42NThqNmc0ejlqem4zemVmMnFkeXl2a2x3YTMSMm9zbW92YWxvcGVyMXB4cGh0Zmhxbng5bnkyN2Q1M3o0MDUyZTNyNzZlN3FxNDk1ZWhtGgsKBXVvc21vEgIxMBIUU3Rha2UgdmlhIEdlbSBXYWxsZXQSaApQCkYKHy9jb3Ntb3MuY3J5cHRvLnNlY3AyNTZrMS5QdWJLZXkSIwohAyyVxifsOE97lv/yUzcWc9eEYGPlJ0LXn5cjWFnWBhsEEgQKAggBGAoSFAoOCgV1b3NtbxIFMTAwMDAQgLUYGkCA133uwfd5FIq0KwZtG+gduTmeUmvgZ4dFmxLb23a37zBIOAx26XVJQ9PNDD2tFlODaVLjnN+a2saa4KOXz/wG"
        );

        let redelegate = SignerInput::mock_osmosis(
            TransactionInputType::Stake(
                Asset::from_chain(Chain::Osmosis),
                StakeType::Redelegate(RedelegateData {
                    delegation: Delegation::mock_osmosis(OSMO_VALIDATOR),
                    to_validator: DelegationValidator::mock_osmosis(OSMO_VALIDATOR_DST),
                }),
            ),
            "",
        );
        let signed = signer.sign_stake(&redelegate, &private_key).unwrap();
        assert_eq!(
            signed_tx_bytes(&signed[0]),
            "CokDCpwBCjcvY29zbW9zLmRpc3RyaWJ1dGlvbi52MWJldGExLk1zZ1dpdGhkcmF3RGVsZWdhdG9yUmV3YXJkEmEKK29zbW8xa2dsZW11bXU4bW42NThqNmc0ejlqem4zemVmMnFkeXl2a2x3YTMSMm9zbW92YWxvcGVyMXB4cGh0Zmhxbng5bnkyN2Q1M3o0MDUyZTNyNzZlN3FxNDk1ZWhtCtEBCiovY29zbW9zLnN0YWtpbmcudjFiZXRhMS5Nc2dCZWdpblJlZGVsZWdhdGUSogEKK29zbW8xa2dsZW11bXU4bW42NThqNmc0ejlqem4zemVmMnFkeXl2a2x3YTMSMm9zbW92YWxvcGVyMXB4cGh0Zmhxbng5bnkyN2Q1M3o0MDUyZTNyNzZlN3FxNDk1ZWhtGjJvc21vdmFsb3BlcjF6MHNoNHM4MHU5OWw2eTlkM3ZmeTU4MnA4amVqZWV1NnRjdWNzMiILCgV1b3NtbxICMTASFFN0YWtlIHZpYSBHZW0gV2FsbGV0EmgKUApGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQMslcYn7DhPe5b/8lM3FnPXhGBj5SdC15+XI1hZ1gYbBBIECgIIARgKEhQKDgoFdW9zbW8SBTEwMDAwEIC1GBpAPgfCbDv4AFbBsGokEl26JCKuyt7R0PN2/jHsBnva4dQqd7kxKIIwGq2yDmwserV4/2B1I51W2JHL0m8/ZOYT7g=="
        );

        let rewards = SignerInput::mock_osmosis(
            TransactionInputType::Stake(
                Asset::from_chain(Chain::Osmosis),
                StakeType::Rewards(vec![DelegationValidator::mock_osmosis(OSMO_VALIDATOR), DelegationValidator::mock_osmosis(OSMO_VALIDATOR)]),
            ),
            "",
        );
        let signed = signer.sign_stake(&rewards, &private_key).unwrap();
        assert_eq!(
            signed_tx_bytes(&signed[0]),
            "CtQCCpwBCjcvY29zbW9zLmRpc3RyaWJ1dGlvbi52MWJldGExLk1zZ1dpdGhkcmF3RGVsZWdhdG9yUmV3YXJkEmEKK29zbW8xa2dsZW11bXU4bW42NThqNmc0ejlqem4zemVmMnFkeXl2a2x3YTMSMm9zbW92YWxvcGVyMXB4cGh0Zmhxbng5bnkyN2Q1M3o0MDUyZTNyNzZlN3FxNDk1ZWhtCpwBCjcvY29zbW9zLmRpc3RyaWJ1dGlvbi52MWJldGExLk1zZ1dpdGhkcmF3RGVsZWdhdG9yUmV3YXJkEmEKK29zbW8xa2dsZW11bXU4bW42NThqNmc0ejlqem4zemVmMnFkeXl2a2x3YTMSMm9zbW92YWxvcGVyMXB4cGh0Zmhxbng5bnkyN2Q1M3o0MDUyZTNyNzZlN3FxNDk1ZWhtEhRTdGFrZSB2aWEgR2VtIFdhbGxldBJoClAKRgofL2Nvc21vcy5jcnlwdG8uc2VjcDI1NmsxLlB1YktleRIjCiEDLJXGJ+w4T3uW//JTNxZz14RgY+UnQteflyNYWdYGGwQSBAoCCAEYChIUCg4KBXVvc21vEgUxMDAwMBCAtRgaQH/U90uCH0zx9AdY+ALIHM5aZ1crBSwYzeZZejb5rWjEMVXRScjOfvng33XFnFHdI4Epp9ykNNtQVUw9BJnZshU="
        );
    }

    // sign_swap should accept a missing/zero provider gas limit and fall back to the preloaded value.
    #[test]
    fn test_sign_swap_falls_back_when_provider_gas_missing() {
        let private_key = hex::decode(OSMO_PRIVATE_KEY_HEX).unwrap();
        let msg_send = r#"[{"typeUrl":"/cosmos.bank.v1beta1.MsgSend","value":{"from_address":"osmo1kglemumu8mn658j6g4z9jzn3zef2qdyyvklwa3","to_address":"osmo1rcjvzz8wzktqfz8qjf0l9q45kzxvd0z0n7l5cf","amount":[{"denom":"uosmo","amount":"10"}]}}]"#;

        for gas_limit in [None, Some("0"), Some("")] {
            let swap_data = SwapData::mock_with_provider_data(SwapProvider::Squid, msg_send, gas_limit);
            let input = SignerInput::mock_osmosis(
                TransactionInputType::Swap(Asset::from_chain(Chain::Osmosis), Asset::from_chain(Chain::Osmosis), swap_data),
                "",
            );
            let signed = CosmosChainSigner.sign_swap(&input, &private_key).expect("swap should sign");
            assert_eq!(signed.len(), 1);
            // Identical bytes to the OSMO native transfer (1 msg, 200000 gas, 10000 uosmo fee).
            assert_eq!(
                signed_tx_bytes(&signed[0]),
                "CooBCocBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEmcKK29zbW8xa2dsZW11bXU4bW42NThqNmc0ejlqem4zemVmMnFkeXl2a2x3YTMSK29zbW8xcmNqdnp6OHd6a3RxZno4cWpmMGw5cTQ1a3p4dmQwejBuN2w1Y2YaCwoFdW9zbW8SAjEwEmgKUApGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQMslcYn7DhPe5b/8lM3FnPXhGBj5SdC15+XI1hZ1gYbBBIECgIIARgKEhQKDgoFdW9zbW8SBTEwMDAwEMCaDBpAVJkDxaS5ZaghmJ6ZtpC9yim7JA8duO8MwOODdJeHEHssH3PQN+4Yl+SVyLtNEW6+IDUKfkG1dfIYOvpRiFlOyg=="
            );
        }
    }
}
