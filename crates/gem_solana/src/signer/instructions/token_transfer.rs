use crate::signer::transaction;
use primitives::{Asset, AssetType, SignerError, SignerInput, SolanaTokenProgramId};
use solana_primitives::{
    Instruction, Pubkey,
    instructions::{
        associated_token::{create_associated_token_account_idempotent, get_associated_token_address_with_program_id},
        memo::memo,
        program_ids::{token_2022_program, token_program},
        token::transfer_checked_with_program_id,
    },
};

pub(in crate::signer) fn token_transfer(input: &SignerInput, sender: Pubkey) -> Result<Vec<Instruction>, SignerError> {
    let token_program_id = token_program_id(input)?;
    let sender_token_address = input
        .metadata
        .get_sender_token_address()?
        .ok_or_else(|| SignerError::invalid_input("missing sender token address"))?;
    let sender_token_address = Pubkey::from_base58(&sender_token_address).map_err(SignerError::from_display)?;
    let mint = Pubkey::from_base58(input.input_type.get_asset().id.get_token_id()?).map_err(SignerError::from_display)?;
    let decimals = token_decimals(input.input_type.get_asset())?;
    let amount = input.value_as_u64()?;

    let mut instructions = transaction::compute_budget_instructions(&input.fee)?;
    let recipient_token_address = match input.metadata.get_recipient_token_address()? {
        Some(recipient_token_address) => Pubkey::from_base58(&recipient_token_address).map_err(SignerError::from_display)?,
        None => {
            let recipient = Pubkey::from_base58(&input.destination_address).map_err(SignerError::from_display)?;
            let recipient_token_address = get_associated_token_address_with_program_id(&recipient, &mint, &token_program_id);
            instructions.push(create_associated_token_account_idempotent(&sender, &recipient, &mint, &token_program_id));
            recipient_token_address
        }
    };

    if let Some(memo_text) = input.get_memo() {
        instructions.push(memo(memo_text, &[]));
    }
    instructions.push(transfer_checked_with_program_id(
        &sender_token_address,
        &mint,
        &recipient_token_address,
        &sender,
        amount,
        decimals,
        &token_program_id,
    ));
    Ok(instructions)
}

fn token_program_id(input: &SignerInput) -> Result<Pubkey, SignerError> {
    let asset = input.input_type.get_asset();
    let token_program_id = token_program_id_for_asset(asset)?;
    if let Some(metadata_program_id) = input.metadata.get_solana_token_program_id()? {
        let metadata_program_id = token_program_id_for_metadata(&metadata_program_id);
        if metadata_program_id != token_program_id {
            return Err(SignerError::invalid_input("Solana token program metadata does not match asset type"));
        }
        return Ok(metadata_program_id);
    }
    Ok(token_program_id)
}

fn token_program_id_for_asset(asset: &Asset) -> Result<Pubkey, SignerError> {
    match &asset.asset_type {
        AssetType::SPL => Ok(token_program()),
        AssetType::SPL2022 => Ok(token_2022_program()),
        other => Err(SignerError::invalid_input(format!("unsupported Solana token type: {other:?}"))),
    }
}

fn token_program_id_for_metadata(token_program_id: &SolanaTokenProgramId) -> Pubkey {
    match token_program_id {
        SolanaTokenProgramId::Token => token_program(),
        SolanaTokenProgramId::Token2022 => token_2022_program(),
    }
}

fn token_decimals(asset: &Asset) -> Result<u8, SignerError> {
    u8::try_from(asset.decimals).map_err(|_| SignerError::invalid_input("invalid Solana token decimals"))
}

#[cfg(test)]
mod tests {
    use crate::signer::{SolanaChainSigner, testkit::*};
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{Asset, AssetId, AssetType, Chain, ChainSigner, GasPriceType, SignerInput, SolanaTokenProgramId, TransactionFee, TransactionInputType, TransactionLoadInput};
    use solana_primitives::{
        Pubkey,
        instructions::{
            associated_token::get_associated_token_address_with_program_id,
            program_ids::{ASSOCIATED_TOKEN_PROGRAM_ID, MEMO_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID, token_program},
        },
    };

    fn transfer_checked_data(amount: u64, decimals: u8) -> Vec<u8> {
        let mut data = vec![12];
        data.extend_from_slice(&amount.to_le_bytes());
        data.push(decimals);
        data
    }

