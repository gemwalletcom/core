use super::model::*;
use super::{address_from_u8, object_id_from_u8};

use crate::{SUI_REQUEST_ADD_STAKE, SUI_REQUEST_WITHDRAW_STAKE, SUI_SYSTEM_ADDRESS, SUI_SYSTEM_ID, SUI_SYSTEM_STATE_OBJECT_ID};
use anyhow::{anyhow, Error};

use std::str::FromStr;
use sui_transaction_builder::{unresolved::Input, Function, Serialized, TransactionBuilder};
use sui_types::{Address, Identifier};

pub fn encode_split_and_stake(input: &StakeInput) -> Result<TxOutput, Error> {
    if let Some(err) = super::validate_enough_balance(&input.coins, input.stake_amount) {
        return Err(err);
    }

    let stake_chain = primitives::StakeChain::Sui;
    if input.stake_amount < stake_chain.get_min_stake_amount() {
        return Err(anyhow!("stake amount is too small"));
    }

    let sender = Address::from_str(&input.sender)?;
    let validator = Address::from_str(&input.validator)?;

    let mut ptb = TransactionBuilder::new();

    // split new coin to stake
    let stake_amount = ptb.input(Serialized(&input.stake_amount));
    let split_result = ptb.split_coins(ptb.gas(), vec![stake_amount]);

    // move call request_add_stake
    let function = Function::new(
        address_from_u8(SUI_SYSTEM_ADDRESS),
        Identifier::new(SUI_SYSTEM_ID).unwrap(),
        Identifier::new(SUI_REQUEST_ADD_STAKE).unwrap(),
        vec![],
    );

    // Get system state object
    let sys_state = ptb.input(Input::shared(object_id_from_u8(SUI_SYSTEM_STATE_OBJECT_ID), 1, true));
    let validator_argument = ptb.input(Serialized(&validator));

    ptb.set_sender(sender);
    ptb.set_gas_budget(input.gas.budget);
    ptb.set_gas_price(input.gas.price);
    ptb.move_call(function, vec![sys_state, split_result, validator_argument]);

    let tx_data = ptb.finish()?;

    TxOutput::from_tx_data(&tx_data)
}

pub fn encode_unstake(input: &UnstakeInput) -> Result<TxOutput, Error> {
    let mut ptb = TransactionBuilder::new();

    let sender = Address::from_str(&input.sender)?;
    let staked_sui = ptb.input(Input::owned(
        input.staked_sui.object_id.parse().unwrap(),
        input.staked_sui.version,
        input.staked_sui.digest.parse().unwrap(),
    ));
    let gas_coin = Input::immutable(
        input.gas_coin.object.object_id.parse().unwrap(),
        input.gas_coin.object.version,
        input.gas_coin.object.digest.parse().unwrap(),
    );
    let function = Function::new(
        address_from_u8(SUI_SYSTEM_ADDRESS),
        Identifier::new(SUI_SYSTEM_ID).unwrap(),
        Identifier::new(SUI_REQUEST_WITHDRAW_STAKE).unwrap(),
        vec![],
    );

    // Get system state object
    let sys_state = ptb.input(Input::shared(object_id_from_u8(SUI_SYSTEM_STATE_OBJECT_ID), 1, true));

    ptb.move_call(function, vec![sys_state, staked_sui]);

    ptb.set_sender(sender);
    ptb.set_gas_budget(input.gas.budget);
    ptb.set_gas_price(input.gas.price);
    ptb.add_gas_objects(vec![gas_coin]);
    let tx_data = ptb.finish()?;

    TxOutput::from_tx_data(&tx_data)
}

#[cfg(test)]
mod tests {
    use crate::SUI_COIN_TYPE;

    use super::*;
    use base64::{engine::general_purpose, Engine as _};

    #[test]
    fn test_encode_split_stake() {
        let mut input = StakeInput {
            sender: "0xe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2".into(),
            validator: "0x61953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab".into(),
            stake_amount: 1_000_000_000,
            gas: Gas {
                budget: 20_000_000,
                price: 750,
            },
            coins: vec![Coin {
                coin_type: SUI_COIN_TYPE.into(),
                balance: 10990277896,
                object: Object {
                    object_id: "0x36b8380aa7531d73723657d73a114cfafedf89dc8c76b6752f6daef17e43dda2".into(),
                    version: 0x3f4d8e5,
                    digest: "HdfF7hswRuvbXbEXjGjmUCt7gLybhvbPvvK8zZbCqyD8".into(),
                },
            }],
        };
        let data = encode_split_and_stake(&input).unwrap();

        assert_eq!(hex::encode(data.tx_data), "000003000800ca9a3b0000000001010000000000000000000000000000000000000000000000000000000000000005010000000000000001002061953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab020200010100000000000000000000000000000000000000000000000000000000000000000000030a7375695f73797374656d11726571756573745f6164645f7374616b6500030101000300000000010200e6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c20136b8380aa7531d73723657d73a114cfafedf89dc8c76b6752f6daef17e43dda2e5d8f4030000000020f71f24516bc04cbf877d42faf459514448c8de6cff48faa44b3eef3b26782e8fe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2ee02000000000000002d31010000000000");
        assert_eq!(hex::encode(data.hash), "66be75b0f86ca3a9f24380adc8d8336d8921d5dbdc78f1b3c24c7d6842ce5911");

        input.stake_amount = 100_000_000;
        let result = encode_split_and_stake(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_unstake() {
        let input = UnstakeInput {
            sender: "0xe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2".into(),
            staked_sui: Object {
                object_id: "0xc8c1666ae68f46b609d40bb51d1ec23dc2e0560f986aae878643b6d215549fcf".into(),
                digest: "CU86BjXRF1XHFRjKBasCYEuaQxhHuyGBpuoJyqsrYoX5".into(),
                version: 64195796,
            },
            gas: Gas {
                budget: 25_000_000,
                price: 750,
            },
            gas_coin: Coin {
                coin_type: SUI_COIN_TYPE.into(),
                balance: 631668351,
                object: Object {
                    object_id: "0x36b8380aa7531d73723657d73a114cfafedf89dc8c76b6752f6daef17e43dda2".into(),
                    version: 68755407,
                    digest: "FHbvG5i7f8o2VrKpXnqGFHNvGxG7BBKREea5avdPN7ke".into(),
                },
            },
        };
        let output = encode_unstake(&input).unwrap();
        let b64_encoded = general_purpose::STANDARD.encode(output.tx_data);
        assert_eq!(b64_encoded, "AAACAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABQEAAAAAAAAAAQEAyMFmauaPRrYJ1Au1HR7CPcLgVg+Yaq6HhkO20hVUn8/UjNMDAAAAACCqY0EI6P2Lzjy4eh4ckx6iz/5S78vLxiOulRCcAgwEcAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMKc3VpX3N5c3RlbRZyZXF1ZXN0X3dpdGhkcmF3X3N0YWtlAAIBAAABAQDmr4D+GwtC/NlnYuXHD16Nrjn48O4PEYysDVW3TiknwgE2uDgKp1Mdc3I2V9c6EUz6/t+J3Ix2tnUvba7xfkPdos8fGQQAAAAAINREZGL0SD9y5n7te55Ju78nQ/PVWycQpwYPm4+JrWej5q+A/hsLQvzZZ2Llxw9eja45+PDuDxGMrA1Vt04pJ8LuAgAAAAAAAEB4fQEAAAAAAA==");
    }
}
