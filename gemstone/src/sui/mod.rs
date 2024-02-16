pub mod model;

use anyhow::anyhow;
use model::{SuiStakeInput, SuiTransferInput, SuiTxOutput, SuiUnstakeInput};
use std::str::FromStr;
use sui_types::{
    base_types::{ObjectID, ObjectRef, SequenceNumber, SuiAddress},
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Argument, Command, ObjectArg, TransactionData},
    Identifier,
};

static SUI_SYSTEM_ID: &str = "sui_system";
static SUI_REQUEST_ADD_STAKE: &str = "request_add_stake";
static SUI_REQUEST_WITHDRAW_STAKE: &str = "request_withdraw_stake";
static SUI_SYSTEM_ADDRESS: u8 = 0x3;
static SUI_SYSTEM_STATE_OBJECT_ID: u8 = 0x5;

pub fn encode_transfer(input: &SuiTransferInput) -> Result<SuiTxOutput, anyhow::Error> {
    if input.coins.is_empty() {
        return Err(anyhow!("empty coins list!"));
    }

    let total_amount: u64 = input.coins.iter().map(|x| x.balance).sum();
    if !input.send_max && total_amount < input.amount {
        return Err(anyhow!(format!(
            "total amount ({}) is less than input amount ({})",
            total_amount, input.amount
        ),));
    }

    let coin_refs: Vec<ObjectRef> = input
        .coins
        .iter()
        .map(|x| x.object_ref.to_tuple())
        .collect();

    let sender = SuiAddress::from_str(&input.sender)?;
    let recipient = SuiAddress::from_str(&input.recipient)?;

    let mut ptb = ProgrammableTransactionBuilder::new();
    if input.send_max {
        ptb.pay_all_sui(recipient);
    } else {
        ptb.pay_sui(vec![recipient], vec![input.amount])?;
    }
    let builder = ptb.finish();
    let tx_data = TransactionData::new_programmable(
        sender,
        coin_refs,
        builder,
        input.gas.budget,
        input.gas.price,
    );
    SuiTxOutput::from_tx_data(&tx_data)
}

pub fn encode_split_and_stake(input: &SuiStakeInput) -> Result<SuiTxOutput, anyhow::Error> {
    if input.coins.is_empty() {
        return Err(anyhow!("empty coins list!"));
    }

    let total_amount: u64 = input.coins.iter().map(|x| x.balance).sum();
    if total_amount < input.stake_amount {
        return Err(anyhow!(format!(
            "total amount ({}) is less than stake amount ({})",
            total_amount, input.stake_amount
        ),));
    }

    if total_amount - input.gas.budget < input.stake_amount {
        return Err(anyhow!(format!(
            "total amount ({}) is less than stake amount + gas ({}+{})",
            total_amount, input.stake_amount, input.gas.budget
        )));
    }

    let coin_refs: Vec<ObjectRef> = input
        .coins
        .iter()
        .map(|x| x.object_ref.to_tuple())
        .collect();
    let sender = SuiAddress::from_str(&input.sender)?;
    let validator = SuiAddress::from_str(&input.validator)?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    // split new coin to stake
    let split_stake_amount = ptb.pure(input.stake_amount)?;
    let Argument::Result(idx) = ptb.command(Command::SplitCoins(
        Argument::GasCoin,
        vec![split_stake_amount],
    )) else {
        panic!("command should always give a Argument::Result")
    };

    // move call request_add_stake
    let move_call = Command::move_call(
        ObjectID::from_single_byte(SUI_SYSTEM_ADDRESS),
        Identifier::new(SUI_SYSTEM_ID).unwrap(),
        Identifier::new(SUI_REQUEST_ADD_STAKE).unwrap(),
        vec![],
        vec![
            ptb.obj(sui_system_state_object())?,
            Argument::NestedResult(idx, 0),
            ptb.pure(validator)?,
        ],
    );
    ptb.command(move_call);

    let tx_data = TransactionData::new_programmable(
        sender,
        coin_refs,
        ptb.finish(),
        input.gas.budget,
        input.gas.price,
    );

    SuiTxOutput::from_tx_data(&tx_data)
}

