use crate::{METAPLEX_CORE_PROGRAM, METAPLEX_PROGRAM, SYSTEM_PROGRAM_ID, SYSVAR_INSTRUCTIONS_ID, get_token_program_by_id, metaplex::metadata::Metadata, signer::transaction};
use primitives::{NFTAsset, SignerError, SignerInput, SolanaNftStandard, SolanaTokenProgramId, TransactionLoadMetadata, contract_constants::SOLANA_METAPLEX_AUTH_RULES_PROGRAM_ID};
use solana_primitives::{
    AccountMeta, Instruction, Pubkey,
    instructions::{
        associated_token::get_associated_token_address_with_program_id,
        memo::memo,
        program_ids::{ASSOCIATED_TOKEN_PROGRAM_ID, system_program},
    },
};

use super::token_transfer::spl_transfer_checked;

const MPL_CORE_TRANSFER_V1: u8 = 14;
const MPL_TOKEN_METADATA_TRANSFER_V1: u8 = 49;

pub(in crate::signer) fn nft_transfer(input: &SignerInput, sender: Pubkey) -> Result<Vec<Instruction>, SignerError> {
    let nft_asset = input.input_type.get_nft_asset().map_err(SignerError::invalid_input)?;
    let TransactionLoadMetadata::Solana { token_program, nft, .. } = &input.metadata else {
        return Err(SignerError::invalid_input("expected Solana metadata"));
    };
    let standard = nft.as_ref().ok_or_else(|| SignerError::invalid_input("missing Solana NFT standard"))?;

    match standard {
        SolanaNftStandard::Core { collection } => metaplex_core_transfer(input, sender, nft_asset, collection.as_deref()),
        SolanaNftStandard::NonFungible => spl_nft_transfer(input, sender, nft_asset, spl_program(token_program.as_ref())?),
        SolanaNftStandard::ProgrammableNonFungible { rule_set } => {
            metaplex_token_metadata_transfer(input, sender, nft_asset, spl_program(token_program.as_ref())?, rule_set.as_deref())
        }
    }
}

fn spl_program(token_program: Option<&SolanaTokenProgramId>) -> Result<&SolanaTokenProgramId, SignerError> {
    token_program.ok_or_else(|| SignerError::invalid_input("missing SPL token program for NFT"))
}

fn spl_nft_transfer(input: &SignerInput, sender: Pubkey, nft_asset: &NFTAsset, token_program: &SolanaTokenProgramId) -> Result<Vec<Instruction>, SignerError> {
    let token_program_id = Pubkey::from_base58(get_token_program_by_id(token_program.clone())).map_err(SignerError::from_display)?;
    let mint = Pubkey::from_base58(&nft_asset.token_id).map_err(SignerError::from_display)?;
    spl_transfer_checked(input, sender, mint, 1, 0, token_program_id)
}

