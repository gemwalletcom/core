use super::model::*;
use anyhow::{anyhow, Error};
use std::str::FromStr;
use sui_transaction_builder::{unresolved::Input, Serialized, TransactionBuilder};
use sui_types::{Address, Argument};

pub fn encode_transfer(input: &TransferInput) -> Result<TxOutput, Error> {
    if let Some(err) = super::validate_enough_balance(&input.coins, input.amount) {
        return Err(err);
    }

    let sender = Address::from_str(&input.sender)?;
    let recipient = Address::from_str(&input.recipient)?;

    let mut ptb = TransactionBuilder::new();
    if input.send_max {
        let recipient_argument = ptb.input(Serialized(&recipient));

        ptb.transfer_objects(vec![ptb.gas()], recipient_argument);
    } else {
        let amount = ptb.input(Serialized(&input.amount));
        let split_result = match ptb.split_coins(ptb.gas(), vec![amount]) {
            Argument::Result(idx) => Argument::NestedResult(idx, 0),
            _ => panic!("self.command should always give a Argument::Result"),
        };
        let recipient_argument = ptb.input(Serialized(&recipient));

        ptb.transfer_objects(vec![split_result], recipient_argument);
    }

    let coins = input.coins.iter().map(|x| x.object.to_input()).collect::<Vec<_>>();
    super::tx::fill_tx(&mut ptb, sender, input.gas.price, input.gas.budget, coins);
    let tx = ptb.finish()?;

    TxOutput::from_tx(&tx)
}

pub fn encode_token_transfer(input: &TokenTransferInput) -> Result<TxOutput, Error> {
    if let Some(err) = super::validate_enough_balance(&input.tokens, input.amount) {
        return Err(err);
    }
    let mut ptb = TransactionBuilder::new();
    let sender = Address::from_str(&input.sender)?;
    let recipient = Address::from_str(&input.recipient)?;

    if input.tokens.is_empty() {
        return Err(anyhow!("tokens vector is empty"));
    }

    let mut coins_inputs: Vec<Argument> = input.tokens.clone().into_iter().map(|x| ptb.input(x.object.to_input())).collect();

    // Get first coin
    let first_coin = coins_inputs.remove(0);

    // Merge coins if more than one
    if !coins_inputs.is_empty() {
        ptb.merge_coins(first_coin, coins_inputs);
    }

    // Split and transfer
    let amount = ptb.input(Serialized(&input.amount));
    let split_result = match ptb.split_coins(first_coin, vec![amount]) {
        Argument::Result(idx) => Argument::NestedResult(idx, 0),
        _ => return Err(anyhow!("failed to split coins")),
    };
    let recipient_argument = ptb.input(Serialized(&recipient));
    ptb.transfer_objects(vec![split_result], recipient_argument);

    let gas_coin = Input::immutable(
        input.gas_coin.object.object_id.parse().unwrap(),
        input.gas_coin.object.version,
        input.gas_coin.object.digest.parse().unwrap(),
    );

    super::tx::fill_tx(&mut ptb, sender, input.gas.price, input.gas.budget, vec![gas_coin]);

    let tx = ptb.finish()?;
    TxOutput::from_tx(&tx)
}

#[cfg(test)]
mod tests {
    use crate::{tx::decode_transaction, SUI_COIN_TYPE};
    use base64::{engine::general_purpose, Engine as _};
    use sui_types::Transaction;

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
        let tx: Transaction = bcs::from_bytes(&output.tx_data).unwrap();
        let b64_encoded = general_purpose::STANDARD.encode(output.tx_data);
        let expected_tx = "AAABACDmr4D+GwtC/NlnYuXHD16Nrjn48O4PEYysDVW3TiknwgEBAQABAACpvQST+b0feSpK7cH5nVRTWnWkbDj9VqjyxrfI11gXoQGfJYyFVm2Xe0yZu2AZVgupnHlucSkSadj588ydnzfbRoz/EwQAAAAAIOqzQffiRRpexyiDEtyjm40KqFMf60ohK5jCJ0z3+Lqwqb0Ek/m9H3kqSu3B+Z1UU1p1pGw4/Vao8sa3yNdYF6HuAgAAAAAAAEB4fQEAAAAAAA==";
        let expected_decoded = decode_transaction(expected_tx).unwrap();

        std::fs::write("transfer.json", serde_json::to_string_pretty(&tx).unwrap()).unwrap();
        std::fs::write("expected_transfer.json", serde_json::to_string_pretty(&expected_decoded).unwrap()).unwrap();

        assert_eq!(tx, expected_decoded);
        assert_eq!(b64_encoded, expected_tx);
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
        let tx: Transaction = bcs::from_bytes(&output.tx_data).unwrap();
        let b64_encoded = general_purpose::STANDARD.encode(output.tx_data);
        let expected_tx = "AAAEAQAaa2Aj02P13K0Cb4PdubsPmHyUHxDbKrhlcXEaGpoe5ji0AAQAAAAAIKZSBGYgBc5PwYeX01SAZHnJYxA3pJRvrUZmR7ToQZTWAQAv2VDzPs355deXyjEwgR56lz1MHaVCesDJEKjF9ui3LTe0AAQAAAAAIFwwpOhb+onitRHRqj+wsEA0nNO2KqqOt8/IVbcC0O7oAAgAGA2PAAAAAAAg5q+A/hsLQvzZZ2Llxw9eja45+PDuDxGMrA1Vt04pJ8IDAwEAAAEBAQACAQAAAQECAAEBAwEAAAABAwCpvQST+b0feSpK7cH5nVRTWnWkbDj9VqjyxrfI11gXoQGJD4xgTHy1zBlNv0lTrT2+vYHvdSa+NR01FMw8wmycHfRmHQQAAAAAICYtptS+v/0HkfChzkJo0QzRDQxhli84CM3mMV/dqUBbqb0Ek/m9H3kqSu3B+Z1UU1p1pGw4/Vao8sa3yNdYF6HuAgAAAAAAAEB4fQEAAAAAAA==";
        let expected_decoded = decode_transaction(expected_tx).unwrap();

        assert_eq!(tx, expected_decoded);
        assert_eq!(b64_encoded, expected_tx);
    }
}
