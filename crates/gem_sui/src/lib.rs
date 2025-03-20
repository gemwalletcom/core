pub mod jsonrpc;
pub mod model;

use anyhow::{anyhow, Error};
use base64::{engine::general_purpose, Engine as _};
use model::*;
use std::str::FromStr;
use sui_transaction_builder::{Function, TransactionBuilder as ProgrammableTransactionBuilder};
use sui_types::{
    transaction::{
        serialization::{
            input_argument::{CallArg, ObjectArg},
            transaction::TransactionData,
        },
        Argument,
    },
    Address as SuiAddress, Identifier, ObjectId as ObjectID, ObjectReference as ObjectRef,
};

static SUI_SYSTEM_ID: &str = "sui_system";
static SUI_REQUEST_ADD_STAKE: &str = "request_add_stake";
static SUI_REQUEST_WITHDRAW_STAKE: &str = "request_withdraw_stake";
static SUI_SYSTEM_ADDRESS: u8 = 0x3;

pub static SUI_SYSTEM_STATE_OBJECT_ID: u8 = 0x5;
pub static SUI_CLOCK_OBJECT_ID: u8 = 0x6;
pub static SUI_FRAMEWORK_PACKAGE_ID: u8 = 0x2;

pub static SUI_COIN_TYPE: &str = "0x2::sui::SUI";
pub static SUI_COIN_TYPE_FULL: &str = "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI";
pub static EMPTY_ADDRESS: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
pub static STORAGE_FEE_UNIT: u64 = 76; // https://blog.sui.io/storage-fees-explained

pub fn encode_transfer(input: &TransferInput) -> Result<TxOutput, Error> {
    if let Some(err) = validate_enough_balance(&input.coins, input.amount) {
        return Err(err);
    }

    let coin_refs: Vec<ObjectRef> = input.coins.iter().map(|x| x.object.to_ref()).collect();

    let sender = SuiAddress::from_str(&input.sender)?;
    let recipient = SuiAddress::from_str(&input.recipient)?;

    let mut ptb = ProgrammableTransactionBuilder::new();
    if input.send_max {
        ptb.pay_all_sui(recipient);
    } else {
        ptb.pay_sui(vec![recipient], vec![input.amount])?;
    }

    let tx_data = TransactionData::new_programmable(sender, coin_refs, ptb.finish(), input.gas.budget, input.gas.price);
    TxOutput::from_tx_data(&tx_data)
}

pub fn encode_token_transfer(input: &TokenTransferInput) -> Result<TxOutput, Error> {
    if let Some(err) = validate_enough_balance(&input.tokens, input.amount) {
        return Err(err);
    }
    let mut ptb = ProgrammableTransactionBuilder::new();
    let sender = SuiAddress::from_str(&input.sender)?;
    let recipient = SuiAddress::from_str(&input.recipient)?;
    let coin_refs: Vec<ObjectRef> = input.tokens.iter().map(|x| x.object.to_ref()).collect();
    let gas_coin = input.gas_coin.object.to_ref();

    ptb.pay(coin_refs, vec![recipient], vec![input.amount])?;
    let tx_data = TransactionData::new_programmable(sender, vec![gas_coin], ptb.finish(), input.gas.budget, input.gas.price);

    TxOutput::from_tx_data(&tx_data)
}

