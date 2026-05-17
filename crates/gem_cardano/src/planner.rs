use num_traits::ToPrimitive;
use primitives::{Chain, SignerError, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, UTXO};

use crate::{
    address::ShelleyAddress,
    transaction::{Transaction, TransactionInput, TransactionOutput},
};

const CARDANO_EXPIRATION_BLOCK_OFFSET: u64 = 7_200;
const FEE_ESTIMATE_LOVELACE: u64 = 170_000;
const MIN_OUTPUT_LOVELACE: u64 = 1_000_000;
const FEE_CONSTANT_MILLI_LOVELACE: u64 = 155_881_000;
const FEE_COEFFICIENT_MILLI_LOVELACE: u64 = 44_046;
const MILLI_LOVELACE: u64 = 1_000;

pub(crate) struct TransactionPlan {
    utxos: Vec<UTXO>,
    amount: u64,
    pub(crate) fee: u64,
    change: u64,
}

pub(crate) fn plan_transfer(input: &TransactionLoadInput) -> Result<TransactionPlan, SignerError> {
    match &input.input_type {
        TransactionInputType::Transfer(asset) if asset.id.chain == Chain::Cardano && asset.id.is_native() => {}
        TransactionInputType::Transfer(_) => return SignerError::invalid_input_err("unsupported Cardano token transfer"),
        _ => return SignerError::invalid_input_err("unsupported Cardano transaction type"),
    }

    let requested_amount = input.value_as_u64()?;

    let utxos = utxos_from_metadata(&input.metadata, &input.sender_address)?;
    if utxos.is_empty() {
        return SignerError::invalid_input_err("missing input UTXOs");
    }

    let selected_utxos = select_utxos(&utxos, requested_amount, input.is_max_value)?;
    let available_amount = sum_amounts(&selected_utxos)?;

    let mut fee = estimate_fee(input, &selected_utxos, requested_amount, input.is_max_value)?;
    let amount = if input.is_max_value {
        available_amount.checked_sub(fee).ok_or_else(|| SignerError::invalid_input("insufficient balance"))?
    } else {
        requested_amount
    };

    let spent = amount.checked_add(fee).ok_or_else(|| SignerError::invalid_input("Cardano amount overflow"))?;
    if spent > available_amount {
        return SignerError::invalid_input_err("insufficient balance");
    }
    let mut change = available_amount - spent;
    if change > 0 && change < MIN_OUTPUT_LOVELACE {
        fee += change;
        change = 0;
    }

    Ok(TransactionPlan {
        utxos: selected_utxos,
        amount,
        fee,
        change,
    })
}

pub(crate) fn transaction_from_plan(input: &TransactionLoadInput, plan: &TransactionPlan) -> Result<Transaction, SignerError> {
    let destination = ShelleyAddress::parse(&input.destination_address)?;
    let expiration_block_number = input
        .metadata
        .get_block_number()
        .map(|block_number| block_number + CARDANO_EXPIRATION_BLOCK_OFFSET)
        .map_err(SignerError::from_display)?;
    let mut outputs = vec![TransactionOutput {
        address: destination.as_bytes().to_vec(),
        amount: plan.amount,
    }];

    if plan.change > 0 {
        let change = ShelleyAddress::parse(&input.sender_address)?;
        outputs.push(TransactionOutput {
            address: change.as_bytes().to_vec(),
            amount: plan.change,
        });
    }

    Ok(Transaction {
        inputs: plan.utxos.iter().map(utxo_transaction_input).collect::<Result<Vec<_>, _>>()?,
        outputs,
        fee: plan.fee,
        expiration_block_number,
    })
}

fn utxos_from_metadata(metadata: &TransactionLoadMetadata, sender_address: &str) -> Result<Vec<UTXO>, SignerError> {
    let sender = ShelleyAddress::parse(sender_address)?;
    let utxos = metadata.get_utxos().map_err(SignerError::from_display)?;
    for utxo in &utxos {
        utxo_transaction_input(utxo)?;
        utxo_amount(utxo)?;
        let address = ShelleyAddress::parse(&utxo.address)?;
        if address.payment_hash() != sender.payment_hash() {
            return SignerError::invalid_input_err("Cardano UTXO address does not match sender address");
        }
    }
    Ok(utxos)
}

