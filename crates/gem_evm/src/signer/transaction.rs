use crate::contracts::IERC20;
use alloy_consensus::TxEip1559;
use alloy_primitives::{Address, Bytes, TxKind, U256};
use alloy_sol_types::SolCall;
use primitives::AssetId;
use std::error::Error;
use std::str::FromStr;

pub fn create_transfer_tx(
    asset_id: &AssetId,
    recipient: &str,
    amount: &str,
    nonce: u64,
    chain_id: u64,
    max_fee_per_gas: u128,
    max_priority_fee_per_gas: u128,
    gas_limit: u64,
) -> Result<TxEip1559, Box<dyn Error + Send + Sync>> {
    let amount_u256 = U256::from_str(amount)?;
    let recipient_address = Address::from_str(recipient)?;

    let (to, value, input) = if let Some(token_address) = &asset_id.token_id {
        let contract_address = Address::from_str(token_address)?;
        let transfer_data = encode_erc20_transfer(recipient, amount_u256)?;
        (contract_address, U256::ZERO, transfer_data)
    } else {
        (recipient_address, amount_u256, Bytes::new())
    };

    Ok(TxEip1559 {
        chain_id,
        nonce,
        gas_limit,
        max_fee_per_gas,
        max_priority_fee_per_gas,
        to: TxKind::Call(to),
        value,
        access_list: Default::default(),
        input,
    })
}

fn encode_erc20_transfer(to: &str, amount: U256) -> Result<Bytes, Box<dyn Error + Send + Sync>> {
    let to_address = Address::from_str(to)?;
    let call = IERC20::transferCall { to: to_address, value: amount };
    Ok(Bytes::from(call.abi_encode()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_create_native_transfer() {
        let asset_id = AssetId::from_chain(Chain::SmartChain);
        let tx = create_transfer_tx(
            &asset_id,
            "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4",
            "1000000000000000000",
            0,
            56,
            5_000_000_000,
            1_000_000_000,
            21000,
        )
        .unwrap();

        assert_eq!(tx.value, U256::from(1_000_000_000_000_000_000u128));
        assert!(tx.input.is_empty());
    }

    #[test]
    fn test_create_token_transfer() {
        let asset_id = AssetId::from_token(Chain::SmartChain, "0x55d398326f99059fF775485246999027B3197955");
        let tx = create_transfer_tx(
            &asset_id,
            "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4",
            "1000000",
            0,
            56,
            5_000_000_000,
            1_000_000_000,
            65000,
        )
        .unwrap();

        assert_eq!(tx.value, U256::ZERO);
        assert!(!tx.input.is_empty());
    }
}