pub fn encode_split_and_stake(input: &StakeInput) -> Result<TxOutput, Error> {
    if let Some(err) = validate_enough_balance(&input.coins, input.stake_amount) {
        return Err(err);
    }

    let stake_chain = primitives::StakeChain::Sui;
    if input.stake_amount < stake_chain.get_min_stake_amount() {
        return Err(anyhow!("stake amount is too small"));
    }

    let coin_refs: Vec<ObjectRef> = input.coins.iter().map(|x| x.object.to_ref()).collect();
    let sender = SuiAddress::from_str(&input.sender)?;
    let validator = SuiAddress::from_str(&input.validator)?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    // split new coin to stake
    let split_stake_amount = CallArg::Pure(input.stake_amount.to_le_bytes().to_vec());
    let Argument::Result(idx) = ptb.split_coins(Argument::Gas, vec![split_stake_amount]) else {
        panic!("command should always give a Argument::Result")
    };

    // move call request_add_stake
    let function = Function::new(
        SuiAddress::from_u8(SUI_SYSTEM_ADDRESS),
        Identifier::new(SUI_SYSTEM_ID).unwrap(),
        Identifier::new(SUI_REQUEST_ADD_STAKE).unwrap(),
        vec![],
    );
    ptb.move_call(
        function,
        vec![ptb.obj(sui_system_state_object())?, Argument::NestedResult(idx, 0), ptb.pure(validator)?],
    );

    let tx_data = TransactionData::new_programmable(sender, coin_refs, ptb.finish(), input.gas.budget, input.gas.price);

    TxOutput::from_tx_data(&tx_data)
}

pub fn encode_unstake(input: &UnstakeInput) -> Result<TxOutput, Error> {
    let mut ptb = ProgrammableTransactionBuilder::new();

    let sender = SuiAddress::from_str(&input.sender)?;
    let staked_sui = ObjectArg::ImmutableOrOwned(input.staked_sui.to_ref());
    let gas_coin = input.gas_coin.object.to_ref();

    let function = Function::new(
        SuiAddress::from_u8(SUI_SYSTEM_ADDRESS),
        Identifier::new(SUI_SYSTEM_ID).unwrap(),
        Identifier::new(SUI_REQUEST_WITHDRAW_STAKE).unwrap(),
        vec![],
    );
    ptb.move_call(function, vec![ptb.obj(sui_system_state_object())?, staked_sui]);

    let tx_data = TransactionData::new_programmable(sender, vec![gas_coin], ptb.finish(), input.gas.budget, input.gas.price);

    TxOutput::from_tx_data(&tx_data)
}

pub(crate) fn decode_transaction(_tx: &str) -> Result<TransactionData, Error> {
    let bytes = general_purpose::STANDARD.decode(_tx)?;
    let tx_data = bcs::from_bytes::<TransactionData>(&bytes)?;
    Ok(tx_data)
}

pub fn validate_and_hash(encoded: &str) -> Result<TxOutput, Error> {
    let tx_data = decode_transaction(encoded)?;
    TxOutput::from_tx_data(&tx_data)
}

pub fn sui_system_state_object() -> ObjectArg {
    ObjectArg::Shared {
        object_id: ObjectID::from_u8(SUI_SYSTEM_STATE_OBJECT_ID),
        initial_shared_version: 1,
        mutable: true,
    }
}

pub fn sui_clock_object() -> ObjectArg {
    ObjectArg::Shared {
        object_id: ObjectID::from_u8(SUI_CLOCK_OBJECT_ID),
        initial_shared_version: 1,
        mutable: false,
    }
}

pub fn validate_enough_balance(coins: &[Coin], amount: u64) -> Option<Error> {
    if coins.is_empty() {
        return Some(anyhow!("coins list is empty"));
    }

    let total_amount: u64 = coins.iter().map(|x| x.balance).sum();
    if total_amount < amount {
        return Some(anyhow!(format!("total amount ({}) is less than amount to send ({})", total_amount, amount),));
    }
    None
}

#[cfg(test)]
mod tests {
    use sui_types::transaction::TransactionKind;

    use super::*;