fn utxo_transaction_input(utxo: &UTXO) -> Result<TransactionInput, SignerError> {
    let transaction_hash: [u8; 32] = hex::decode(&utxo.transaction_id)
        .map_err(|_| SignerError::invalid_input("invalid Cardano UTXO transaction id"))?
        .try_into()
        .map_err(|_| SignerError::invalid_input("invalid Cardano UTXO transaction id"))?;
    let output_index = utxo.vout.to_u64().ok_or_else(|| SignerError::invalid_input("invalid Cardano UTXO output index"))?;
    Ok(TransactionInput { transaction_hash, output_index })
}

fn utxo_amount(utxo: &UTXO) -> Result<u64, SignerError> {
    let amount = utxo.value.parse::<u64>().map_err(|_| SignerError::invalid_input("invalid Cardano UTXO amount"))?;
    if amount == 0 {
        return SignerError::invalid_input_err("invalid Cardano UTXO amount");
    }
    Ok(amount)
}

fn select_utxos(utxos: &[UTXO], amount: u64, max_amount: bool) -> Result<Vec<UTXO>, SignerError> {
    if max_amount {
        return Ok(utxos.to_vec());
    }

    let target = amount
        .checked_mul(4)
        .and_then(|value| value.checked_div(3))
        .and_then(|value| value.checked_add(FEE_ESTIMATE_LOVELACE))
        .and_then(|value| value.checked_add(MIN_OUTPUT_LOVELACE))
        .ok_or_else(|| SignerError::invalid_input("Cardano amount overflow"))?;
    let mut candidates = utxos.iter().map(|utxo| Ok((utxo.clone(), utxo_amount(utxo)?))).collect::<Result<Vec<_>, SignerError>>()?;
    candidates.sort_by(|(_, left_amount), (_, right_amount)| right_amount.cmp(left_amount));

    let mut selected = Vec::new();
    let mut selected_amount = 0u64;
    for (utxo, amount) in candidates {
        selected_amount = selected_amount.checked_add(amount).ok_or_else(|| SignerError::invalid_input("Cardano amount overflow"))?;
        selected.push(utxo);
        if selected_amount >= target {
            break;
        }
    }
    if selected.is_empty() {
        return SignerError::invalid_input_err("missing input UTXOs");
    }
    Ok(selected)
}

fn estimate_fee(input: &TransactionLoadInput, selected_utxos: &[UTXO], requested_amount: u64, max_amount: bool) -> Result<u64, SignerError> {
    let available_amount = sum_amounts(selected_utxos)?;
    let amount = if max_amount {
        available_amount.saturating_sub(FEE_ESTIMATE_LOVELACE)
    } else {
        requested_amount.min(available_amount.saturating_sub(FEE_ESTIMATE_LOVELACE))
    };
    let change = available_amount.saturating_sub(amount).saturating_sub(FEE_ESTIMATE_LOVELACE);
    let plan = TransactionPlan {
        utxos: selected_utxos.to_vec(),
        amount,
        fee: FEE_ESTIMATE_LOVELACE,
        change,
    };
    let transaction = transaction_from_plan(input, &plan)?;
    Ok(transaction_fee(transaction.signed_size() as u64))
}

fn transaction_fee(transaction_size: u64) -> u64 {
    (FEE_CONSTANT_MILLI_LOVELACE + transaction_size * FEE_COEFFICIENT_MILLI_LOVELACE).div_ceil(MILLI_LOVELACE)
}

fn sum_amounts(utxos: &[UTXO]) -> Result<u64, SignerError> {
    utxos.iter().try_fold(0u64, |sum, utxo| {
        sum.checked_add(utxo_amount(utxo)?).ok_or_else(|| SignerError::invalid_input("Cardano amount overflow"))
    })
}