pub fn encode_unstake(input: &SuiUnstakeInput) -> Result<SuiTxOutput, anyhow::Error> {
    let mut ptb = ProgrammableTransactionBuilder::new();

    let sender = SuiAddress::from_str(&input.sender)?;
    let staked_sui = ObjectArg::ImmOrOwnedObject(input.staked_sui.to_tuple());
    let gas_coin = input.gas_coin.object_ref.to_tuple();

    let move_call = Command::move_call(
        ObjectID::from_single_byte(SUI_SYSTEM_ADDRESS),
        Identifier::new(SUI_SYSTEM_ID).unwrap(),
        Identifier::new(SUI_REQUEST_WITHDRAW_STAKE).unwrap(),
        vec![],
        vec![ptb.obj(sui_system_state_object())?, ptb.obj(staked_sui)?],
    );
    ptb.command(move_call);

    let tx_data = TransactionData::new_programmable(
        sender,
        vec![gas_coin],
        ptb.finish(),
        input.gas.budget,
        input.gas.price,
    );

    SuiTxOutput::from_tx_data(&tx_data)
}

pub fn sui_system_state_object() -> ObjectArg {
    ObjectArg::SharedObject {
        id: ObjectID::from_single_byte(SUI_SYSTEM_STATE_OBJECT_ID),
        initial_shared_version: SequenceNumber::from_u64(1),
        mutable: true,
    }
}

#[cfg(test)]
mod tests {
    use self::model::*;
    use super::*;
    use base64::{engine::general_purpose, Engine as _};

    #[test]
    fn test_encode_transfer() {
        let input = SuiTransferInput {
            sender: "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".into(),
            recipient: "0xe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2".into(),
            amount: 8993996480,
            coins: vec![SuiCoin {
                coin_type: "0x2::sui::SUI".into(),
                balance: 8994756360,
                object_ref: SuiObjectRef {
                    object_id: "0x9f258c85566d977b4c99bb6019560ba99c796e71291269d8f9f3cc9d9f37db46"
                        .into(),
                    digest: "GoAwPNYEBKyAgzmQgnxW23bdhnHaLXcqT3o1nEZo4KPM".into(),
                    version: 68419468,
                },
            }],
            send_max: true,
            gas: SuiGas {
                budget: 25_000_000,
                price: 750,
            },
        };

        let output = encode_transfer(&input).unwrap();
        let b64_encoded = general_purpose::STANDARD.encode(output.tx_data);
        assert_eq!(b64_encoded, "AAABACDmr4D+GwtC/NlnYuXHD16Nrjn48O4PEYysDVW3TiknwgEBAQABAACpvQST+b0feSpK7cH5nVRTWnWkbDj9VqjyxrfI11gXoQGfJYyFVm2Xe0yZu2AZVgupnHlucSkSadj588ydnzfbRoz/EwQAAAAAIOqzQffiRRpexyiDEtyjm40KqFMf60ohK5jCJ0z3+Lqwqb0Ek/m9H3kqSu3B+Z1UU1p1pGw4/Vao8sa3yNdYF6HuAgAAAAAAAEB4fQEAAAAAAA==");
    }

    #[test]
    fn test_encode_split_stake() {
        let input = SuiStakeInput {
            sender: "0xe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2".into(),
            validator: "0x61953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab".into(),
            stake_amount: 1_000_000_000,
            gas: SuiGas {
                budget: 20_000_000,
                price: 750,
            },
            coins: vec![SuiCoin {
                coin_type: "0x2::sui::SUI".into(),
                balance: 10990277896,
                object_ref: SuiObjectRef {
                    object_id: "0x36b8380aa7531d73723657d73a114cfafedf89dc8c76b6752f6daef17e43dda2"
                        .into(),
                    version: 0x3f4d8e5,
                    digest: "HdfF7hswRuvbXbEXjGjmUCt7gLybhvbPvvK8zZbCqyD8".into(),
                },
            }],
        };
        let data = encode_split_and_stake(&input).unwrap();

        assert_eq!(hex::encode(data.tx_data), "000003000800ca9a3b0000000001010000000000000000000000000000000000000000000000000000000000000005010000000000000001002061953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab020200010100000000000000000000000000000000000000000000000000000000000000000000030a7375695f73797374656d11726571756573745f6164645f7374616b6500030101000300000000010200e6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c20136b8380aa7531d73723657d73a114cfafedf89dc8c76b6752f6daef17e43dda2e5d8f4030000000020f71f24516bc04cbf877d42faf459514448c8de6cff48faa44b3eef3b26782e8fe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2ee02000000000000002d31010000000000");
        assert_eq!(
            hex::encode(data.hash),
            "66be75b0f86ca3a9f24380adc8d8336d8921d5dbdc78f1b3c24c7d6842ce5911"
        );
    }

    #[test]
    fn test_unstake() {
        // FIXME add test
    }
}