    #[test]
    fn test_sign_token_transfer() {
        let signer = SolanaChainSigner;
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::mock_spl_token()),
            sender_address: sender_address(),
            destination_address: TEST_RECIPIENT.to_string(),
            value: "123456".to_string(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: solana_metadata(Some(TEST_SENDER_TOKEN_ADDRESS), None, Some(SolanaTokenProgramId::Token)),
        };
        let input = SignerInput::new(input, TransactionFee::default());

        let result = signer.sign_token_transfer(&input, &TEST_PRIVATE_KEY).unwrap();

        let transaction = crate::decode_transaction(&result).unwrap();
        let mint = Pubkey::from_base58(Asset::mock_spl_token().id.get_token_id().unwrap()).unwrap();
        let recipient = Pubkey::from_base58(TEST_RECIPIENT).unwrap();
        let recipient_token_address = get_associated_token_address_with_program_id(&recipient, &mint, &token_program());
        assert_eq!(
            (0..transaction.instructions().len()).map(|index| program_id(&transaction, index)).collect::<Vec<_>>(),
            vec![ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID]
        );
        assert_eq!(transaction.instructions()[0].data, vec![1]);
        assert_eq!(account_key(&transaction, 0, 1), recipient_token_address);
        assert_eq!(account_key(&transaction, 1, 2), recipient_token_address);
        assert_eq!(transaction.instructions()[1].data, transfer_checked_data(123456, 6));

        let spl2022_asset = Asset::new(
            AssetId::from_token(Chain::Solana, Asset::mock_spl_token().id.get_token_id().unwrap()),
            "Token 2022".to_string(),
            "T22".to_string(),
            6,
            AssetType::SPL2022,
        );
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Transfer(spl2022_asset),
            sender_address: sender_address(),
            destination_address: TEST_RECIPIENT.to_string(),
            value: "7".to_string(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: solana_metadata(Some(TEST_SENDER_TOKEN_ADDRESS), Some(TEST_SENDER_TOKEN_ADDRESS), Some(SolanaTokenProgramId::Token2022)),
        };
        let input = SignerInput::new(input, TransactionFee::default());

        let result = signer.sign_token_transfer(&input, &TEST_PRIVATE_KEY).unwrap();

        let transaction = crate::decode_transaction(&result).unwrap();
        assert_eq!(transaction.instructions().len(), 1);
        assert_eq!(program_id(&transaction, 0), TOKEN_2022_PROGRAM_ID);
        assert_eq!(transaction.instructions()[0].data, transfer_checked_data(7, 6));

        let mismatched_asset = Asset::mock_spl_token();
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Transfer(mismatched_asset),
            sender_address: sender_address(),
            destination_address: TEST_RECIPIENT.to_string(),
            value: "7".to_string(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: solana_metadata(Some(TEST_SENDER_TOKEN_ADDRESS), Some(TEST_SENDER_TOKEN_ADDRESS), Some(SolanaTokenProgramId::Token2022)),
        };
        let input = SignerInput::new(input, TransactionFee::default());
        assert_eq!(
            signer.sign_token_transfer(&input, &TEST_PRIVATE_KEY).unwrap_err().to_string(),
            "Invalid input: Solana token program metadata does not match asset type"
        );

        let input = TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::mock_spl_token()),
            sender_address: sender_address(),
            destination_address: TEST_RECIPIENT.to_string(),
            value: "123456".to_string(),
            gas_price: GasPriceType::regular(0),
            memo: Some("token memo".to_string()),
            is_max_value: false,
            metadata: solana_metadata(Some(TEST_SENDER_TOKEN_ADDRESS), Some(TEST_SENDER_TOKEN_ADDRESS), Some(SolanaTokenProgramId::Token)),
        };
        let input = SignerInput::new(input, TransactionFee::default());
        let result = signer.sign_token_transfer(&input, &TEST_PRIVATE_KEY).unwrap();
        let transaction = crate::decode_transaction(&result).unwrap();
        assert_eq!(
            (0..transaction.instructions().len()).map(|index| program_id(&transaction, index)).collect::<Vec<_>>(),
            vec![MEMO_PROGRAM_ID, TOKEN_PROGRAM_ID]
        );
        assert_eq!(transaction.instructions()[0].accounts, Vec::<u8>::new());
        assert_eq!(transaction.instructions()[0].data, b"token memo");
    }
}
