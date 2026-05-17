use crate::signer::transaction;
use primitives::{SignerError, SignerInput};
use solana_primitives::{
    Instruction, Pubkey,
    instructions::{memo::memo, system::transfer},
};

pub(in crate::signer) fn native_transfer(input: &SignerInput, sender: Pubkey) -> Result<Vec<Instruction>, SignerError> {
    let recipient = Pubkey::from_base58(&input.destination_address).map_err(SignerError::from_display)?;
    let mut instructions = transaction::compute_budget_instructions(&input.fee)?;
    if let Some(memo_text) = input.get_memo() {
        instructions.push(memo(memo_text, &[]));
    }
    instructions.push(transfer(&sender, &recipient, input.value_as_u64()?));
    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use crate::signer::{SolanaChainSigner, testkit::*};
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{Asset, ChainSigner, GasPriceType, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput};
    use solana_primitives::instructions::program_ids::{COMPUTE_BUDGET_PROGRAM_ID, MEMO_PROGRAM_ID, SYSTEM_PROGRAM_ID};

    // https://github.com/trustwallet/wallet-core/blob/master/rust/tw_tests/tests/chains/solana/solana_sign.rs
    const REFERENCE_TRANSFER_PRIVATE_KEY: &str = "A7psj2GW7ZMdY4E5hJq14KMeYg7HFjULSsWSrTXZLvYr";
    const REFERENCE_TRANSFER_TX: &str = "3p2kzZ1DvquqC6LApPuxpTg5CCDVPqJFokGSnGhnBHrta4uq7S2EyehV1XNUVXp51D69GxGzQZUjikfDzbWBG2aFtG3gHT1QfLzyFKHM4HQtMQMNXqay1NAeiiYZjNhx9UvMX4uAQZ4Q6rx6m2AYfQ7aoMUrejq298q1wBFdtS9XVB5QTiStnzC7zs97FUEK2T4XapjF1519EyFBViTfHpGpnf5bfizDzsW9kYUtRDW1UC2LgHr7npgq5W9TBmHf9hSmRgM9XXucjXLqubNWE7HUMhbKjuBqkirRM";

    fn transfer_data(lamports: u64) -> Vec<u8> {
        let mut data = vec![2, 0, 0, 0];
        data.extend_from_slice(&lamports.to_le_bytes());
        data
    }

    #[test]
    fn test_sign_transfer() {
        let signer = SolanaChainSigner;
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::mock_sol()),
            sender_address: sender_address(),
            destination_address: TEST_RECIPIENT.to_string(),
            value: "42".to_string(),
            gas_price: GasPriceType::solana(5_000u64, 0u64, 2u64),
            memo: Some("HelloSolanaMemo".to_string()),
            is_max_value: false,
            metadata: solana_metadata(None, None, None),
        };
        let fee = TransactionFee::new_gas_price_type(GasPriceType::solana(5_000u64, 0u64, 2u64), 5_000u64.into(), 2_000u64.into(), Default::default());
        let input = SignerInput::new(input, fee);

        let result = signer.sign_transfer(&input, &TEST_PRIVATE_KEY).unwrap();

        let transaction = crate::decode_transaction(&result).unwrap();
        assert_eq!(transaction.signatures().len(), 1);
        assert_ne!(transaction.signatures()[0].as_bytes(), &[0u8; 64]);
        assert_eq!(
            (0..transaction.instructions().len()).map(|index| program_id(&transaction, index)).collect::<Vec<_>>(),
            vec![COMPUTE_BUDGET_PROGRAM_ID, COMPUTE_BUDGET_PROGRAM_ID, MEMO_PROGRAM_ID, SYSTEM_PROGRAM_ID]
        );
        assert_eq!(transaction.instructions()[0].data, {
            let mut data = vec![3];
            data.extend_from_slice(&2u64.to_le_bytes());
            data
        });
        assert_eq!(transaction.instructions()[1].data, {
            let mut data = vec![2];
            data.extend_from_slice(&2_000u32.to_le_bytes());
            data
        });
        assert_eq!(transaction.instructions()[2].accounts, Vec::<u8>::new());
        assert_eq!(transaction.instructions()[2].data, b"HelloSolanaMemo");
        assert_eq!(transaction.instructions()[3].data, transfer_data(42));
    }

    #[test]
    fn test_sign_reference_transfer() {
        let signer = SolanaChainSigner;
        let private_key = private_key_base58(REFERENCE_TRANSFER_PRIVATE_KEY);
        let transfer = TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::mock_sol()),
            sender_address: sender_address_for_key(&private_key),
            destination_address: TEST_RECIPIENT.to_string(),
            value: "42".to_string(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: solana_metadata(None, None, None),
        };
        let transfer = SignerInput::new(transfer, TransactionFee::default());

        let result = signer.sign_transfer(&transfer, &private_key).unwrap();

        assert_eq!(base58_transaction(&result), REFERENCE_TRANSFER_TX);
    }
}