fn metaplex_token_metadata_transfer(
    input: &SignerInput,
    sender: Pubkey,
    nft_asset: &NFTAsset,
    spl_token_program: &SolanaTokenProgramId,
    rule_set: Option<&str>,
) -> Result<Vec<Instruction>, SignerError> {
    let mpl_program = Pubkey::from_base58(METAPLEX_PROGRAM).map_err(SignerError::from_display)?;
    let token_program = Pubkey::from_base58(get_token_program_by_id(spl_token_program.clone())).map_err(SignerError::from_display)?;
    let ata_program = Pubkey::from_base58(ASSOCIATED_TOKEN_PROGRAM_ID).map_err(SignerError::from_display)?;
    let sysvar_instructions = Pubkey::from_base58(SYSVAR_INSTRUCTIONS_ID).map_err(SignerError::from_display)?;
    let mint = Pubkey::from_base58(&nft_asset.token_id).map_err(SignerError::from_display)?;
    let recipient = Pubkey::from_base58(&input.destination_address).map_err(SignerError::from_display)?;

    let sender_token_address = input
        .metadata
        .get_sender_token_address()
        .map_err(SignerError::from_display)?
        .ok_or_else(|| SignerError::invalid_input("missing sender token address"))?;
    let sender_token_address = Pubkey::from_base58(&sender_token_address).map_err(SignerError::from_display)?;
    let recipient_token_address = match input.metadata.get_recipient_token_address().map_err(SignerError::from_display)? {
        Some(address) => Pubkey::from_base58(&address).map_err(SignerError::from_display)?,
        None => get_associated_token_address_with_program_id(&recipient, &mint, &token_program),
    };

    let metadata_pda = Metadata::find_pda(mint).ok_or_else(|| SignerError::invalid_input("failed to derive metadata PDA"))?.0;
    let master_edition = Metadata::find_master_edition_pda(mint)
        .ok_or_else(|| SignerError::invalid_input("failed to derive master edition PDA"))?
        .0;
    let source_token_record = Metadata::find_token_record_pda(mint, sender_token_address)
        .ok_or_else(|| SignerError::invalid_input("failed to derive source token record PDA"))?
        .0;
    let destination_token_record = Metadata::find_token_record_pda(mint, recipient_token_address)
        .ok_or_else(|| SignerError::invalid_input("failed to derive destination token record PDA"))?
        .0;

    let (auth_rules_program, auth_rules) = match rule_set {
        Some(rule_set) => {
            let program = Pubkey::from_base58(SOLANA_METAPLEX_AUTH_RULES_PROGRAM_ID).map_err(SignerError::from_display)?;
            let rules = Pubkey::from_base58(rule_set).map_err(|_| SignerError::invalid_input(format!("invalid Solana pNFT rule set: {rule_set}")))?;
            (program, rules)
        }
        None => (mpl_program, mpl_program),
    };

    let mut data = Vec::with_capacity(11);
    data.push(MPL_TOKEN_METADATA_TRANSFER_V1);
    data.push(0);
    data.extend_from_slice(&1u64.to_le_bytes());
    data.push(0);

    let mut instructions = transaction::compute_budget_instructions(&input.fee)?;
    if let Some(memo_text) = input.get_memo() {
        instructions.push(memo(memo_text, &[]));
    }
    instructions.push(Instruction {
        program_id: mpl_program,
        accounts: vec![
            AccountMeta::new_writable(sender_token_address),
            AccountMeta::new_readonly(sender),
            AccountMeta::new_writable(recipient_token_address),
            AccountMeta::new_readonly(recipient),
            AccountMeta::new_readonly(mint),
            AccountMeta::new_writable(metadata_pda),
            AccountMeta::new_readonly(master_edition),
            AccountMeta::new_writable(source_token_record),
            AccountMeta::new_writable(destination_token_record),
            AccountMeta::new_signer(sender),
            AccountMeta::new_signer_writable(sender),
            AccountMeta::new_readonly(system_program()),
            AccountMeta::new_readonly(sysvar_instructions),
            AccountMeta::new_readonly(token_program),
            AccountMeta::new_readonly(ata_program),
            AccountMeta::new_readonly(auth_rules_program),
            AccountMeta::new_readonly(auth_rules),
        ],
        data,
    });
    Ok(instructions)
}

