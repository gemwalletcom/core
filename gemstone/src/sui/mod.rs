#[derive(uniffi::Record)]
pub struct SuiCoin {
    pub coin_type: String,
    pub coin_object_id: String,
    pub version: u64,
    pub digest: String,
    pub balance: u64,
}

#[derive(uniffi::Record)]
pub struct SuiStakeInput {
    pub sender: String,
    pub validator: String,
    pub stake_amount: u64,
    pub gas_budget: u64,
    pub gas_price: u64,
    pub coin: SuiCoin,
}

use bcs;
use std::str::FromStr;
use sui_sdk::types::{
    base_types::{ObjectID, ObjectRef, SequenceNumber, SuiAddress},
    crypto::{Ed25519SuiSignature, Signature, ToFromBytes},
    digests::ObjectDigest,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Argument, Command, ObjectArg, Transaction, TransactionData},
    Identifier,
};

static SUI_SYSTEM_ID: &str = "sui_system";
static SUI_REQUEST_ADD_STAKE: &str = "request_add_stake";
static SUI_SYSTEM_ADDRESS: u8 = 0x3;
static SUI_SYSTEM_STATE_OBJECT_ID: u8 = 0x5;

pub fn encode_split_and_stake(input: &SuiStakeInput) -> Result<Vec<u8>, anyhow::Error> {
    let object_id = ObjectID::from_hex_literal(&input.coin.coin_object_id)?;
    let object_digest = ObjectDigest::from_str(&input.coin.digest)?;
    let coin_ref: ObjectRef = (
        object_id,
        SequenceNumber::from_u64(input.coin.version),
        object_digest,
    );
    let sender = SuiAddress::from_str(&input.sender)?;
    let validator = SuiAddress::from_str(&input.validator)?;

    let mut ptb = ProgrammableTransactionBuilder::new();
    let split_coint_amount = ptb.pure(input.stake_amount)?;
    ptb.command(Command::SplitCoins(
        Argument::GasCoin,
        vec![split_coint_amount],
    ));

    let obj_arg = ObjectArg::SharedObject {
        id: ObjectID::from_single_byte(SUI_SYSTEM_STATE_OBJECT_ID),
        initial_shared_version: SequenceNumber::from_u64(1),
        mutable: true,
    };

    let move_call = Command::move_call(
        ObjectID::from_single_byte(SUI_SYSTEM_ADDRESS),
        Identifier::new(SUI_SYSTEM_ID).unwrap(),
        Identifier::new(SUI_REQUEST_ADD_STAKE).unwrap(),
        vec![],
        vec![
            ptb.obj(obj_arg)?,
            Argument::NestedResult(0, 0),
            ptb.pure(validator)?,
        ],
    );
    ptb.command(move_call);

    let builder = ptb.finish();

    let tx_data = TransactionData::new_programmable(
        sender,
        vec![coin_ref],
        builder,
        input.gas_budget,
        input.gas_price,
    );
    let data = &bcs::to_bytes(&tx_data)?;
    Ok(data.clone())
}

pub fn encode_tx_signature(tx: Vec<u8>, sig: Vec<u8>) -> Result<Vec<u8>, anyhow::Error> {
    let tx_data: TransactionData = bcs::from_bytes(&tx)?;
    let signature = Ed25519SuiSignature::from_bytes(&sig)?;
    let tx = Transaction::from_data(tx_data, vec![Signature::Ed25519SuiSignature(signature)]);
    let data = &bcs::to_bytes(&tx)?;
    Ok(data.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_encode_split_stake() {
        let input = SuiStakeInput {
            sender: "0xe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2".into(),
            validator: "0x61953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab".into(),
            stake_amount: 1_000_000_000,
            gas_budget: 20_000_000,
            gas_price: 750,

            coin: SuiCoin {
                coin_type: "0x2::sui::SUI".into(),
                coin_object_id:
                    "0x36b8380aa7531d73723657d73a114cfafedf89dc8c76b6752f6daef17e43dda2".into(),
                version: 63683110,
                digest: "E3CzP8KM9qQSUfAp6U3VgTXJTTNEZTQzW6qnCbZYyDgN".into(),
                balance: 12000000000,
            },
        };
        let data = encode_split_and_stake(&input).unwrap();

        assert_eq!(hex::encode(data), "000003000800ca9a3b0000000001010000000000000000000000000000000000000000000000000000000000000005010000000000000001002061953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab020200010100000000000000000000000000000000000000000000000000000000000000000000030a7375695f73797374656d11726571756573745f6164645f7374616b6500030101000300000000010200e6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c20136b8380aa7531d73723657d73a114cfafedf89dc8c76b6752f6daef17e43dda226bacb030000000020c1b8a5e6599401729381e3d66a166f06653f165ed67575c729446ed74043c94be6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2ee02000000000000002d31010000000000");
    }
}