#[cfg(test)]
mod tests {
    use primitives::{Asset, AssetType, Chain, GasPriceType, TransactionInputType, TransactionLoadMetadata};

    use super::*;

    const OWN_ADDRESS_1: &str = "addr1q8043m5heeaydnvtmmkyuhe6qv5havvhsf0d26q3jygsspxlyfpyk6yqkw0yhtyvtr0flekj84u64az82cufmqn65zdsylzk23";
    const TO_ADDRESS: &str = "addr1q92cmkgzv9h4e5q7mnrzsuxtgayvg4qr7y3gyx97ukmz3dfx7r9fu73vqn25377ke6r0xk97zw07dqr9y5myxlgadl2s0dgke5";
    const TEST_EXPIRATION_BLOCK_NUMBER: u64 = 190_000_000;
    const TEST_BLOCK_NUMBER: u64 = TEST_EXPIRATION_BLOCK_NUMBER - CARDANO_EXPIRATION_BLOCK_OFFSET;

    fn wallet_core_input(amount: &str, is_max_value: bool) -> TransactionLoadInput {
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Cardano)),
            sender_address: OWN_ADDRESS_1.to_string(),
            destination_address: TO_ADDRESS.to_string(),
            value: amount.to_string(),
            gas_price: GasPriceType::regular(0u64),
            memo: None,
            is_max_value,
            metadata: TransactionLoadMetadata::Cardano {
                block_number: TEST_BLOCK_NUMBER,
                utxos: vec![
                    UTXO {
                        transaction_id: "f074134aabbfb13b8aec7cf5465b1e5a862bde5cb88532cc7e64619179b3e767".to_string(),
                        vout: 1,
                        value: "1500000".to_string(),
                        address: OWN_ADDRESS_1.to_string(),
                    },
                    UTXO {
                        transaction_id: "554f2fd942a23d06835d26bbd78f0106fa94c8a551114a0bef81927f66467af0".to_string(),
                        vout: 0,
                        value: "6500000".to_string(),
                        address: OWN_ADDRESS_1.to_string(),
                    },
                ],
            },
        }
    }

    #[test]
    fn test_plan_transfer_vectors() {
        let plan = plan_transfer(&wallet_core_input("7000000", false)).unwrap();
        assert_eq!(plan.utxos.len(), 2);
        assert_eq!(sum_amounts(&plan.utxos).unwrap(), 8_000_000);
        assert_eq!(plan.amount, 7_000_000);
        assert_eq!(plan.fee, 1_000_000);
        assert_eq!(plan.change, 0);
        assert_eq!(plan.utxos[0].value, "6500000");
        assert_eq!(plan.utxos[1].value, "1500000");

        let plan = plan_transfer(&wallet_core_input("1", false)).unwrap();
        assert_eq!(plan.utxos.len(), 1);
        assert_eq!(sum_amounts(&plan.utxos).unwrap(), 6_500_000);
        assert_eq!(plan.amount, 1);
        assert_eq!(plan.fee, 168_479);
        assert_eq!(plan.change, 6_331_520);

        let plan = plan_transfer(&wallet_core_input("2000000", false)).unwrap();
        assert_eq!(plan.utxos.len(), 1);
        assert_eq!(sum_amounts(&plan.utxos).unwrap(), 6_500_000);
        assert_eq!(plan.amount, 2_000_000);
        assert_eq!(plan.fee, 168_655);
        assert_eq!(plan.change, 4_331_345);

        let plan = plan_transfer(&wallet_core_input("2000000", true)).unwrap();
        assert_eq!(plan.utxos.len(), 2);
        assert_eq!(sum_amounts(&plan.utxos).unwrap(), 8_000_000);
        assert_eq!(plan.amount, 7_832_622);
        assert_eq!(plan.fee, 167_378);
        assert_eq!(plan.change, 0);
        assert_eq!(plan.utxos[0].value, "1500000");
        assert_eq!(plan.utxos[1].value, "6500000");
    }

    #[test]
    fn test_plan_transfer_android_vector_fee() {
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Cardano)),
            sender_address: "addr1q9d2dxen8ywvs9yzxxn2w4mvffn797fquauvugt2ug7mfsuqj3lzdq9h0rsketzszrnfm930658swmpe7kpq53c2tmwql4rvtq".to_string(),
            destination_address: "addr1q9d2dxen8ywvs9yzxxn2w4mvffn797fquauvugt2ug7mfsuqj3lzdq9h0rsketzszrnfm930658swmpe7kpq53c2tmwql4rvtq".to_string(),
            value: "10000".to_string(),
            gas_price: GasPriceType::regular(0u64),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Cardano {
                block_number: TEST_BLOCK_NUMBER,
                utxos: vec![UTXO {
                    address: "addr1q9d2dxen8ywvs9yzxxn2w4mvffn797fquauvugt2ug7mfsuqj3lzdq9h0rsketzszrnfm930658swmpe7kpq53c2tmwql4rvtq".to_string(),
                    transaction_id: "412c5a964cf4515210bf4b82f45df6521c38e1e5381f27638fc509bef6679378".to_string(),
                    value: "7945975".to_string(),
                    vout: 1,
                }],
            },
        };

        let plan = plan_transfer(&input).unwrap();
        assert_eq!(plan.fee, 168_567);
        assert_eq!(plan.change, 7_767_408);
    }

    #[test]
    fn test_plan_transfer_validation() {
        let mut input = wallet_core_input("1", false);
        input.metadata = TransactionLoadMetadata::Cardano {
            utxos: vec![],
            block_number: TEST_BLOCK_NUMBER,
        };
        assert!(plan_transfer(&input).is_err());

        input = wallet_core_input("1", false);
        input.metadata = TransactionLoadMetadata::Cardano {
            block_number: TEST_BLOCK_NUMBER,
            utxos: vec![UTXO {
                transaction_id: "zz".to_string(),
                vout: 0,
                value: "1".to_string(),
                address: OWN_ADDRESS_1.to_string(),
            }],
        };
        assert!(plan_transfer(&input).is_err());

        input = wallet_core_input("1", false);
        input.destination_address = "stake1uykptcz226y5r5at5rfqqm00p9n0z0yfajz3gk3j3wm8dxg2sn0r4".to_string();
        assert!(plan_transfer(&input).is_err());

        input = wallet_core_input("1", false);
        input.input_type = TransactionInputType::Transfer(Asset::mock_with_params(
            Chain::Cardano,
            Some("policy.asset".to_string()),
            "Cardano Token".to_string(),
            "TOKEN".to_string(),
            0,
            AssetType::TOKEN,
        ));
        assert_eq!(plan_transfer(&input).err().unwrap().to_string(), "Invalid input: unsupported Cardano token transfer");

        input = wallet_core_input("1", false);
        input.metadata = TransactionLoadMetadata::Cardano {
            block_number: TEST_BLOCK_NUMBER,
            utxos: vec![UTXO {
                transaction_id: "f074134aabbfb13b8aec7cf5465b1e5a862bde5cb88532cc7e64619179b3e767".to_string(),
                vout: 1,
                value: "1500000".to_string(),
                address: TO_ADDRESS.to_string(),
            }],
        };
        assert_eq!(
            plan_transfer(&input).err().unwrap().to_string(),
            "Invalid input: Cardano UTXO address does not match sender address"
        );
    }

    #[test]
    fn test_transaction_expiration_block_number() {
        let input = wallet_core_input("1", false);
        let plan = plan_transfer(&input).unwrap();
        let transaction = transaction_from_plan(&input, &plan).unwrap();
        assert_eq!(transaction.expiration_block_number, TEST_EXPIRATION_BLOCK_NUMBER);
    }
}