fn metaplex_core_transfer(input: &SignerInput, sender: Pubkey, nft_asset: &NFTAsset, collection: Option<&str>) -> Result<Vec<Instruction>, SignerError> {
    let core_program = Pubkey::from_base58(METAPLEX_CORE_PROGRAM).map_err(SignerError::from_display)?;
    let system_program_id = Pubkey::from_base58(SYSTEM_PROGRAM_ID).map_err(SignerError::from_display)?;
    let asset = Pubkey::from_base58(&nft_asset.token_id).map_err(SignerError::from_display)?;
    let new_owner = Pubkey::from_base58(&input.destination_address).map_err(SignerError::from_display)?;
    let collection_account = match collection {
        Some(address) => Pubkey::from_base58(address).map_err(|_| SignerError::invalid_input(format!("invalid Solana Core NFT collection: {address}")))?,
        None => core_program,
    };

    let mut instructions = transaction::compute_budget_instructions(&input.fee)?;
    if let Some(memo_text) = input.get_memo() {
        instructions.push(memo(memo_text, &[]));
    }
    instructions.push(Instruction {
        program_id: core_program,
        accounts: vec![
            AccountMeta::new_writable(asset),
            AccountMeta::new_readonly(collection_account),
            AccountMeta::new_signer_writable(sender),
            AccountMeta::new_signer(sender),
            AccountMeta::new_readonly(new_owner),
            AccountMeta::new_readonly(system_program_id),
            AccountMeta::new_readonly(core_program),
        ],
        data: vec![MPL_CORE_TRANSFER_V1, 0],
    });
    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use crate::{
        METAPLEX_CORE_PROGRAM, METAPLEX_PROGRAM, SYSTEM_PROGRAM_ID, SYSVAR_INSTRUCTIONS_ID, TOKEN_PROGRAM,
        signer::{SolanaChainSigner, testkit::*},
    };
    use primitives::contract_constants::SOLANA_METAPLEX_AUTH_RULES_PROGRAM_ID;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{
        Asset, Chain, ChainSigner, GasPriceType, NFTAsset, NFTAssetId, NFTImages, NFTResource, NFTType, SignerInput, SolanaNftStandard, SolanaTokenProgramId, TransactionFee,
        TransactionInputType, TransactionLoadInput, TransactionLoadMetadata,
    };
    use solana_primitives::{
        Pubkey,
        instructions::{
            associated_token::get_associated_token_address_with_program_id,
            program_ids::{ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, token_program},
        },
    };

    const NFT_MINT: &str = "HP82kPNXnQcozjDrV4dLYfV6wwABQDMVPJXezDbZXHEy";
    const PNFT_RULE_SET: &str = "Brq4ESPuwPNwBhzvEcY2uM1fXTB171yWuem6U8jEiHiY";
    const CORE_ASSET: &str = "HpYF5mAtjshGy93ce4FWjKg4XkFUocyAKm5BMdQ18d1K";
    const CORE_COLLECTION: &str = "5pQfZttNUtaj8sySRY9RsdtB81aEAQDh2vnacpxiwTpT";

    fn transfer_checked_data() -> Vec<u8> {
        let mut data = vec![12];
        data.extend_from_slice(&1u64.to_le_bytes());
        data.push(0);
        data
    }

    fn nft_asset(token_id: &str, collection: &str) -> NFTAsset {
        let id = NFTAssetId::new(Chain::Solana, collection, token_id);
        NFTAsset {
            id: id.clone(),
            collection_id: id.get_collection_id(),
            contract_address: Some(token_id.to_string()),
            token_id: token_id.to_string(),
            token_type: NFTType::SPL,
            name: "Solana NFT".to_string(),
            description: None,
            chain: Chain::Solana,
            resource: NFTResource::new(String::new(), String::new()),
            images: NFTImages {
                preview: NFTResource::new(String::new(), String::new()),
            },
            attributes: vec![],
        }
    }

    fn signer_input(nft_asset: NFTAsset, metadata: TransactionLoadMetadata) -> SignerInput {
        let input = TransactionLoadInput {
            input_type: TransactionInputType::TransferNft(Asset::from_chain(Chain::Solana), nft_asset),
            sender_address: sender_address(),
            destination_address: TEST_RECIPIENT.to_string(),
            value: "1".to_string(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata,
        };
        SignerInput::new(input, TransactionFee::default())
    }

    #[test]
    fn test_sign_spl_nft_transfer() {
        let signer = SolanaChainSigner;
        let input = signer_input(
            nft_asset(NFT_MINT, NFT_MINT),
            TransactionLoadMetadata::mock_solana_nft(TEST_SENDER_TOKEN_ADDRESS, SolanaTokenProgramId::Token, SolanaNftStandard::NonFungible),
        );

        let result = signer.sign_nft_transfer(&input, &TEST_PRIVATE_KEY).unwrap();

        let transaction = crate::decode_transaction(&result).unwrap();
        let mint = Pubkey::from_base58(NFT_MINT).unwrap();
        let recipient = Pubkey::from_base58(TEST_RECIPIENT).unwrap();
        let recipient_token_address = get_associated_token_address_with_program_id(&recipient, &mint, &token_program());
        assert_eq!(
            (0..transaction.instructions().len()).map(|index| program_id(&transaction, index)).collect::<Vec<_>>(),
            vec![ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID]
        );
        assert_eq!(transaction.instructions()[0].data, vec![1]);
        assert_eq!(account_key(&transaction, 0, 1), recipient_token_address);
        assert_eq!(account_key(&transaction, 1, 0).to_string(), TEST_SENDER_TOKEN_ADDRESS);
        assert_eq!(account_key(&transaction, 1, 1), mint);
        assert_eq!(account_key(&transaction, 1, 2), recipient_token_address);
        assert_eq!(transaction.instructions()[1].data, transfer_checked_data());
    }

    #[test]
    fn test_sign_p_nft_transfer() {
        let signer = SolanaChainSigner;
        let input = signer_input(
            nft_asset(NFT_MINT, NFT_MINT),
            TransactionLoadMetadata::mock_solana_nft(
                TEST_SENDER_TOKEN_ADDRESS,
                SolanaTokenProgramId::Token,
                SolanaNftStandard::ProgrammableNonFungible {
                    rule_set: Some(PNFT_RULE_SET.to_string()),
                },
            ),
        );

        let result = signer.sign_nft_transfer(&input, &TEST_PRIVATE_KEY).unwrap();

        let transaction = crate::decode_transaction(&result).unwrap();
        assert_eq!(program_id(&transaction, 0), METAPLEX_PROGRAM);
        let inst_data = &transaction.instructions()[0].data;
        let mut expected = vec![49, 0];
        expected.extend_from_slice(&1u64.to_le_bytes());
        expected.push(0);
        assert_eq!(inst_data, &expected);

        let mint = Pubkey::from_base58(NFT_MINT).unwrap();
        let source = Pubkey::from_base58(TEST_SENDER_TOKEN_ADDRESS).unwrap();
        let recipient = Pubkey::from_base58(TEST_RECIPIENT).unwrap();
        let recipient_ata = get_associated_token_address_with_program_id(&recipient, &mint, &Pubkey::from_base58(TOKEN_PROGRAM).unwrap());
        let metadata_pda = crate::metaplex::metadata::Metadata::find_pda(mint).unwrap().0;
        let master_edition = crate::metaplex::metadata::Metadata::find_master_edition_pda(mint).unwrap().0;
        let source_record = crate::metaplex::metadata::Metadata::find_token_record_pda(mint, source).unwrap().0;
        let dest_record = crate::metaplex::metadata::Metadata::find_token_record_pda(mint, recipient_ata).unwrap().0;
        let auth_rules = Pubkey::from_base58(PNFT_RULE_SET).unwrap();
        let auth_rules_program = Pubkey::from_base58(SOLANA_METAPLEX_AUTH_RULES_PROGRAM_ID).unwrap();
        let sysvar_instructions = Pubkey::from_base58(SYSVAR_INSTRUCTIONS_ID).unwrap();
        let system_program_pk = Pubkey::from_base58(SYSTEM_PROGRAM_ID).unwrap();
        let sender = Pubkey::from_base58(&sender_address()).unwrap();

        assert_eq!(account_key(&transaction, 0, 0), source);
        assert_eq!(account_key(&transaction, 0, 1), sender);
        assert_eq!(account_key(&transaction, 0, 2), recipient_ata);
        assert_eq!(account_key(&transaction, 0, 3), recipient);
        assert_eq!(account_key(&transaction, 0, 4), mint);
        assert_eq!(account_key(&transaction, 0, 5), metadata_pda);
        assert_eq!(account_key(&transaction, 0, 6), master_edition);
        assert_eq!(account_key(&transaction, 0, 7), source_record);
        assert_eq!(account_key(&transaction, 0, 8), dest_record);
        assert_eq!(account_key(&transaction, 0, 9), sender);
        assert_eq!(account_key(&transaction, 0, 10), sender);
        assert_eq!(account_key(&transaction, 0, 11), system_program_pk);
        assert_eq!(account_key(&transaction, 0, 12), sysvar_instructions);
        assert_eq!(account_key(&transaction, 0, 13).to_string(), TOKEN_PROGRAM);
        assert_eq!(account_key(&transaction, 0, 14).to_string(), ASSOCIATED_TOKEN_PROGRAM_ID);
        assert_eq!(account_key(&transaction, 0, 15), auth_rules_program);
        assert_eq!(account_key(&transaction, 0, 16), auth_rules);
    }

    #[test]
    fn test_sign_core_nft_transfer() {
        let signer = SolanaChainSigner;
        let input = signer_input(nft_asset(CORE_ASSET, CORE_COLLECTION), TransactionLoadMetadata::mock_solana_core_nft(Some(CORE_COLLECTION)));

        let result = signer.sign_nft_transfer(&input, &TEST_PRIVATE_KEY).unwrap();

        let transaction = crate::decode_transaction(&result).unwrap();
        let core_program = Pubkey::from_base58(METAPLEX_CORE_PROGRAM).unwrap();
        let system_program_pk = Pubkey::from_base58(SYSTEM_PROGRAM_ID).unwrap();
        let asset = Pubkey::from_base58(CORE_ASSET).unwrap();
        let collection = Pubkey::from_base58(CORE_COLLECTION).unwrap();
        let recipient = Pubkey::from_base58(TEST_RECIPIENT).unwrap();
        let sender = Pubkey::from_base58(&sender_address()).unwrap();
        assert_eq!(transaction.instructions().len(), 1);
        assert_eq!(program_id(&transaction, 0), METAPLEX_CORE_PROGRAM);
        assert_eq!(transaction.instructions()[0].data, vec![14, 0]);
        assert_eq!(account_key(&transaction, 0, 0), asset);
        assert_eq!(account_key(&transaction, 0, 1), collection);
        assert_eq!(account_key(&transaction, 0, 2), sender);
        assert_eq!(account_key(&transaction, 0, 3), sender);
        assert_eq!(account_key(&transaction, 0, 4), recipient);
        assert_eq!(account_key(&transaction, 0, 5), system_program_pk);
        assert_eq!(account_key(&transaction, 0, 6), core_program);

        let input = signer_input(nft_asset(CORE_ASSET, CORE_ASSET), TransactionLoadMetadata::mock_solana_core_nft(None));
        let result = signer.sign_nft_transfer(&input, &TEST_PRIVATE_KEY).unwrap();
        let transaction = crate::decode_transaction(&result).unwrap();
        assert_eq!(account_key(&transaction, 0, 1), core_program);
    }
}
