use crate::models::*;
use std::error::Error;
use std::str::FromStr;
use sui_transaction_builder::{ObjectInput, TransactionBuilder};
use sui_types::{Address, TypeTag};

use super::{TransactionBuilderInput, build_amount_coin, finish_transaction};

pub(super) fn requires_hybrid_funding(coins: &OwnedCoins<Coin>, amount: u64) -> bool {
    coins.address_balance < amount && coins.coin_total() < amount
}

fn build_transfer_ptb(input: &TransferInput) -> Result<TransactionBuilder, Box<dyn Error + Send + Sync>> {
    if let Some(err) = crate::validate_enough_balance(&input.coins, input.amount) {
        return Err(err);
    }
    if input.coins.coins.is_empty() {
        return Err("No SUI coins available for gas".into());
    }
    if !input.send_max && requires_hybrid_funding(&input.coins, input.amount) {
        return Err("Sui native transfer: amount requires combining Address Balance with Coin<SUI> objects, which is not supported".into());
    }

    let recipient = Address::from_str(&input.recipient)?;

    let mut ptb = TransactionBuilder::new();
    if input.send_max {
        let recipient_argument = ptb.pure(&recipient);
        let gas = ptb.gas();
        ptb.transfer_objects(vec![gas], recipient_argument);
        return Ok(ptb);
    }

    let send_coin = if input.coins.address_balance >= input.amount {
        let coin_type: TypeTag = input
            .coins
            .coin_type
            .parse()
            .map_err(|err| format!("invalid Sui native coin type {}: {err}", input.coins.coin_type))?;
        ptb.funds_withdrawal_coin(coin_type, input.amount)
    } else {
        let amount = ptb.pure(&input.amount);
        let gas = ptb.gas();
        let mut split_results = ptb.split_coins(gas, vec![amount]);
        split_results.pop().expect("split_coins should return one argument")
    };

    let recipient_argument = ptb.pure(&recipient);
    ptb.transfer_objects(vec![send_coin], recipient_argument);

    Ok(ptb)
}

pub fn encode_transfer(input: &TransferInput) -> Result<TxOutput, Box<dyn Error + Send + Sync>> {
    let ptb = build_transfer_ptb(input)?;
    let gas_objects = input.coins.coins.iter().map(|x| x.object.to_input()).collect::<Vec<_>>();
    finish_transaction(ptb, TransactionBuilderInput::new(input.sender.as_str(), input.gas.price, input.gas.budget, gas_objects))
        .map_err(|err| Box::new(err) as Box<dyn Error + Send + Sync>)
}

fn build_token_transfer_ptb(input: &TokenTransferInput) -> Result<TransactionBuilder, Box<dyn Error + Send + Sync>> {
    let tokens = &input.tokens;
    if let Some(err) = crate::validate_enough_balance(tokens, input.amount) {
        return Err(err);
    }

    let coin_type: TypeTag = tokens.coin_type.parse().map_err(|err| format!("invalid Sui token coin type {}: {err}", tokens.coin_type))?;
    let recipient = Address::from_str(&input.recipient)?;
    let mut ptb = TransactionBuilder::new();
    let amount_coin = build_amount_coin(&mut ptb, coin_type, input.amount, tokens.address_balance, &tokens.coins)?;
    let recipient_argument = ptb.pure(&recipient);
    ptb.transfer_objects(vec![amount_coin], recipient_argument);

    Ok(ptb)
}

pub fn encode_token_transfer(input: &TokenTransferInput) -> Result<TxOutput, Box<dyn Error + Send + Sync>> {
    let ptb = build_token_transfer_ptb(input)?;
    let gas_coin = ObjectInput::immutable(input.gas_coin.object.object_id, input.gas_coin.object.version, input.gas_coin.object.digest);
    finish_transaction(ptb, TransactionBuilderInput::new(input.sender.as_str(), input.gas.price, input.gas.budget, vec![gas_coin]))
        .map_err(|err| Box::new(err) as Box<dyn Error + Send + Sync>)
}

#[cfg(test)]
mod tests {
    use crate::{SUI_COIN_TYPE, tx_builder::decode_transaction};
    use gem_encoding::encode_base64;
    use primitives::asset_constants::SUI_USDC_TOKEN_ID;
    use sui_types::Transaction;

    use super::*;