    #[test]
    fn test_encode_transfer() {
        let input = TransferInput {
            sender: "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".into(),
            recipient: "0xe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2".into(),
            amount: 8993996480,
            coins: vec![Coin {
                coin_type: SUI_COIN_TYPE.into(),
                balance: 8994756360,
                object: Object {
                    object_id: "0x9f258c85566d977b4c99bb6019560ba99c796e71291269d8f9f3cc9d9f37db46".into(),
                    digest: "GoAwPNYEBKyAgzmQgnxW23bdhnHaLXcqT3o1nEZo4KPM".into(),
                    version: 68419468,
                },
            }],
            send_max: true,
            gas: Gas {
                budget: 25_000_000,
                price: 750,
            },
        };

        let output = encode_transfer(&input).unwrap();
        let b64_encoded = general_purpose::STANDARD.encode(output.tx_data);
        assert_eq!(b64_encoded, "AAABACDmr4D+GwtC/NlnYuXHD16Nrjn48O4PEYysDVW3TiknwgEBAQABAACpvQST+b0feSpK7cH5nVRTWnWkbDj9VqjyxrfI11gXoQGfJYyFVm2Xe0yZu2AZVgupnHlucSkSadj588ydnzfbRoz/EwQAAAAAIOqzQffiRRpexyiDEtyjm40KqFMf60ohK5jCJ0z3+Lqwqb0Ek/m9H3kqSu3B+Z1UU1p1pGw4/Vao8sa3yNdYF6HuAgAAAAAAAEB4fQEAAAAAAA==");
    }

    #[test]
    fn test_encode_token_transfer() {
        let input = TokenTransferInput {
            sender: "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".into(),
            recipient: "0xe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2".into(),
            amount: 2400000000,
            tokens: vec![
                Coin {
                    coin_type: "0xe4239cd951f6c53d9c41e25270d80d31f925ad1655e5ba5b543843d4a66975ee::SUIP::SUIP".into(),
                    balance: 1400000000,
                    object: Object {
                        object_id: "0x1a6b6023d363f5dcad026f83ddb9bb0f987c941f10db2ab86571711a1a9a1ee6".into(),
                        digest: "CCFDRi15n2mhBVGAoa594VynBKgSRbgZQZgjT4wxFu7B".into(),
                        version: 67155000,
                    },
                },
                Coin {
                    coin_type: "0xe4239cd951f6c53d9c41e25270d80d31f925ad1655e5ba5b543843d4a66975ee::SUIP::SUIP".into(),
                    balance: 1000000000,
                    object: Object {
                        object_id: "0x2fd950f33ecdf9e5d797ca3130811e7a973d4c1da5427ac0c910a8c5f6e8b72d".into(),
                        digest: "7CsXhia2TGqy7bXnxH4WLbkzYJBPvCnNVuLvzByvLsRh".into(),
                        version: 67154999,
                    },
                },
            ],
            gas: Gas {
                budget: 25_000_000,
                price: 750,
            },
            gas_coin: Coin {
                coin_type: SUI_COIN_TYPE.into(),
                balance: 100000000,
                object: Object {
                    object_id: "0x890f8c604c7cb5cc194dbf4953ad3dbebd81ef7526be351d3514cc3cc26c9c1d".into(),
                    digest: "3a2sHuj9pJg7RHub4w9EPyBtpxVfHzk52M91HErwMQ4J".into(),
                    version: 69035764,
                },
            },
        };

        let output = encode_token_transfer(&input).unwrap();
        let b64_encoded = general_purpose::STANDARD.encode(output.tx_data);
        assert_eq!(b64_encoded, "AAAEAQAaa2Aj02P13K0Cb4PdubsPmHyUHxDbKrhlcXEaGpoe5ji0AAQAAAAAIKZSBGYgBc5PwYeX01SAZHnJYxA3pJRvrUZmR7ToQZTWAQAv2VDzPs355deXyjEwgR56lz1MHaVCesDJEKjF9ui3LTe0AAQAAAAAIFwwpOhb+onitRHRqj+wsEA0nNO2KqqOt8/IVbcC0O7oAAgAGA2PAAAAAAAg5q+A/hsLQvzZZ2Llxw9eja45+PDuDxGMrA1Vt04pJ8IDAwEAAAEBAQACAQAAAQECAAEBAwEAAAABAwCpvQST+b0feSpK7cH5nVRTWnWkbDj9VqjyxrfI11gXoQGJD4xgTHy1zBlNv0lTrT2+vYHvdSa+NR01FMw8wmycHfRmHQQAAAAAICYtptS+v/0HkfChzkJo0QzRDQxhli84CM3mMV/dqUBbqb0Ek/m9H3kqSu3B+Z1UU1p1pGw4/Vao8sa3yNdYF6HuAgAAAAAAAEB4fQEAAAAAAA==");
    }

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

