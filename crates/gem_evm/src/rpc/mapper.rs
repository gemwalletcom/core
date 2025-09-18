use chrono::DateTime;
use itertools::izip;
use lazy_static::lazy_static;
use num_bigint::BigUint;
use num_traits::Num;

use super::{staking_mapper::StakingMapper, swap_mapper::SwapMapper};
use crate::{
    address::{ethereum_address_checksum, ethereum_address_from_topic},
    registry::ContractRegistry,
    rpc::model::{Block, Transaction, TransactionReciept, TransactionReplayTrace},
};
use primitives::{chain::Chain, transaction_metadata_types::TransactionNFTTransferMetadata, AssetId, NFTAssetId, TransactionType};

pub const INPUT_0X: &str = "0x";
pub const FUNCTION_ERC20_TRANSFER: &str = "0xa9059cbb";
pub const FUNCTION_EIP721_TRANSFER: &str = "0x23b872dd"; // transferFrom(address from, address to, uint256 tokenId)
pub const FUNCTION_EIP1155_TRANSFER: &str = "0xf242432a"; // safeTransferFrom(address from, address to, uint256 tokenId, uint256 amount, bytes data)
pub const TRANSFER_TOPIC: &str = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
pub const TRANSFER_SINGLE: &str = "0xc3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62";
pub const TRANSFER_GAS_LIMIT: u64 = 21000;

lazy_static! {
    pub static ref CONTRACT_REGISTRY: ContractRegistry = ContractRegistry::default();
}
pub struct EthereumMapper;

impl EthereumMapper {
    pub fn map_transactions(
        chain: Chain,
        block: Block,
        transactions_reciepts: Vec<TransactionReciept>,
        traces: Option<Vec<TransactionReplayTrace>>,
    ) -> Vec<primitives::Transaction> {
        match traces {
            Some(traces) => izip!(block.transactions.into_iter(), transactions_reciepts.iter(), traces.iter())
                .filter_map(|(transaction, receipt, trace)| {
                    EthereumMapper::map_transaction(chain, &transaction, receipt, Some(trace), &block.timestamp, Some(&CONTRACT_REGISTRY))
                })
                .collect(),
            None => block
                .transactions
                .into_iter()
                .zip(transactions_reciepts.iter())
                .filter_map(|(transaction, receipt)| {
                    EthereumMapper::map_transaction(chain, &transaction, receipt, None, &block.timestamp, Some(&CONTRACT_REGISTRY))
                })
                .collect(),
        }
    }