    #[test]
    fn test_encode_transfer() {
        let input = TransferInput {
            sender: "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".into(),
            recipient: "0xe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2".into(),
            amount: 8993996480,
            coins: OwnedCoins::new(
                SUI_COIN_TYPE.into(),
                vec![Coin {
                    coin_type: SUI_COIN_TYPE.into(),
                    balance: 8994756360,
                    object: Object {
                        object_id: "0x9f258c85566d977b4c99bb6019560ba99c796e71291269d8f9f3cc9d9f37db46".parse().unwrap(),
                        digest: "GoAwPNYEBKyAgzmQgnxW23bdhnHaLXcqT3o1nEZo4KPM".parse().unwrap(),
                        version: 68419468,
                    },
                }],
                0,
            ),
            send_max: true,
            gas: Gas { budget: 25_000_000, price: 750 },
        };

        let output = encode_transfer(&input).unwrap();
        let tx: Transaction = bcs::from_bytes(&output.tx_data).unwrap();
        let b64_encoded = encode_base64(&output.tx_data);
        let expected_tx = "AAABACDmr4D+GwtC/NlnYuXHD16Nrjn48O4PEYysDVW3TiknwgEBAQABAACpvQST+b0feSpK7cH5nVRTWnWkbDj9VqjyxrfI11gXoQGfJYyFVm2Xe0yZu2AZVgupnHlucSkSadj588ydnzfbRoz/EwQAAAAAIOqzQffiRRpexyiDEtyjm40KqFMf60ohK5jCJ0z3+Lqwqb0Ek/m9H3kqSu3B+Z1UU1p1pGw4/Vao8sa3yNdYF6HuAgAAAAAAAEB4fQEAAAAAAA==";
        let expected_decoded = decode_transaction(expected_tx).unwrap();

        assert_eq!(tx, expected_decoded);
        assert_eq!(b64_encoded, expected_tx);
    }

    #[test]
    fn test_encode_token_transfer() {
        let suip_coin_type = "0xe4239cd951f6c53d9c41e25270d80d31f925ad1655e5ba5b543843d4a66975ee::SUIP::SUIP";
        let input = TokenTransferInput {
            sender: "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".into(),
            recipient: "0xe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2".into(),
            amount: 2400000000,
            tokens: OwnedCoins::new(
                suip_coin_type.into(),
                vec![
                    Coin {
                        coin_type: suip_coin_type.into(),
                        balance: 1400000000,
                        object: Object {
                            object_id: "0x1a6b6023d363f5dcad026f83ddb9bb0f987c941f10db2ab86571711a1a9a1ee6".parse().unwrap(),
                            digest: "CCFDRi15n2mhBVGAoa594VynBKgSRbgZQZgjT4wxFu7B".parse().unwrap(),
                            version: 67155000,
                        },
                    },
                    Coin {
                        coin_type: suip_coin_type.into(),
                        balance: 1000000000,
                        object: Object {
                            object_id: "0x2fd950f33ecdf9e5d797ca3130811e7a973d4c1da5427ac0c910a8c5f6e8b72d".parse().unwrap(),
                            digest: "7CsXhia2TGqy7bXnxH4WLbkzYJBPvCnNVuLvzByvLsRh".parse().unwrap(),
                            version: 67154999,
                        },
                    },
                ],
                0,
            ),
            gas: Gas { budget: 25_000_000, price: 750 },
            gas_coin: Coin {
                coin_type: SUI_COIN_TYPE.into(),
                balance: 100000000,
                object: Object {
                    object_id: "0x890f8c604c7cb5cc194dbf4953ad3dbebd81ef7526be351d3514cc3cc26c9c1d".parse().unwrap(),
                    digest: "3a2sHuj9pJg7RHub4w9EPyBtpxVfHzk52M91HErwMQ4J".parse().unwrap(),
                    version: 69035764,
                },
            },
        };

        let output = encode_token_transfer(&input).unwrap();
        let tx: Transaction = bcs::from_bytes(&output.tx_data).unwrap();
        let b64_encoded = encode_base64(&output.tx_data);
        let expected_tx = "AAAEAQAaa2Aj02P13K0Cb4PdubsPmHyUHxDbKrhlcXEaGpoe5ji0AAQAAAAAIKZSBGYgBc5PwYeX01SAZHnJYxA3pJRvrUZmR7ToQZTWAQAv2VDzPs355deXyjEwgR56lz1MHaVCesDJEKjF9ui3LTe0AAQAAAAAIFwwpOhb+onitRHRqj+wsEA0nNO2KqqOt8/IVbcC0O7oAAgAGA2PAAAAAAAg5q+A/hsLQvzZZ2Llxw9eja45+PDuDxGMrA1Vt04pJ8IDAwEAAAEBAQACAQAAAQECAAEBAwEAAAABAwCpvQST+b0feSpK7cH5nVRTWnWkbDj9VqjyxrfI11gXoQGJD4xgTHy1zBlNv0lTrT2+vYHvdSa+NR01FMw8wmycHfRmHQQAAAAAICYtptS+v/0HkfChzkJo0QzRDQxhli84CM3mMV/dqUBbqb0Ek/m9H3kqSu3B+Z1UU1p1pGw4/Vao8sa3yNdYF6HuAgAAAAAAAEB4fQEAAAAAAA==";
        let expected_decoded = decode_transaction(expected_tx).unwrap();

        assert_eq!(tx, expected_decoded);
        assert_eq!(b64_encoded, expected_tx);
    }

