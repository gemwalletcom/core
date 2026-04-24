use std::str::FromStr;

use alloy_consensus::TxEip1559;
use alloy_primitives::{Address, Bytes, TxKind, U256};
use num_bigint::BigInt;
use num_traits::Num;
use primitives::{ChainSigner, EVMChain, NFTType, SignerError, SignerInput, StakeType, decode_hex, swap::SwapQuoteDataType};

use super::model::TransactionParams;
use super::sign_eip1559_tx;
use crate::encode::{encode_erc20_approve, encode_erc20_transfer, encode_erc721_transfer, encode_erc1155_transfer};

#[allow(dead_code)]
pub struct EvmChainSigner {
    chain: EVMChain,
}

impl EvmChainSigner {
    pub fn new(chain: EVMChain) -> Self {
        Self { chain }
    }
}

impl ChainSigner for EvmChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let params = TransactionParams::from_input(input)?;
        sign_and_encode(
            &build_eip1559_transaction(&params, &input.destination_address, value_u256(&input.value)?, Bytes::new())?,
            private_key,
        )
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let params = TransactionParams::from_input(input)?;
        let token_id = input.input_type.get_asset().id.get_token_id()?;
        let data = encode_erc20_transfer(&input.destination_address, &BigInt::from_str_radix(&input.value, 10)?)?;
        sign_and_encode(&build_eip1559_transaction(&params, token_id, U256::ZERO, Bytes::from(data))?, private_key)
    }

    fn sign_nft_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let params = TransactionParams::from_input(input)?;
        let nft_asset = input.input_type.get_nft_asset()?;
        let contract = nft_asset.get_contract_address()?;
        let data = match nft_asset.token_type {
            NFTType::ERC721 => encode_erc721_transfer(&input.sender_address, &input.destination_address, &nft_asset.token_id),
            NFTType::ERC1155 => encode_erc1155_transfer(&input.sender_address, &input.destination_address, &nft_asset.token_id),
            _ => return Err(SignerError::invalid_input("unsupported NFT type for EVM")),
        }?;
        sign_and_encode(&build_eip1559_transaction(&params, contract, U256::ZERO, Bytes::from(data))?, private_key)
    }

    fn sign_token_approval(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let params = TransactionParams::from_input(input)?;
        let approval = input.input_type.get_approval_data()?;
        sign_and_encode(
            &build_eip1559_transaction(&params, &approval.token, U256::ZERO, Bytes::from(encode_erc20_approve(&approval.spender)?))?,
            private_key,
        )
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap = input.input_type.get_swap_data()?;
        let swap_data = &swap.data;
        let from_asset = input.input_type.get_asset();

        match swap_data.data_type {
            SwapQuoteDataType::Transfer => {
                let params = TransactionParams::from_input(input)?;
                if from_asset.id.is_token() {
                    let token_id = from_asset.id.get_token_id()?;
                    let amount = BigInt::from_str_radix(&input.value, 10)?;
                    let data = encode_erc20_transfer(&swap_data.to, &amount)?;
                    Ok(vec![sign_and_encode(
                        &build_eip1559_transaction(&params, token_id, U256::ZERO, Bytes::from(data))?,
                        private_key,
                    )?])
                } else {
                    Ok(vec![sign_and_encode(
                        &build_eip1559_transaction(&params, &swap_data.to, value_u256(&input.value)?, Bytes::new())?,
                        private_key,
                    )?])
                }
            }
            SwapQuoteDataType::Contract => {
                let value = value_u256(&swap_data.value)?;
                let gas_limit = match &swap_data.approval {
                    Some(_) => swap_data.gas_limit.as_ref().and_then(|gl| gl.parse().ok()).ok_or("missing swap gas limit")?,
                    None => input.fee.gas_limit()?,
                };
                sign_contract_call(
                    input,
                    &swap_data.to,
                    decode_hex(&swap_data.data)?,
                    gas_limit,
                    value,
                    swap_data.approval.as_ref(),
                    private_key,
                )
            }
        }
    }

    fn sign_earn(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let earn_data = input.input_type.get_earn_data()?;
        let gas_limit = earn_data.gas_limit.as_ref().and_then(|gl| gl.parse().ok()).map_or_else(|| input.fee.gas_limit(), Ok)?;
        sign_contract_call(
            input,
            &earn_data.contract_address,
            decode_hex(&earn_data.call_data)?,
            gas_limit,
            U256::ZERO,
            earn_data.approval.as_ref(),
            private_key,
        )
    }

    fn sign_stake(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let stake_type = input.input_type.get_stake_type()?;
        let contract_call = input.metadata.get_contract_call()?;
        let value = match stake_type {
            StakeType::Stake(_) => value_u256(&input.value)?,
            _ => U256::ZERO,
        };
        sign_contract_call(
            input,
            &contract_call.contract_address,
            decode_hex(&contract_call.call_data)?,
            input.fee.gas_limit()?,
            value,
            None,
            private_key,
        )
    }

    fn sign_withdrawal(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let contract_call = input.metadata.get_contract_call()?;
        let params = TransactionParams::from_input(input)?;
        let gas_limit = input.fee.gas_limit()?;
        let transaction = build_eip1559_transaction(
            &TransactionParams { gas_limit, ..params },
            &contract_call.contract_address,
            U256::ZERO,
            Bytes::from(decode_hex(&contract_call.call_data)?),
        )?;
        sign_and_encode(&transaction, private_key)
    }

    fn sign_data(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let extra = input.input_type.get_generic_data()?;
        let base = TransactionParams::from_input(input)?;
        let gas_limit = extra.gas_limit.as_ref().and_then(|gl| gl.to_string().parse().ok()).unwrap_or(base.gas_limit);
        let params = TransactionParams { gas_limit, ..base };
        sign_and_encode(
            &build_eip1559_transaction(&params, &extra.to, value_u256(&input.value)?, Bytes::from(extra.data.clone().unwrap_or_default()))?,
            private_key,
        )
    }

    fn sign_message(&self, message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
        let json_str = std::str::from_utf8(message).map_err(|_| SignerError::invalid_input("message must be valid UTF-8"))?;
        Ok(format!("0x{}", signer::Signer::sign_eip712(json_str, private_key)?))
    }
}

