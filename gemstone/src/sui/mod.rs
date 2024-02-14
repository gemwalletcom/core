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

#[derive(uniffi::Record)]
pub struct SuiStakeOutput {
    pub tx_data: Vec<u8>,
    pub hash: Vec<u8>,
}

use bcs;
use blake2::{digest::consts::U32, Blake2b, Digest};
type Blake2b256 = Blake2b<U32>;
use std::str::FromStr;
use sui_types::{
    base_types::{ObjectID, ObjectRef, SequenceNumber, SuiAddress},
    digests::ObjectDigest,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Argument, Command, ObjectArg, TransactionData},
    Identifier,
};

static SUI_SYSTEM_ID: &str = "sui_system";
static SUI_REQUEST_ADD_STAKE: &str = "request_add_stake";
static SUI_SYSTEM_ADDRESS: u8 = 0x3;
static SUI_SYSTEM_STATE_OBJECT_ID: u8 = 0x5;

pub fn encode_split_and_stake(input: &SuiStakeInput) -> Result<SuiStakeOutput, anyhow::Error> {
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
    let data = bcs::to_bytes(&tx_data)?;
    // let message = IntentMessage::new(Intent::sui_transaction(), tx_data.clone());
    let mut message = vec![0x0u8, 0x0, 0x0];
    message.append(&mut data.clone());
    let mut hasher = Blake2b256::new();
    hasher.update(&message);

    Ok(SuiStakeOutput {
        tx_data: data,
        hash: hasher.finalize().to_vec(),
    })
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
                version: 0x3f4d8e5,
                digest: "HdfF7hswRuvbXbEXjGjmUCt7gLybhvbPvvK8zZbCqyD8".into(),
                balance: 10990277896,
            },
        };
        let data = encode_split_and_stake(&input).unwrap();

        assert_eq!(hex::encode(data.tx_data), "000003000800ca9a3b0000000001010000000000000000000000000000000000000000000000000000000000000005010000000000000001002061953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab020200010100000000000000000000000000000000000000000000000000000000000000000000030a7375695f73797374656d11726571756573745f6164645f7374616b6500030101000300000000010200e6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c20136b8380aa7531d73723657d73a114cfafedf89dc8c76b6752f6daef17e43dda2e5d8f4030000000020f71f24516bc04cbf877d42faf459514448c8de6cff48faa44b3eef3b26782e8fe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2ee02000000000000002d31010000000000");
        assert_eq!(
            hex::encode(data.hash),
            "66be75b0f86ca3a9f24380adc8d8336d8921d5dbdc78f1b3c24c7d6842ce5911"
        );
    }
}