    #[test]
    fn test_encode_token_transfer_from_address_balance() {
        let input = TokenTransferInput {
            sender: "0x1b4cd8b734f2465614678ca0450ce9c4f2ff4835c6a7545522892a1a8fb67991".into(),
            recipient: "0xcf3abaeecfaf42990b8481c03000000000000000000000000000000000000000".into(),
            amount: 200_000_000,
            tokens: OwnedCoins::new(SUI_USDC_TOKEN_ID.into(), vec![], 2_605_380_809),
            gas: Gas { budget: 25_000_000, price: 750 },
            gas_coin: Coin::mock_sui(),
        };

        let output = encode_token_transfer(&input).unwrap();
        let tx: Transaction = bcs::from_bytes(&output.tx_data).unwrap();
        match tx.kind {
            sui_types::TransactionKind::ProgrammableTransaction(ptb) => {
                assert_eq!(ptb.inputs.len(), 2, "expected withdrawal + recipient inputs only");
                assert!(matches!(ptb.inputs[0], sui_types::Input::FundsWithdrawal(_)), "first input must be FundsWithdrawal");
                assert_eq!(ptb.commands.len(), 2, "expected redeem_funds + transfer_objects");
            }
            _ => panic!("expected ProgrammableTransaction"),
        }
    }

    #[test]
    fn test_encode_token_transfer_mixed_balance_and_coin() {
        let input = TokenTransferInput {
            sender: "0x1b4cd8b734f2465614678ca0450ce9c4f2ff4835c6a7545522892a1a8fb67991".into(),
            recipient: "0xcf3abaeecfaf42990b8481c03000000000000000000000000000000000000000".into(),
            amount: 200_000_000,
            tokens: OwnedCoins::new(
                SUI_USDC_TOKEN_ID.into(),
                vec![Coin {
                    coin_type: SUI_USDC_TOKEN_ID.into(),
                    balance: 150_000_000,
                    object: Object {
                        object_id: "0xfa8dca3e71a9ab44eef5becf50358d9c665aef33522e77940ee840c03b385bf3".parse().unwrap(),
                        digest: "HHwqY8eMncQPwrGtdbxGpJ7Sz1QacdvrcUNG9ywtxLs5".parse().unwrap(),
                        version: 895_958_996,
                    },
                }],
                60_000_000,
            ),
            gas: Gas { budget: 25_000_000, price: 750 },
            gas_coin: Coin::mock_sui(),
        };

        let output = encode_token_transfer(&input).unwrap();
        let tx: Transaction = bcs::from_bytes(&output.tx_data).unwrap();
        match tx.kind {
            sui_types::TransactionKind::ProgrammableTransaction(ptb) => {
                assert!(ptb.inputs.iter().any(|inp| matches!(inp, sui_types::Input::FundsWithdrawal(_))));
                assert!(ptb.inputs.iter().any(|inp| matches!(inp, sui_types::Input::ImmutableOrOwned(_))));
                let withdrawals: Vec<u64> = ptb
                    .inputs
                    .iter()
                    .filter_map(|inp| match inp {
                        sui_types::Input::FundsWithdrawal(w) => w.amount(),
                        _ => None,
                    })
                    .collect();
                assert_eq!(withdrawals, vec![50_000_000], "expected shortfall withdrawal only");
            }
            _ => panic!("expected ProgrammableTransaction"),
        }
    }

    #[test]
    fn test_encode_native_transfer_without_gas_coin_rejected() {
        let input = TransferInput {
            sender: "0x1b4cd8b734f2465614678ca0450ce9c4f2ff4835c6a7545522892a1a8fb67991".into(),
            recipient: "0xcf3abaeecfaf42990b8481c03000000000000000000000000000000000000000".into(),
            amount: 1_000_000,
            coins: OwnedCoins::new(SUI_COIN_TYPE.into(), vec![], 2_000_000),
            send_max: false,
            gas: Gas { budget: 25_000_000, price: 750 },
        };
        let err = encode_transfer(&input).expect_err("missing Coin<SUI> for gas must be rejected early");
        assert!(err.to_string().contains("No SUI coins available for gas"), "got: {err}");
    }

    #[test]
    fn test_encode_native_transfer_hybrid_rejected() {
        let input = TransferInput {
            sender: "0x1b4cd8b734f2465614678ca0450ce9c4f2ff4835c6a7545522892a1a8fb67991".into(),
            recipient: "0xcf3abaeecfaf42990b8481c03000000000000000000000000000000000000000".into(),
            amount: 8_000_000_000,
            coins: OwnedCoins::new(
                SUI_COIN_TYPE.into(),
                vec![Coin {
                    coin_type: SUI_COIN_TYPE.into(),
                    balance: 5_000_000_000,
                    object: Object::mock(),
                }],
                4_000_000_000,
            ),
            send_max: false,
            gas: Gas { budget: 25_000_000, price: 750 },
        };
        let err = encode_transfer(&input).expect_err("hybrid native SUI must be rejected");
        assert!(err.to_string().contains("not supported"), "error must explain hybrid is unsupported: {err}");
    }
}