fn value_u256(value: &str) -> Result<U256, SignerError> {
    U256::from_str(value).map_err(SignerError::from_display)
}

fn build_eip1559_transaction(params: &TransactionParams, to: &str, value: U256, input: Bytes) -> Result<TxEip1559, SignerError> {
    let to_address = Address::parse_checksummed(to, None)
        .or_else(|_| to.parse::<Address>())
        .map_err(|_| SignerError::invalid_input("invalid to address"))?;

    Ok(TxEip1559 {
        chain_id: params.chain_id,
        nonce: params.nonce,
        gas_limit: params.gas_limit,
        max_fee_per_gas: params.max_fee_per_gas,
        max_priority_fee_per_gas: params.max_priority_fee_per_gas,
        to: TxKind::Call(to_address),
        value,
        access_list: Default::default(),
        input,
    })
}

fn sign_and_encode(transaction: &TxEip1559, private_key: &[u8]) -> Result<String, SignerError> {
    Ok(hex::encode(sign_eip1559_tx(transaction, private_key)?))
}

fn sign_contract_call(
    input: &SignerInput,
    contract_address: &str,
    call_data: Vec<u8>,
    gas_limit: u64,
    value: U256,
    approval: Option<&primitives::swap::ApprovalData>,
    private_key: &[u8],
) -> Result<Vec<String>, SignerError> {
    let params = TransactionParams::from_input(input)?;

    if let Some(approval) = approval {
        let approval_transaction = build_eip1559_transaction(&params, &approval.token, U256::ZERO, Bytes::from(encode_erc20_approve(&approval.spender)?))?;
        let main_params = TransactionParams {
            nonce: params.nonce + 1,
            gas_limit,
            ..params
        };
        let main_transaction = build_eip1559_transaction(&main_params, contract_address, value, Bytes::from(call_data))?;
        Ok(vec![sign_and_encode(&approval_transaction, private_key)?, sign_and_encode(&main_transaction, private_key)?])
    } else {
        let main_params = TransactionParams { gas_limit, ..params };
        Ok(vec![sign_and_encode(
            &build_eip1559_transaction(&main_params, contract_address, value, Bytes::from(call_data))?,
            private_key,
        )?])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{
        Asset, Chain, ChainSigner, DelegationValidator, EVMChain, NFTType, SignerInput, TransactionInputType, TransactionLoadMetadata, TransferDataExtra,
        WalletConnectionSessionAppMetadata, contract_call_data::ContractCallData, nft::NFTAsset, swap::*,
    };

    #[test]
    fn test_sign_transfer() {
        let signer = EvmChainSigner::new(EVMChain::Ethereum);
        let input = SignerInput::mock_evm(TransactionInputType::Transfer(Asset::from_chain(Chain::Ethereum)), "1000000000000000000", 21000);
        assert_eq!(
            signer.sign_transfer(&input, &TEST_PRIVATE_KEY).unwrap(),
            "02f8730180843b9aca008504a817c800825208942b5ad5c4795c026514f8317c7a215e218dccd6cf880de0b6b3a764000080c001a0ea6700354e2542e163e08c111d7b1d7e2a9d371a06977c9a79c42783c3237af9a001809a71f1fa2309f204b4ebed1a9e68f0e60ab736b98284727f2d8427ab705f"
        );
    }

    #[test]
    fn test_sign_token_transfer() {
        let signer = EvmChainSigner::new(EVMChain::Ethereum);
        let input = SignerInput::mock_evm(TransactionInputType::Transfer(Asset::mock_erc20()), "1000000", 65000);
        assert_eq!(
            signer.sign_token_transfer(&input, &TEST_PRIVATE_KEY).unwrap(),
            "02f8b00180843b9aca008504a817c80082fde894a0b86a33e6441066d64bb38954e41f6b4b925c5980b844a9059cbb0000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cf00000000000000000000000000000000000000000000000000000000000f4240c001a09ca8ae6c1d3e9a70465ae36e44c4ca9982a0b94c3cb8ec7c56e6a183f2d04f16a02275a147339b8a41e36670cbec2df08df035ea3b403eedd8325b150b53a3d7f4"
        );
    }

    #[test]
    fn test_sign_nft_transfer() {
        let signer = EvmChainSigner::new(EVMChain::Ethereum);

        let input = SignerInput::mock_evm(TransactionInputType::TransferNft(Asset::from_chain(Chain::Ethereum), NFTAsset::mock()), "0", 100000);
        assert_eq!(
            signer.sign_nft_transfer(&input, &TEST_PRIVATE_KEY).unwrap(),
            "02f8d10180843b9aca008504a817c800830186a094dac17f958d2ee523a2206206994597c13d831ec780b86442842e0e0000000000000000000000007e5f4552091a69125d5dfcb7b8c2659029395bdf0000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cf0000000000000000000000000000000000000000000000000000000000000001c080a08371f982a5384532d5ac3336a174239f571cd75663ddc1d6f3892a59c940c983a035902737dfc2f2af6df4244e741f652f4d764230aecf9d5a910d0d027dc4238d"
        );

        let input = SignerInput::mock_evm(
            TransactionInputType::TransferNft(Asset::from_chain(Chain::Ethereum), NFTAsset::mock_with_type(NFTType::ERC1155)),
            "0",
            100000,
        );
        assert_eq!(
            signer.sign_nft_transfer(&input, &TEST_PRIVATE_KEY).unwrap(),
            "02f901310180843b9aca008504a817c800830186a094dac17f958d2ee523a2206206994597c13d831ec780b8c4f242432a0000000000000000000000007e5f4552091a69125d5dfcb7b8c2659029395bdf0000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cf0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000c080a032eb8933adf3d5fb105c292e413bae35339b9ee6b30ac4b8b679e9f14a0009dba00964abf4af5aa48120c18942ef2b64189bda437a97a7616e14b39624c0870329"
        );
    }

    #[test]
    fn test_sign_token_approval() {
        let signer = EvmChainSigner::new(EVMChain::Ethereum);
        let input = SignerInput::mock_evm(TransactionInputType::TokenApprove(Asset::from_chain(Chain::Ethereum), ApprovalData::mock()), "0", 65000);
        assert_eq!(
            signer.sign_token_approval(&input, &TEST_PRIVATE_KEY).unwrap(),
            "02f8b00180843b9aca008504a817c80082fde894dac17f958d2ee523a2206206994597c13d831ec780b844095ea7b30000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc001a02fde3a01cfa4c2349782fa026932003f5eae7077763db75b31be593e7c15f4a3a048071a05fccf3905cb59be79fc9a0442d801907506193c158858cd6b7ef31fa3"
        );
    }

    #[test]
    fn test_sign_swap_without_approval() {
        let signer = EvmChainSigner::new(EVMChain::Ethereum);
        let swap_data = SwapData {
            quote: SwapQuote::mock(),
            data: SwapQuoteData {
                value: "1000000000000000000".to_string(),
                data: "abcd".to_string(),
                gas_limit: None,
                ..SwapQuoteData::mock()
            },
        };
        let input = SignerInput::mock_evm(
            TransactionInputType::Swap(Asset::from_chain(Chain::Ethereum), Asset::from_chain(Chain::Ethereum), swap_data),
            "1000000000000000000",
            200000,
        );
        let result = signer.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            "02f8760180843b9aca008504a817c80083030d40942b5ad5c4795c026514f8317c7a215e218dccd6cf880de0b6b3a764000082abcdc001a0a576e21827f710051c5d9402777cd913469bfa46a6a87281c60b2c48eb620db1a052ec097cab72419fd228b09276db146c1ad8c4e6fa35a34166ba718bdeadc892"
        );
    }

    #[test]
    fn test_sign_swap_with_approval() {
        let signer = EvmChainSigner::new(EVMChain::Ethereum);
        let swap_data = SwapData {
            quote: SwapQuote::mock(),
            data: SwapQuoteData {
                data: "abcd".to_string(),
                approval: Some(ApprovalData::mock()),
                gas_limit: Some("200000".to_string()),
                ..SwapQuoteData::mock()
            },
        };
        let input = SignerInput::mock_evm(
            TransactionInputType::Swap(Asset::from_chain(Chain::Ethereum), Asset::from_chain(Chain::Ethereum), swap_data),
            "0",
            65000,
        );
        let result = signer.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            "02f8b00180843b9aca008504a817c80082fde894dac17f958d2ee523a2206206994597c13d831ec780b844095ea7b30000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc001a02fde3a01cfa4c2349782fa026932003f5eae7077763db75b31be593e7c15f4a3a048071a05fccf3905cb59be79fc9a0442d801907506193c158858cd6b7ef31fa3"
        );
        assert_eq!(
            result[1],
            "02f86e0101843b9aca008504a817c80083030d40942b5ad5c4795c026514f8317c7a215e218dccd6cf8082abcdc080a02ecc5acc573cb465ae28b24756c384a5dfd5eb4ced9479d73d93e50dea7f30fba01cbb3e7bc343e8294ed26a005b001408498ed364402b5a37830be9b8d850bda4"
        );
    }

    #[test]
    fn test_sign_stake() {
        let signer = EvmChainSigner::new(EVMChain::Ethereum);
        let metadata = TransactionLoadMetadata::Evm {
            nonce: 5,
            chain_id: 1,
            contract_call: Some(ContractCallData::mock_with_call_data(
                "3a29dbae0000000000000000000000000000000000000000000000000000000000000017",
            )),
        };
        let input = SignerInput::mock_evm_with_metadata(
            TransactionInputType::Stake(Asset::from_chain(Chain::Ethereum), StakeType::Stake(DelegationValidator::mock())),
            "1000000000000000000",
            200000,
            metadata,
        );
        let result = signer.sign_stake(&input, &TEST_PRIVATE_KEY).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            "02f8980105843b9aca008504a817c80083030d40942b5ad5c4795c026514f8317c7a215e218dccd6cf880de0b6b3a7640000a43a29dbae0000000000000000000000000000000000000000000000000000000000000017c001a067b915da126e46cfb9c78db7aa5c277743b2de43450dd744f2e5bc16146b0954a01fa8f62608f997199eab37464df9cc1821a2c66a6c8f36d6ba438206a7a3556a"
        );
    }

    #[test]
    fn test_sign_data() {
        let signer = EvmChainSigner::new(EVMChain::Ethereum);
        let extra = TransferDataExtra::mock_encoded_transaction(vec![0xab, 0xcd]);
        let input = SignerInput::mock_evm(
            TransactionInputType::Generic(Asset::from_chain(Chain::Ethereum), WalletConnectionSessionAppMetadata::mock(), extra),
            "0",
            100000,
        );
        assert_eq!(
            signer.sign_data(&input, &TEST_PRIVATE_KEY).unwrap(),
            "02f86e0180843b9aca008504a817c800830186a0942b5ad5c4795c026514f8317c7a215e218dccd6cf8082abcdc080a085087f2d3c999ea4e253274ed68a9e58cc7eb9f2ee7e037897ce371ddc74f0bea06613bd4201e26f5738ca375fcbd19ee8e8a1fd633e7d2f2061a14f3d0ee1173a"
        );
    }

    #[test]
    fn test_sign_earn() {
        let signer = EvmChainSigner::new(EVMChain::Ethereum);
        let input = SignerInput::mock_evm(
            TransactionInputType::Earn(
                Asset::from_chain(Chain::Ethereum),
                primitives::EarnType::Deposit(DelegationValidator::mock()),
                ContractCallData::mock(),
            ),
            "0",
            200000,
        );
        let result = signer.sign_earn(&input, &TEST_PRIVATE_KEY).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            "02f86e0180843b9aca008504a817c80083030d40942b5ad5c4795c026514f8317c7a215e218dccd6cf8082abcdc001a00a342182b976d28a460ede1a104708d57d1174a5f4ba383eef91c2a774dfab62a0657d5a65976b2241a13554260c538372066938c51eb8116bbad6211c6b29bfff"
        );
    }

    #[test]
    fn test_sign_earn_with_approval() {
        let signer = EvmChainSigner::new(EVMChain::Ethereum);
        let earn_data = ContractCallData {
            approval: Some(ApprovalData::mock()),
            gas_limit: Some("200000".to_string()),
            ..ContractCallData::mock()
        };
        let input = SignerInput::mock_evm(
            TransactionInputType::Earn(Asset::from_chain(Chain::Ethereum), primitives::EarnType::Deposit(DelegationValidator::mock()), earn_data),
            "0",
            65000,
        );
        let result = signer.sign_earn(&input, &TEST_PRIVATE_KEY).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            "02f8b00180843b9aca008504a817c80082fde894dac17f958d2ee523a2206206994597c13d831ec780b844095ea7b30000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc001a02fde3a01cfa4c2349782fa026932003f5eae7077763db75b31be593e7c15f4a3a048071a05fccf3905cb59be79fc9a0442d801907506193c158858cd6b7ef31fa3"
        );
        assert_eq!(
            result[1],
            "02f86e0101843b9aca008504a817c80083030d40942b5ad5c4795c026514f8317c7a215e218dccd6cf8082abcdc080a02ecc5acc573cb465ae28b24756c384a5dfd5eb4ced9479d73d93e50dea7f30fba01cbb3e7bc343e8294ed26a005b001408498ed364402b5a37830be9b8d850bda4"
        );
    }

    #[test]
    fn test_invalid_metadata() {
        let signer = EvmChainSigner::new(EVMChain::Ethereum);
        let input = SignerInput::mock_evm_with_metadata(
            TransactionInputType::Transfer(Asset::from_chain(Chain::Ethereum)),
            "1000000000000000000",
            21000,
            TransactionLoadMetadata::None,
        );
        assert!(signer.sign_transfer(&input, &TEST_PRIVATE_KEY).is_err());
    }
}