    #[test]
    fn test_decode_transaction() {
        let tx = "AAAPAAhkx5NBAAAAAAAIKUO8sgMAAAAAAQAAAQAAAQAACGTHk0EAAAAAAQFexM/GvrUlJRacMqd+FsKIt7/Lm4mCielL8xCFcLPvpBbjZwAAAAAAAQEB2qRikmMsPE2PMfI+oPmzaij/NnfpaEmA5EOEA6Z6PY8uBRgAAAAAAAABAYBJ0AkRYmmsBO4UIGt6/YtktYASefhUAe5LOXefgJE0zicvAAAAAAABAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABgEAAAAAAAAAAAEB8ZTZsbytly5Fp91n3Umz7h4zV6AKUIUMUs1Ru0UOE7QXwmUAAAAAAAABASjkmd/16GSi6v5HYmmk9QNfHBbzONp74YsQNJmr8nHO7fIyAAAAAAABAQHwxA1nsHgADhgDIzTDMlxHueyfPZrkEovoINVGY9FOO+/yMgAAAAAAAQEBNdNbDlsXdZPYw6gBRiSFVy/DCGHmzpalWvbcRzBwknju8jIAAAAAAAAAIJP2W4wWwmM0O79mz5+O72nLHbyS0T8MMxsNyut2tKq2BgIAAQEAAADcFXIbqoK6ZIItWFpzSaFQj3bZSugOiZsG5INpwld1Dghzd2FwX2NhcBFvYnRhaW5fcm91dGVyX2NhcAIHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIDc3VpA1NVSQAH5COc2VH2xT2cQeJScNgNMfklrRZV5bpbVDhD1KZpde4EU1VJUARTVUlQAAUCAAABAQABAgABAwABBAAA3BVyG6qCumSCLVhac0mhUI922UroDombBuSDacJXdQ4Ic3dhcF9jYXANaW5pdGlhdGVfcGF0aAEHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIDc3VpA1NVSQACAgEAAQUAAB7GqMWsC4uXwofNNLn8apS1OgfJMKhQWVJnncjUs3gKBnJvdXRlchBzd2FwX2JfdG9fYV9ieV9iAwcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgNzdWkDU1VJAAfkI5zZUfbFPZxB4lJw2A0x+SWtFlXlultUOEPUpml17gRTVUlQBFNVSVAABwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACA3N1aQNTVUkABgEGAAIBAAEHAAEIAAICAAEJAADcFXIbqoK6ZIItWFpzSaFQj3bZSugOiZsG5INpwld1Dghzd2FwX2NhcBFyZXR1cm5fcm91dGVyX2NhcAIHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIDc3VpA1NVSQAH5COc2VH2xT2cQeJScNgNMfklrRZV5bpbVDhD1KZpde4EU1VJUARTVUlQAAYCAQACAwABCgABCwABDAABDQABAQIDAAEOAJP2W4wWwmM0O79mz5+O72nLHbyS0T8MMxsNyut2tKq2AQAX1Cs2B1S8591qpdZjDUOB/CBDy2V8/6tqhBbwbdyxj734BAAAAAAg6yrtiW5R0TC68GDMmZye6U+KDjfZlq21n3bztRGzXjuT9luMFsJjNDu/Zs+fju9pyx28ktE/DDMbDcrrdrSqtu4CAAAAAAAA3P9fAAAAAAAA";
        let tx_data = decode_transaction(tx).unwrap();
        let TransactionData::V1(data) = tx_data;

        assert_eq!(data.sender.to_string(), "0x93f65b8c16c263343bbf66cf9f8eef69cb1dbc92d13f0c331b0dcaeb76b4aab6");
        match data.kind {
            TransactionKind::ProgrammableTransaction(programmable) => {
                assert_eq!(programmable.commands.len(), 6);
            }
            _ => panic!("wrong kind"),
        }

        let output = validate_and_hash(tx).unwrap();
        assert_eq!(hex::encode(output.hash), "883f6f54145fdaf357e3d404a8353b1f6eda265bc2b28ec8178631e092c24e3b");
    }
}
