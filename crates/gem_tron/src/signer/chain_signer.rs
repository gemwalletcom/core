use primitives::{ChainSigner, SignerError, SignerInput, hex::encode_with_0x};
use signer::Signer;

use super::{message::tron_hash_message, transaction};

#[derive(Default)]
pub struct TronChainSigner;

impl ChainSigner for TronChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        transaction::sign_transfer(input, private_key)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        transaction::sign_token_transfer(input, private_key)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        transaction::sign_swap(input, private_key)
    }

    fn sign_stake(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        transaction::sign_stake(input, private_key)
    }

    fn sign_data(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        transaction::sign_data(input, private_key)
    }

    fn sign_message(&self, message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
        let digest = tron_hash_message(message);
        let signature = Signer::sign_ethereum_digest(&digest, private_key)?;
        Ok(encode_with_0x(&signature))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use gem_hash::sha2::sha256;
    use num_bigint::BigInt;
    use primitives::{
        Asset, AssetId, AssetType, Chain, ChainSigner, Delegation, DelegationValidator, GasPriceType, Resource, SignerInput, StakeType, TransactionFee, TransactionInputType,
        TransactionLoadInput, TransactionLoadMetadata, TransferDataExtra, TransferDataOutputAction, TransferDataOutputType, TronStakeData, TronUnfreeze, TronVote,
        WalletConnectionSessionAppMetadata, decode_hex,
    };
    use serde_json::{Value, json};

    use super::TronChainSigner;

    const PRIVATE_KEY: &str = "2d8f68944bdbfbc0769542fba8fc2d2a3de67393334471624364c7006da2aa54";
    const NILE_PRIVATE_KEY: &str = "75065f100e38d3f3b4c5c4235834ba8216de62272a4f03532c44b31a5734360a";
    const SENDER: &str = "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC";
    const NILE_SENDER: &str = "TWWb9EjUWai17YEVB7FR8hreupYJKG9sMR";
    const RECIPIENT: &str = "THTR75o8xXAgCTQqpiot2AFRAjvW1tSbVV";

    fn private_key() -> Vec<u8> {
        hex::decode(PRIVATE_KEY).unwrap()
    }

    fn nile_private_key() -> Vec<u8> {
        hex::decode(NILE_PRIVATE_KEY).unwrap()
    }

    fn metadata(stake_data: TronStakeData) -> TransactionLoadMetadata {
        TransactionLoadMetadata::Tron {
            block_number: 3_111_739,
            block_version: 3,
            block_timestamp: 1_539_295_479_000,
            transaction_tree_root: "64288c2db0641316762a99dbb02ef7c90f968b60f9f2e410835980614332f86d".to_string(),
            parent_hash: "00000000002f7b3af4f5f8b9e23a30c530f719f165b742e7358536b280eead2d".to_string(),
            witness_address: "415863f6091b8e71766da808b1dd3159790f61de7d".to_string(),
            stake_data,
        }
    }

    fn nile_metadata(stake_data: TronStakeData) -> TransactionLoadMetadata {
        TransactionLoadMetadata::Tron {
            block_number: 34_395_330,
            block_version: 26,
            block_timestamp: 1_676_983_541_337,
            transaction_tree_root: "9b54db7f84bd19bbad9ff1fccef894c1aade6879450e9e9e2accec751eaa1f52".to_string(),
            parent_hash: "00000000020cd4c13a67497a3a433a3105bc5a73a041ee3da98407d5a2a2bf1b".to_string(),
            witness_address: "4150d3765e4e670727ebac9d5b598f74b75a3d54a7".to_string(),
            stake_data,
        }
    }

    fn fee(fee: u64, gas_limit: u64) -> TransactionFee {
        TransactionFee::new_gas_price_type(GasPriceType::regular(0), BigInt::from(fee), BigInt::from(gas_limit), HashMap::new())
    }

    fn signer_input(
        input_type: TransactionInputType,
        sender: &str,
        destination: &str,
        value: &str,
        transaction_fee: TransactionFee,
        memo: Option<&str>,
        metadata: TransactionLoadMetadata,
    ) -> SignerInput {
        SignerInput::new(
            TransactionLoadInput {
                input_type,
                sender_address: sender.to_string(),
                destination_address: destination.to_string(),
                value: value.to_string(),
                gas_price: GasPriceType::regular(0),
                memo: memo.map(str::to_string),
                is_max_value: false,
                metadata,
            },
            transaction_fee,
        )
    }

    fn native_input(value: &str, transaction_fee: TransactionFee, memo: Option<&str>) -> SignerInput {
        signer_input(
            TransactionInputType::Transfer(Asset::from_chain(Chain::Tron)),
            SENDER,
            RECIPIENT,
            value,
            transaction_fee,
            memo,
            metadata(TronStakeData::Votes(vec![])),
        )
    }

    fn trc20_asset(contract: &str) -> Asset {
        Asset::new(AssetId::from_token(Chain::Tron, contract), "Token".to_string(), "TOKEN".to_string(), 6, AssetType::TRC20)
    }

    fn signed_json(output: String) -> Value {
        let value: Value = serde_json::from_str(&output).unwrap();
        assert_hash_matches(&value);
        value
    }

    fn assert_hash_matches(output: &Value) {
        let raw_data_hex = output["raw_data_hex"].as_str().unwrap();
        let raw_data = decode_hex(raw_data_hex).unwrap();
        assert_eq!(hex::encode(sha256(&raw_data)), output["txID"].as_str().unwrap());
    }

    fn signature(output: &Value) -> &str {
        output["signature"][0].as_str().unwrap()
    }

    fn assert_raw_recovery_id(output: &Value) {
        let signature = signature(output);
        assert_eq!(signature.len(), 130);
        assert!(signature.ends_with("00") || signature.ends_with("01"));
    }

    fn validator(id: &str) -> DelegationValidator {
        DelegationValidator::stake(Chain::Tron, id.to_string(), String::new(), true, 0.0, 0.0)
    }

    // Source vector:
    // https://github.com/trustwallet/wallet-core/blob/master/tests/chains/Tron/SignerTests.cpp
    #[test]
    fn sign_transfer_matches_wallet_core() {
        let input = native_input("2000000", TransactionFee::default(), None);
        let output = signed_json(TronChainSigner.sign_transfer(&input, &private_key()).unwrap());

        assert_eq!(output["txID"], "dc6f6d9325ee44ab3c00528472be16e1572ab076aa161ccd12515029869d0451");
        assert_eq!(
            signature(&output),
            "ede769f6df28aefe6a846be169958c155e23e7e5c9621d2e8dce1719b4d952b63e8a8bf9f00e41204ac1bf69b1a663dacdf764367e48e4a5afcd6b055a747fb200"
        );
    }

    #[test]
    fn sign_transfer_includes_mobile_fee_limit() {
        let input = native_input("100", fee(10, 0), None);
        let output = signed_json(TronChainSigner.sign_transfer(&input, &private_key()).unwrap());

        assert_eq!(output["raw_data"]["fee_limit"], 10);
    }

    // Source vector:
    // https://github.com/trustwallet/wallet-core/blob/master/tests/chains/Tron/SignerTests.cpp
    #[test]
    fn sign_transfer_with_memo_matches_wallet_core() {
        let input = signer_input(
            TransactionInputType::Transfer(Asset::from_chain(Chain::Tron)),
            "TFnYQCt892UNjn67pjAULTSTkB7YvqsnPp",
            "TBUCzgc29vykkvFaEG2mgRtxKvaKe6skwX",
            "100000",
            TransactionFee::default(),
            Some("Test memo"),
            TransactionLoadMetadata::Tron {
                block_number: 66_725_852,
                block_version: 30,
                block_timestamp: 1_730_827_017_000,
                transaction_tree_root: "a94f115089893f37336baf32dbf6cb7d06adc13cf6bf046d9bc22748bd72e7a6".to_string(),
                parent_hash: "0000000003fa27db7d67f93920f64733532412ab6a71eb4089dc48c8ff5e182c".to_string(),
                witness_address: "4167e39013be3cdd3814bed152d7439fb5b6791409".to_string(),
                stake_data: TronStakeData::Votes(vec![]),
            },
        );
        let private_key = hex::decode("7c2108a30f6f69f8dce72a7df897eabadfe9810eee6976b43bdf8c0b0d35337d").unwrap();
        let output = signed_json(TronChainSigner.sign_transfer(&input, &private_key).unwrap());

        assert_eq!(output["txID"], "20321755964d6ec5bcfc9ebfb15faeb043787ae599fff44442962e12e1c357f1");
        assert_eq!(
            signature(&output),
            "6fcee79c61f660ec689299f77924f32b5020b4c41593056052ef07d640cc799325103fab130c8691e8a224c96cd0704a698ac356ff789a543c284605668bf38000"
        );
        assert_eq!(output["raw_data"]["data"], "54657374206d656d6f");
    }

    #[test]
    fn sign_token_transfer_builds_trc20_trigger_contract() {
        let input = signer_input(
            TransactionInputType::Transfer(trc20_asset(RECIPIENT)),
            SENDER,
            "TW1dU4L3eNm7Lw8WvieLKEHpXWAussRG9Z",
            "1000",
            fee(0, 10),
            None,
            metadata(TronStakeData::Votes(vec![])),
        );
        let output = signed_json(TronChainSigner.sign_token_transfer(&input, &private_key()).unwrap());
        let contract = &output["raw_data"]["contract"][0];
        let value = &contract["parameter"]["value"];

        assert_eq!(contract["type"], "TriggerSmartContract");
        assert_eq!(value["contract_address"], "41521ea197907927725ef36d70f25f850d1659c7c7");
        assert_eq!(value["owner_address"], "415cd0fb0ab3ce40f3051414c604b27756e69e43db");
        assert_eq!(
            value["data"],
            "a9059cbb000000000000000000000041dbd7c53729b3310e1843083000fa84abad99696100000000000000000000000000000000000000000000000000000000000003e8"
        );
        assert_eq!(output["raw_data"]["fee_limit"], 10);
        assert_raw_recovery_id(&output);
    }

    #[test]
    fn sign_token_transfer_uses_gas_limit_as_fee_limit() {
        let input = signer_input(
            TransactionInputType::Transfer(trc20_asset(RECIPIENT)),
            SENDER,
            "TW1dU4L3eNm7Lw8WvieLKEHpXWAussRG9Z",
            "1000",
            fee(10, 20),
            None,
            metadata(TronStakeData::Votes(vec![])),
        );
        let output = signed_json(TronChainSigner.sign_token_transfer(&input, &private_key()).unwrap());

        assert_eq!(output["raw_data"]["fee_limit"], 20);
    }

    #[test]
    fn sign_transfer_based_swap_uses_swap_destination() {
        let input = signer_input(
            TransactionInputType::Swap(
                Asset::from_chain(Chain::Tron),
                Asset::from_chain(Chain::Tron),
                primitives::swap::SwapData {
                    quote: primitives::swap::SwapQuote {
                        from_address: SENDER.to_string(),
                        from_value: "2000000".to_string(),
                        to_address: "TW1dU4L3eNm7Lw8WvieLKEHpXWAussRG9Z".to_string(),
                        to_value: "1".to_string(),
                        provider_data: primitives::swap::SwapProviderData {
                            provider: primitives::SwapProvider::Okx,
                            name: "OKX".to_string(),
                            protocol_name: "okx".to_string(),
                        },
                        slippage_bps: 50,
                        eta_in_seconds: None,
                        use_max_amount: None,
                    },
                    data: primitives::swap::SwapQuoteData {
                        to: "TW1dU4L3eNm7Lw8WvieLKEHpXWAussRG9Z".to_string(),
                        data_type: primitives::swap::SwapQuoteDataType::Transfer,
                        value: "2000000".to_string(),
                        data: String::new(),
                        memo: None,
                        approval: None,
                        gas_limit: None,
                    },
                },
            ),
            SENDER,
            RECIPIENT,
            "2000000",
            TransactionFee::default(),
            None,
            metadata(TronStakeData::Votes(vec![])),
        );
        let output = signed_json(TronChainSigner.sign_swap(&input, &private_key()).unwrap().remove(0));

        assert_eq!(
            output["raw_data"]["contract"][0]["parameter"]["value"]["to_address"],
            "41dbd7c53729b3310e1843083000fa84abad996961"
        );
    }

    // Source vector:
    // https://github.com/trustwallet/wallet-core/blob/master/tests/chains/Tron/SignerTests.cpp
    #[test]
    fn sign_vote_witness_matches_wallet_core() {
        let input = signer_input(
            TransactionInputType::Stake(Asset::from_chain(Chain::Tron), StakeType::Stake(validator(RECIPIENT))),
            SENDER,
            RECIPIENT,
            "0",
            TransactionFee::default(),
            None,
            metadata(TronStakeData::Votes(vec![TronVote {
                validator: RECIPIENT.to_string(),
                count: 3,
            }])),
        );
        let output = signed_json(TronChainSigner.sign_stake(&input, &private_key()).unwrap().remove(0));

        assert_eq!(output["txID"], "3f923e9dd9571a66624fafeda27baa3e00aba1709d3fdc5c97c77b81fda18c1f");
        assert_eq!(
            signature(&output),
            "79ec1073ae1319ef9303a2f5a515876cfd67f8f0e155bdbde1115d391c05358a3c32f148bfafacf07e1619aaed728d9ffbc2c7e4a5046003c7b74feb86fc68e400"
        );
    }

    #[test]
    fn sign_vote_witness_keeps_multiple_votes() {
        let input = signer_input(
            TransactionInputType::Stake(Asset::from_chain(Chain::Tron), StakeType::Stake(validator(RECIPIENT))),
            SENDER,
            RECIPIENT,
            "0",
            fee(10, 0),
            None,
            metadata(TronStakeData::Votes(vec![
                TronVote {
                    validator: "TLyqzVGLV1srkB7dToTAEqgDSfPtXRJZYH".to_string(),
                    count: 1,
                },
                TronVote {
                    validator: "TCEo1hMAdaJrQmvnGTCcGT2LqrGU4N7Jqf".to_string(),
                    count: 2,
                },
            ])),
        );
        let output = signed_json(TronChainSigner.sign_stake(&input, &private_key()).unwrap().remove(0));
        let votes = output["raw_data"]["contract"][0]["parameter"]["value"]["votes"].as_array().unwrap();

        assert_eq!(votes.len(), 2);
        assert_eq!(votes[0]["vote_count"], 1);
        assert_eq!(votes[1]["vote_count"], 2);
        assert_eq!(output["raw_data"]["fee_limit"], 10);
        assert_raw_recovery_id(&output);
    }

    #[test]
    fn sign_unstake_votes_builds_vote_witness_contract() {
        let input = signer_input(
            TransactionInputType::Stake(Asset::from_chain(Chain::Tron), StakeType::Unstake(Delegation::mock_tron(RECIPIENT))),
            SENDER,
            RECIPIENT,
            "0",
            TransactionFee::default(),
            None,
            metadata(TronStakeData::Votes(vec![TronVote {
                validator: RECIPIENT.to_string(),
                count: 2,
            }])),
        );
        let output = signed_json(TronChainSigner.sign_stake(&input, &private_key()).unwrap().remove(0));
        let contract = &output["raw_data"]["contract"][0];
        let value = &contract["parameter"]["value"];
        let votes = value["votes"].as_array().unwrap();

        assert_eq!(contract["type"], "VoteWitnessContract");
        assert_eq!(value["owner_address"], "415cd0fb0ab3ce40f3051414c604b27756e69e43db");
        assert_eq!(votes.len(), 1);
        assert_eq!(votes[0]["vote_count"], 2);
        assert_eq!(value["support"], true);
    }

    #[test]
    fn sign_freeze_v2_builds_energy_contract() {
        let input = signer_input(
            TransactionInputType::Stake(Asset::from_chain(Chain::Tron), StakeType::Freeze(Resource::Energy)),
            NILE_SENDER,
            RECIPIENT,
            "10000000",
            TransactionFee::default(),
            None,
            nile_metadata(TronStakeData::Votes(vec![])),
        );
        let output = signed_json(TronChainSigner.sign_stake(&input, &nile_private_key()).unwrap().remove(0));
        let value = &output["raw_data"]["contract"][0]["parameter"]["value"];

        assert_eq!(output["raw_data"]["contract"][0]["type"], "FreezeBalanceV2Contract");
        assert_eq!(value["frozen_balance"], 10_000_000);
        assert_eq!(value["resource"], "ENERGY");
        assert_raw_recovery_id(&output);
    }

    #[test]
    fn sign_unfreeze_v2_builds_contract() {
        let input = signer_input(
            TransactionInputType::Stake(Asset::from_chain(Chain::Tron), StakeType::Unfreeze(Resource::Energy)),
            NILE_SENDER,
            RECIPIENT,
            "510000000",
            TransactionFee::default(),
            None,
            nile_metadata(TronStakeData::Votes(vec![])),
        );
        let output = signed_json(TronChainSigner.sign_stake(&input, &nile_private_key()).unwrap().remove(0));
        let value = &output["raw_data"]["contract"][0]["parameter"]["value"];

        assert_eq!(output["raw_data"]["contract"][0]["type"], "UnfreezeBalanceV2Contract");
        assert_eq!(value["unfreeze_balance"], 510_000_000);
        assert_eq!(value["resource"], "ENERGY");
    }

    // Source vector:
    // https://github.com/trustwallet/wallet-core/blob/master/tests/chains/Tron/SignerTests.cpp
    #[test]
    fn sign_withdraw_rewards_matches_wallet_core() {
        let input = signer_input(
            TransactionInputType::Stake(Asset::from_chain(Chain::Tron), StakeType::Rewards(vec![])),
            SENDER,
            RECIPIENT,
            "0",
            TransactionFee::default(),
            None,
            metadata(TronStakeData::Votes(vec![])),
        );
        let output = signed_json(TronChainSigner.sign_stake(&input, &private_key()).unwrap().remove(0));

        assert_eq!(output["txID"], "69aaa954dcd61f28a6a73e979addece6e36541522e5b3374b18b4ef9bc3de4cb");
        assert_eq!(
            signature(&output),
            "cb7d23a5eb23284a25ba6deaa231de0f18d8d103592e3312bff101a4219a3e02167eca24b3f4ce78b34f0c1842b6f7fb8d813f530c4c54342cdedef9f8e1f85100"
        );
    }

    #[test]
    fn sign_withdraw_expire_unfreeze_builds_contract() {
        let input = signer_input(
            TransactionInputType::Stake(Asset::from_chain(Chain::Tron), StakeType::Withdraw(Delegation::mock_tron(RECIPIENT))),
            NILE_SENDER,
            RECIPIENT,
            "0",
            TransactionFee::default(),
            None,
            nile_metadata(TronStakeData::Votes(vec![])),
        );
        let output = signed_json(TronChainSigner.sign_stake(&input, &nile_private_key()).unwrap().remove(0));

        assert_eq!(output["raw_data"]["contract"][0]["type"], "WithdrawExpireUnfreezeContract");
        assert_eq!(
            output["raw_data"]["contract"][0]["parameter"]["value"]["owner_address"],
            "41e151e4937bca41df55a67697724d9a64efcffdd5"
        );
        assert_raw_recovery_id(&output);
    }

    #[test]
    fn sign_unstake_unfreeze_outputs_one_transaction_per_unfreeze() {
        let input = signer_input(
            TransactionInputType::Stake(Asset::from_chain(Chain::Tron), StakeType::Unstake(Delegation::mock_tron(RECIPIENT))),
            SENDER,
            RECIPIENT,
            "0",
            TransactionFee::default(),
            None,
            metadata(TronStakeData::Unfreeze(vec![
                TronUnfreeze {
                    resource: Resource::Bandwidth,
                    amount: 1,
                },
                TronUnfreeze {
                    resource: Resource::Energy,
                    amount: 2,
                },
            ])),
        );
        let mut outputs = TronChainSigner.sign_stake(&input, &private_key()).unwrap();

        assert_eq!(outputs.len(), 2);
        assert_eq!(signed_json(outputs.remove(0))["raw_data"]["contract"][0]["parameter"]["value"]["resource"], "BANDWIDTH");
        assert_eq!(signed_json(outputs.remove(0))["raw_data"]["contract"][0]["parameter"]["value"]["resource"], "ENERGY");
    }

    fn raw_transfer_transaction() -> Value {
        json!({
            "raw_data": {
                "contract": [{
                    "parameter": {
                        "type_url": "type.googleapis.com/protocol.TransferContract",
                        "value": {
                            "amount": 2000000u64,
                            "owner_address": "415cd0fb0ab3ce40f3051414c604b27756e69e43db",
                            "to_address": "41521ea197907927725ef36d70f25f850d1659c7c7"
                        }
                    },
                    "type": "TransferContract"
                }],
                "expiration": 1539331479000u64,
                "ref_block_bytes": "7b3b",
                "ref_block_hash": "b21ace8d6ac20e7e",
                "timestamp": 1539295479000u64
            },
            "raw_data_hex": "0a027b3b2208b21ace8d6ac20e7e40d8abb9bae62c5a67080112630a2d747970652e676f6f676c65617069732e636f6d2f70726f746f636f6c2e5472616e73666572436f6e747261637412320a15415cd0fb0ab3ce40f3051414c604b27756e69e43db121541521ea197907927725ef36d70f25f850d1659c7c71880897a70d889a4a9e62c",
            "txID": "dc6f6d9325ee44ab3c00528472be16e1572ab076aa161ccd12515029869d0451"
        })
    }

    fn generic_input(transaction: Value, output_type: TransferDataOutputType) -> SignerInput {
        generic_payload(json!({ "transaction": transaction }), output_type)
    }

    fn generic_payload(payload: Value, output_type: TransferDataOutputType) -> SignerInput {
        let payload = serde_json::to_vec(&payload).unwrap();
        signer_input(
            TransactionInputType::Generic(
                Asset::from_chain(Chain::Tron),
                WalletConnectionSessionAppMetadata {
                    name: "Test".to_string(),
                    description: "Test".to_string(),
                    url: "https://example.com".to_string(),
                    icon: "https://example.com/icon.png".to_string(),
                },
                TransferDataExtra {
                    data: Some(payload),
                    output_type,
                    output_action: TransferDataOutputAction::Sign,
                    to: SENDER.to_string(),
                    gas_limit: None,
                    gas_price: None,
                },
            ),
            SENDER,
            RECIPIENT,
            "0",
            TransactionFee::default(),
            None,
            TransactionLoadMetadata::None,
        )
    }

    // Source vector:
    // https://github.com/trustwallet/wallet-core/blob/master/tests/chains/Tron/SignerTests.cpp
    #[test]
    fn sign_raw_json_transfer_matches_wallet_core() {
        let input = generic_input(raw_transfer_transaction(), TransferDataOutputType::EncodedTransaction);
        let output = signed_json(TronChainSigner.sign_data(&input, &private_key()).unwrap());

        assert_eq!(output["txID"], "dc6f6d9325ee44ab3c00528472be16e1572ab076aa161ccd12515029869d0451");
        assert_eq!(
            signature(&output),
            "ede769f6df28aefe6a846be169958c155e23e7e5c9621d2e8dce1719b4d952b63e8a8bf9f00e41204ac1bf69b1a663dacdf764367e48e4a5afcd6b055a747fb200"
        );
    }

    #[test]
    fn sign_raw_json_signature_only_returns_signature() {
        let input = generic_input(raw_transfer_transaction(), TransferDataOutputType::Signature);

        assert_eq!(
            TronChainSigner.sign_data(&input, &private_key()).unwrap(),
            "ede769f6df28aefe6a846be169958c155e23e7e5c9621d2e8dce1719b4d952b63e8a8bf9f00e41204ac1bf69b1a663dacdf764367e48e4a5afcd6b055a747fb200"
        );
    }

    #[test]
    fn sign_raw_json_without_transaction_id_derives_output_transaction_id() {
        let mut transaction = raw_transfer_transaction();
        transaction.as_object_mut().unwrap().remove("txID");
        let input = generic_input(transaction, TransferDataOutputType::EncodedTransaction);
        let output = signed_json(TronChainSigner.sign_data(&input, &private_key()).unwrap());

        assert_eq!(output["txID"], "dc6f6d9325ee44ab3c00528472be16e1572ab076aa161ccd12515029869d0451");
    }

    #[test]
    fn sign_raw_json_rejects_missing_raw_data_and_transaction_id() {
        let mut transaction = raw_transfer_transaction();
        transaction.as_object_mut().unwrap().remove("raw_data");
        transaction.as_object_mut().unwrap().remove("txID");
        let input = generic_input(transaction, TransferDataOutputType::EncodedTransaction);

        assert_eq!(
            TronChainSigner.sign_data(&input, &private_key()).unwrap_err().to_string(),
            "Invalid input: Missing raw_data or transaction ID"
        );
    }

    #[test]
    fn sign_raw_json_wallet_connect_request_preserves_transaction_fields() {
        let payload = serde_json::from_str(include_str!("../../../gem_wallet_connect/testdata/tron_send_transaction.json")).unwrap();
        let input = generic_payload(payload, TransferDataOutputType::EncodedTransaction);
        let output = signed_json(TronChainSigner.sign_data(&input, &private_key()).unwrap());

        assert_eq!(output["txID"], "0c195049c6eb9792017e1411604ef691c2a02725603edacb91721831fa85c4b2");
        assert_eq!(output["visible"], false);
        assert!(output.get("address").is_none());
    }

    #[test]
    fn sign_raw_json_rejects_transaction_id_mismatch() {
        let mut transaction = raw_transfer_transaction();
        transaction["txID"] = json!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        let input = generic_input(transaction, TransferDataOutputType::EncodedTransaction);

        assert_eq!(
            TronChainSigner.sign_data(&input, &private_key()).unwrap_err().to_string(),
            "Invalid input: transaction ID does not match hash of raw_data_hex"
        );
    }

    #[test]
    fn sign_raw_json_rejects_raw_data_mismatch() {
        let mut transaction = raw_transfer_transaction();
        transaction["raw_data"]["contract"][0]["parameter"]["value"]["amount"] = json!(3_000_000u64);
        let input = generic_input(transaction, TransferDataOutputType::EncodedTransaction);

        assert_eq!(
            TronChainSigner.sign_data(&input, &private_key()).unwrap_err().to_string(),
            "Invalid input: raw_data does not match raw_data_hex"
        );
    }

    #[test]
    fn sign_raw_json_rejects_unsupported_contract() {
        let transaction = json!({
            "raw_data": {
                "contract": [{
                    "parameter": {
                        "type_url": "type.googleapis.com/protocol.SetAccountIdContract",
                        "value": {
                            "account_id": "74657374",
                            "owner_address": "415cd0fb0ab3ce40f3051414c604b27756e69e43db"
                        }
                    },
                    "type": "SetAccountIdContract"
                }],
                "expiration": 1539331479000u64,
                "ref_block_bytes": "7b3b",
                "ref_block_hash": "b21ace8d6ac20e7e",
                "timestamp": 1539295479000u64
            },
            "raw_data_hex": "0a027b3b2208b21ace8d6ac20e7e40d8abb9bae62c5a56081312520a31747970652e676f6f676c65617069732e636f6d2f70726f746f636f6c2e5365744163636f756e744964436f6e7472616374121d0a04746573741215415cd0fb0ab3ce40f3051414c604b27756e69e43db70d889a4a9e62c",
            "txID": "b3e6d49784acfe62f83f1235bab54613cfb7813dddc8cffc87ced07cafc02fbe"
        });
        let input = generic_input(transaction, TransferDataOutputType::EncodedTransaction);

        assert_eq!(
            TronChainSigner.sign_data(&input, &private_key()).unwrap_err().to_string(),
            "Invalid input: unsupported Tron contract type: SetAccountIdContract"
        );
    }

    #[test]
    fn sign_raw_json_rejects_known_unsupported_contract() {
        let mut transaction = raw_transfer_transaction();
        transaction["raw_data"]["contract"][0]["parameter"]["type_url"] = json!("type.googleapis.com/protocol.DelegateResourceContract");
        transaction["raw_data"]["contract"][0]["type"] = json!("DelegateResourceContract");
        let input = generic_input(transaction, TransferDataOutputType::EncodedTransaction);

        assert_eq!(
            TronChainSigner.sign_data(&input, &private_key()).unwrap_err().to_string(),
            "Invalid input: unsupported Tron contract type: DelegateResourceContract"
        );
    }

    #[test]
    fn sign_transfer_rejects_invalid_address() {
        let input = signer_input(
            TransactionInputType::Transfer(Asset::from_chain(Chain::Tron)),
            SENDER,
            "INVALID_NOT_BASE58",
            "100",
            TransactionFee::default(),
            None,
            metadata(TronStakeData::Votes(vec![])),
        );

        assert_eq!(
            TronChainSigner.sign_transfer(&input, &private_key()).unwrap_err().to_string(),
            "Invalid input: invalid Tron address: INVALID_NOT_BASE58"
        );
    }

    #[test]
    fn sign_transfer_rejects_sender_private_key_mismatch() {
        let input = native_input("100", TransactionFee::default(), None);

        assert_eq!(
            TronChainSigner.sign_transfer(&input, &nile_private_key()).unwrap_err().to_string(),
            "Invalid input: Tron sender address does not match private key"
        );
    }

    #[test]
    fn sign_transfer_rejects_invalid_metadata() {
        let input = signer_input(
            TransactionInputType::Transfer(Asset::from_chain(Chain::Tron)),
            SENDER,
            RECIPIENT,
            "100",
            TransactionFee::default(),
            None,
            TransactionLoadMetadata::None,
        );

        assert_eq!(
            TronChainSigner.sign_transfer(&input, &private_key()).unwrap_err().to_string(),
            "Invalid input: Missing tron metadata"
        );
    }

    #[test]
    fn sign_message_uses_ethereum_recovery_id_offset() {
        let signature = TronChainSigner.sign_message(b"This is a message to be signed for Tron", &private_key()).unwrap();
        let bytes = decode_hex(&signature).unwrap();

        assert!(signature.starts_with("0x"));
        assert_eq!(bytes.len(), 65);
        assert!(bytes[64] == 27 || bytes[64] == 28);
    }
}