    pub fn map_transaction(
        chain: Chain,
        transaction: &Transaction,
        transaction_reciept: &TransactionReciept,
        trace: Option<&TransactionReplayTrace>,
        timestamp: &BigUint,
        contract_registry: Option<&ContractRegistry>,
    ) -> Option<primitives::Transaction> {
        let state = transaction_reciept.get_state();
        let hash = transaction.hash.clone();
        let value = transaction.value.to_string();
        let fee = transaction_reciept.get_fee().to_string();
        let fee_asset_id = chain.as_asset_id();
        let from = ethereum_address_checksum(&transaction.from.clone()).ok()?;
        let to = ethereum_address_checksum(&transaction.to.clone().unwrap_or_default()).ok()?;
        let created_at = DateTime::from_timestamp(timestamp.clone().try_into().ok()?, 0)?;

        let is_smart_contract_call = transaction.to.is_some() && transaction.input.len() > 2;
        let is_native_transfer = transaction.input == INPUT_0X && transaction_reciept.gas_used == BigUint::from(TRANSFER_GAS_LIMIT);
        let is_native_transfer_with_data = transaction.input.len() > 2
            && transaction.gas > TRANSFER_GAS_LIMIT
            && Self::get_data_cost(&transaction.input).is_some_and(|data_cost| transaction_reciept.gas_used <= BigUint::from(TRANSFER_GAS_LIMIT + data_cost));

        if is_native_transfer || is_native_transfer_with_data {
            let transaction = primitives::Transaction::new(
                hash,
                chain.as_asset_id(),
                from,
                to,
                None,
                TransactionType::Transfer,
                state,
                fee.to_string(),
                fee_asset_id,
                value,
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }

        // Try to decode Uniswap V3 or V4 transaction
        if transaction.to.is_some() && transaction.input.len() >= 8 {
            if let Some(tx) = SwapMapper::map_transaction(&chain, transaction, transaction_reciept, trace, created_at, contract_registry) {
                return Some(tx);
            }
        }

        // nft eip 721

        if transaction.input.starts_with(FUNCTION_EIP721_TRANSFER)
            && transaction_reciept
                .logs
                .last()
                .is_some_and(|log| log.topics.len() == 4 && log.topics.first().is_some_and(|x| x == TRANSFER_TOPIC))
        {
            if let Some(log) = transaction_reciept.logs.last() {
                let address = ethereum_address_from_topic(&log.topics[2])?;
                let token_id = BigUint::from_str_radix(&log.topics[3].replace("0x", ""), 16).ok()?;
                let contract_address = ethereum_address_checksum(&log.address).ok()?;
                let metadata = TransactionNFTTransferMetadata::from_asset_id(NFTAssetId::new(chain, &contract_address, &token_id.to_string()));

                let transaction = primitives::Transaction::new(
                    hash,
                    AssetId::from_chain(chain),
                    from.clone(),
                    address,
                    None,
                    TransactionType::TransferNFT,
                    state,
                    fee.to_string(),
                    fee_asset_id,
                    "0".to_string(),
                    None,
                    serde_json::to_value(metadata).ok(),
                    created_at,
                );
                return Some(transaction);
            }
        }

        // nft eip 1155

        if transaction.input.starts_with(FUNCTION_EIP1155_TRANSFER)
            && transaction_reciept
                .logs
                .last()
                .is_some_and(|log| log.topics.len() == 4 && log.topics.first().is_some_and(|x| x == TRANSFER_SINGLE))
        {
            if let Some(log) = transaction_reciept.logs.last() {
                let to_address = ethereum_address_from_topic(&log.topics[3])?;
                let token_id = BigUint::from_str_radix(&log.data.replace("0x", "")[0..64], 16).ok()?;
                let contract_address = ethereum_address_checksum(&log.address).ok()?;
                let metadata = TransactionNFTTransferMetadata::from_asset_id(NFTAssetId::new(chain, &contract_address, &token_id.to_string()));

                let transaction = primitives::Transaction::new(
                    hash,
                    AssetId::from_chain(chain),
                    from.clone(),
                    to_address,
                    None,
                    TransactionType::TransferNFT,
                    state,
                    fee.to_string(),
                    fee_asset_id,
                    "0".to_string(),
                    None,
                    serde_json::to_value(metadata).ok(),
                    created_at,
                );
                return Some(transaction);
            }
        }

        // erc20 transfer - check both direct transfer calls and smart contract calls that emit transfer events
        let transfer_log = transaction_reciept.logs.iter().find(|log| {
            // ERC20 transfers have exactly 3 topics (event signature, from, to)
            log.topics.len() == 3 && log.topics.first().is_some_and(|x| x == TRANSFER_TOPIC)
        });

        if let Some(log) = transfer_log {
            let from_address_in_log = ethereum_address_from_topic(log.topics.get(1)?)?;
            let to_address_in_log = ethereum_address_from_topic(log.topics.get(2)?)?;
            let amount = BigUint::from_str_radix(&log.data.replace("0x", ""), 16).ok()?;
            let token_id = ethereum_address_checksum(&log.address).ok()?;

            // Check if this is a relevant ERC20 transfer
            let is_direct_transfer = transaction.input.starts_with(FUNCTION_ERC20_TRANSFER);

            if is_direct_transfer
                || (is_smart_contract_call && (from_address_in_log == from || from_address_in_log == to)) && transaction_reciept.logs.len() <= 2
            {
                return Some(primitives::Transaction::new(
                    hash,
                    AssetId::from_token(chain, &token_id),
                    from.clone(),
                    to_address_in_log,
                    None,
                    TransactionType::Transfer,
                    state,
                    fee.to_string(),
                    fee_asset_id,
                    amount.to_string(),
                    None,
                    None,
                    created_at,
                ));
            }
        }

        // Try to decode BSC staking transaction
        if let Some(tx) = StakingMapper::map_transaction(&chain, transaction, transaction_reciept, created_at) {
            return Some(tx);
        }

        // Check for smart contract call
        // if transaction.to.is_some() && transaction.input.len() > 2 && Self::has_smart_contract_indicators(transaction, transaction_reciept) {
        //     let transaction = primitives::Transaction::new(
        //         transaction.hash.clone(),
        //         chain.as_asset_id(),
        //         from,
        //         to,
        //         None,
        //         TransactionType::SmartContractCall,
        //         state,
        //         fee.to_string(),
        //         chain.as_asset_id(),
        //         value,
        //         None,
        //         None,
        //         created_at,
        //     );
        //     return Some(transaction);
        // }
        None
    }

    fn get_data_cost(input: &str) -> Option<u64> {
        let bytes = hex::decode(input.trim_start_matches("0x")).ok()?;
        let data_cost = bytes.iter().map(|byte| if *byte == 0 { 4 } else { 68 }).sum();

        Some(data_cost)
    }

    #[allow(unused)]
    fn has_smart_contract_indicators(transaction: &Transaction, transaction_reciept: &TransactionReciept) -> bool {
        // 1. Gas limit > 21,000 (simple transfers use exactly 21,000)
        // 2. Receipt has logs (contract execution emits events)
        let has_logs = !transaction_reciept.logs.is_empty();

        transaction.gas > TRANSFER_GAS_LIMIT && has_logs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::model::{Log, Transaction, TransactionReciept};
    use num_bigint::BigUint;
    use primitives::{Chain, JsonRpcResult};

    #[test]
    fn test_map_smart_contract_call() {
        let contract_call_tx_json: serde_json::Value = serde_json::from_str(include_str!("../../testdata/contract_call_tx.json")).unwrap();
        let contract_call_tx: Transaction = serde_json::from_value::<JsonRpcResult<Transaction>>(contract_call_tx_json).unwrap().result;

        let contract_call_receipt_json: serde_json::Value = serde_json::from_str(include_str!("../../testdata/contract_call_tx_receipt.json")).unwrap();
        let contract_call_receipt = serde_json::from_value::<JsonRpcResult<TransactionReciept>>(contract_call_receipt_json)
            .unwrap()
            .result;

        let _transaction = EthereumMapper::map_transaction(
            Chain::Ethereum,
            &contract_call_tx,
            &contract_call_receipt,
            None,
            &BigUint::from(1735671600u64),
            None,
        );

        // assert_eq!(transaction.transaction_type, TransactionType::SmartContractCall);
        // assert_eq!(transaction.hash, "0x876707912c2d625723aa14bf268d83ede36c2657c70da500628e40e6b51577c9");
        // assert_eq!(transaction.from, "0x39ab5f6f1269590225EdAF9ad4c5967B09243747");
        // assert_eq!(transaction.to, "0xB907Dcc926b5991A149d04Cb7C0a4a25dC2D8f9a");
    }

    #[test]
    fn test_has_smart_contract_indicators() {
        let contract_call_tx_json: serde_json::Value = serde_json::from_str(include_str!("../../testdata/contract_call_tx.json")).unwrap();
        let contract_call_tx: Transaction = serde_json::from_value::<JsonRpcResult<Transaction>>(contract_call_tx_json).unwrap().result;

        let contract_call_receipt_json: serde_json::Value = serde_json::from_str(include_str!("../../testdata/contract_call_tx_receipt.json")).unwrap();
        let contract_call_receipt = serde_json::from_value::<JsonRpcResult<TransactionReciept>>(contract_call_receipt_json)
            .unwrap()
            .result;

        assert!(EthereumMapper::has_smart_contract_indicators(&contract_call_tx, &contract_call_receipt));
        assert_eq!(contract_call_tx.gas, 400000);
    }

    #[test]
    fn test_erc20_transfer() {
        let erc20_transfer_tx =
            serde_json::from_value::<JsonRpcResult<Transaction>>(serde_json::from_str(include_str!("../../testdata/transfer_erc20.json")).unwrap())
                .unwrap()
                .result;
        let erc20_transfer_receipt = serde_json::from_value::<JsonRpcResult<TransactionReciept>>(
            serde_json::from_str(include_str!("../../testdata/transfer_erc20_receipt.json")).unwrap(),
        )
        .unwrap()
        .result;

        let transaction = EthereumMapper::map_transaction(
            Chain::Arbitrum,
            &erc20_transfer_tx,
            &erc20_transfer_receipt,
            None,
            &BigUint::from(1735671600u64),
            None,
        )
        .unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(
            transaction.asset_id,
            AssetId::from_token(Chain::Arbitrum, "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9")
        );
        assert_eq!(transaction.from, "0x8d7460E51bCf4eD26877cb77E56f3ce7E9f5EB8F");
        assert_eq!(transaction.to, "0x2Fc617E933a52713247CE25730f6695920B3befe");
        assert_eq!(transaction.value, "4801292");
    }

    #[test]
    fn test_nft_eip721_transfer() {
        let transaction =
            serde_json::from_value::<JsonRpcResult<Transaction>>(serde_json::from_str(include_str!("../../testdata/transfer_nft_eip721.json")).unwrap())
                .unwrap()
                .result;
        let transaction_reciept = serde_json::from_value::<JsonRpcResult<TransactionReciept>>(
            serde_json::from_str(include_str!("../../testdata/transfer_nft_eip721_receipt.json")).unwrap(),
        )
        .unwrap()
        .result;

        let transaction =
            EthereumMapper::map_transaction(Chain::Ethereum, &transaction, &transaction_reciept, None, &BigUint::from(1735671600u64), None).unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::TransferNFT);

        assert_eq!(transaction.asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(transaction.from, "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4");
        assert_eq!(transaction.to, "0xf1158986419F6058231b0Dbd7A78Ff0674ebBc50");
        assert_eq!(transaction.value, "0");
        assert_eq!(
            transaction.metadata,
            Some(serde_json::json!(TransactionNFTTransferMetadata::new(
                "ethereum_0x47A00fC8590C11bE4c419D9Ae50DEc267B6E24ee_9143".to_string(),
                None,
            )))
        );
    }

    #[test]
    fn test_nft_eip1155_transfer() {
        let transaction =
            serde_json::from_value::<JsonRpcResult<Transaction>>(serde_json::from_str(include_str!("../../testdata/transfer_nft_eip1155.json")).unwrap())
                .unwrap()
                .result;
        let transaction_reciept = serde_json::from_value::<JsonRpcResult<TransactionReciept>>(
            serde_json::from_str(include_str!("../../testdata/transfer_nft_eip1155_receipt.json")).unwrap(),
        )
        .unwrap()
        .result;

        let transaction =
            EthereumMapper::map_transaction(Chain::Ethereum, &transaction, &transaction_reciept, None, &BigUint::from(1735671600u64), None).unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::TransferNFT);

        assert_eq!(transaction.asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(transaction.from, "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4");
        assert_eq!(transaction.to, "0xEE67a32a55318a211CE4BB5051Ed98c679851143");
        assert_eq!(transaction.value, "0");
        assert_eq!(
            transaction.metadata,
            Some(serde_json::json!(TransactionNFTTransferMetadata::new(
                "ethereum_0xD4416b13d2b3a9aBae7AcD5D6C2BbDBE25686401_78312089388574796712357673212383836573632856632295981350303734331484536429721".to_string(),
                None,
            )))
        );
    }

    #[test]
    fn test_smart_contract_erc20_transfer() {
        let sc_erc20_tx =
            serde_json::from_value::<JsonRpcResult<Transaction>>(serde_json::from_str(include_str!("../../testdata/contract_erc20_tx.json")).unwrap())
                .unwrap()
                .result;
        let sc_erc20_receipt = serde_json::from_value::<JsonRpcResult<TransactionReciept>>(
            serde_json::from_str(include_str!("../../testdata/contract_erc20_receipt.json")).unwrap(),
        )
        .unwrap()
        .result;

        let transaction = EthereumMapper::map_transaction(Chain::Arbitrum, &sc_erc20_tx, &sc_erc20_receipt, None, &BigUint::from(1735671600u64), None).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(
            transaction.asset_id,
            AssetId::from_token(Chain::Arbitrum, "0xaf88d065e77c8cC2239327C5EDb3A432268e5831")
        );
        assert_eq!(transaction.from, "0x58E1b0E63C905D5982324FCd9108582623b8132e");
        assert_eq!(transaction.to, "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC");
        assert_eq!(transaction.value, "930678651");
    }

    #[test]
    fn test_native_transfer_high_gas_limit() {
        let transaction =
            serde_json::from_value::<JsonRpcResult<Transaction>>(serde_json::from_str(include_str!("../../testdata/transfer_high_gas_limit.json")).unwrap())
                .unwrap()
                .result;
        let transaction_receipt = serde_json::from_value::<JsonRpcResult<TransactionReciept>>(
            serde_json::from_str(include_str!("../../testdata/transfer_high_gas_limit_receipt.json")).unwrap(),
        )
        .unwrap()
        .result;

        let result = EthereumMapper::map_transaction(Chain::Ethereum, &transaction, &transaction_receipt, None, &BigUint::from(1735671600u64), None);

        assert!(result.is_some());
        let tx = result.unwrap();
        assert_eq!(tx.transaction_type, TransactionType::Transfer);
        assert_eq!(tx.asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(tx.from, "0x8D25Fb438C6efCD08679ffA82766869B50E24608");
        assert_eq!(tx.to, "0x0700572b54ccA24Dad0eD4Cdad2c3d3ab6dB652a");
        assert_eq!(tx.value, "2739900000000000000");
        assert_eq!(tx.hash, "0x0c0626172dbba6984a2e95b3abf1caba39cf11d3c9bc99d7de9ac814671c0cb1");
    }

    #[test]
    fn test_map_smartchain_staking_transaction() {
        let transaction = Transaction {
            hash: "0x21442c7c30a6c1d6be3b5681275adb1f1402cef066579c4f53ec4f1c8c056ab0".to_string(),
            from: "0xf1a3687303606a6fd48179ce503164cdcbabeab6".to_string(),
            to: Some("0x0000000000000000000000000000000000002002".to_string()),
            value: BigUint::parse_bytes(b"1158e460913d00000", 16).unwrap(),
            gas: 280395,
            input: "0x982ef0a7000000000000000000000000d34403249b2d82aaddb14e778422c966265e5fb50000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            block_number: BigUint::from(0x1234u32),
        };

        let receipt = TransactionReciept {
            gas_used: BigUint::from(100000u32),
            effective_gas_price: BigUint::from(20000000000u64),
            l1_fee: None,
            logs: vec![Log {
                address: "0x0000000000000000000000000000000000002002".to_string(),
                topics: vec![
                    "0x24d7bda8602b916d64417f0dbfe2e2e88ec9b1157bd9f596dfdb91ba26624e04".to_string(), // Delegated event
                    "0x000000000000000000000000d34403249B2d82AAdDB14e778422c966265e5Fb5".to_string(), // operatorAddress
                    "0x000000000000000000000000f1a3687303606a6fD48179Ce503164CDcBAbeaB6".to_string(), // delegator
                ],
                data: "0x00000000000000000000000000000000000000000000000d5cc0065cf2d900aa0000000000000000000000000000000000000000000000001158e460913d00000"
                    .to_string(),
            }],
            status: "0x1".to_string(),
            block_number: BigUint::from(0x1234u32),
        };

        let result = EthereumMapper::map_transaction(Chain::SmartChain, &transaction, &receipt, None, &BigUint::from(1735671600u64), None);

        assert!(result.is_some());
        let tx = result.unwrap();
        assert_eq!(tx.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(tx.from, "0xf1a3687303606a6fD48179Ce503164CDcBAbeaB6");
        assert_eq!(tx.to, "0xd34403249B2d82AAdDB14e778422c966265e5Fb5");
        assert_eq!(tx.contract.unwrap(), "0x0000000000000000000000000000000000002002");
        assert!(tx.metadata.is_none());
    }
}
