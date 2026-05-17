use super::stake_account;
use crate::signer::transaction;
use primitives::{SignerError, SignerInput, StakeType};
use solana_primitives::{Instruction, Pubkey, instructions::memo::memo};

pub(in crate::signer) fn stake(input: &SignerInput, sender: Pubkey) -> Result<Vec<Instruction>, SignerError> {
    let stake_type = input.input_type.get_stake_type().map_err(SignerError::invalid_input)?;
    let mut instructions = transaction::compute_budget_instructions(&input.fee)?;
    match stake_type {
        StakeType::Stake(validator) => {
            let validator = Pubkey::from_base58(&validator.id).map_err(SignerError::from_display)?;
            let stake_account = stake_account::from_blockhash(&sender, input)?;
            let seed = stake_account::seed_from_blockhash(input)?;
            instructions.extend(stake_account::delegate_instructions(sender, validator, stake_account, seed, input.value_as_u64()?)?);
            if let Some(memo_text) = input.get_memo() {
                instructions.push(memo(memo_text, &[]));
            }
        }
        StakeType::Unstake(delegation) => {
            let stake_account = Pubkey::from_base58(&delegation.base.delegation_id).map_err(SignerError::from_display)?;
            instructions.push(stake_account::deactivate_instruction(stake_account, sender)?);
        }
        StakeType::Withdraw(delegation) => {
            let stake_account = Pubkey::from_base58(&delegation.base.delegation_id).map_err(SignerError::from_display)?;
            instructions.push(stake_account::withdraw_instruction(stake_account, sender, sender, input.value_as_u64()?)?);
        }
        StakeType::Redelegate(_) | StakeType::Rewards(_) => {
            return Err(SignerError::invalid_input("unsupported Solana stake action"));
        }
        StakeType::Freeze(_) | StakeType::Unfreeze(_) => {
            return Err(SignerError::invalid_input("Solana does not support freeze operations"));
        }
    }
    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signer::{SolanaChainSigner, testkit::*};
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{
        Asset, Chain, ChainSigner, Delegation, DelegationValidator, GasPriceType, SignerInput, StakeType, TransactionFee, TransactionInputType, TransactionLoadInput,
    };
    use solana_primitives::{
        Pubkey,
        instructions::program_ids::{MEMO_PROGRAM_ID, SYSTEM_PROGRAM_ID as SYSTEM_PROGRAM_ID_STRING},
    };

    // https://github.com/trustwallet/wallet-core/blob/master/rust/tw_tests/tests/chains/solana/solana_sign.rs
    const REFERENCE_STAKE_PRIVATE_KEY: &str = "AevJ4EWcvQ6dptBDvF2Ri5pU6QSBjkzSGHMfbLFKa746";
    const REFERENCE_STAKE_ACCOUNT: &str = "6XMLCn47d5kPi3g4YcjqFvDuxWnpVADpN2tXpeRc4XUB";
    const REFERENCE_VALIDATOR: &str = "4jpwTqt1qZoR7u6u639z2AngYFGN3nakvKhowcnRZDEC";
    const REFERENCE_DELEGATE_STAKE_TX: &str = concat!(
        "j24mVM9Zgu5vDZhPLGGuCRXQnP9djNtxdHh4txN3S7dwJsNNL5fbhzGpPgSUAcLGoMVCfF9TuqTYfpfJnb4sJFe1ahM8yPL5HwuKL6py5AZJFi8SWx9fvaVB699dCPo1GT3JoEBLPCZ9o2jQtnwzLkzTYJnKv2axqhKWFE2sz6TBA5J39eZcjMFUYgyxz6Q5S4MWqYQCb8UET2NAEZoKcfy7j8N25WXL6Gj4j3hBZjpHQQNaGaNEprEqyma3ZuVhpGiCALSsuzVLX3wZVo4icXwe952deMFA4tH3BK1jcSQCgfmcKDJ9nd7bdrnUUs4BoMdF1uDZB5LxE2UH8QiqtYvaUcorF4SJ3gPxM5ykbyPsNK1cSYZF9NMpW2GofyC17eELwnHQTQB2kqphxJZu7BahvkwiDPPeeydiXAkBspJ3nc3PCBujv6WJw22ZHw5j6zAP8ZGnCW44pqtWD5qifF9tTKhySKdANNiWifs3tSCCPQqjfJXu14drNinR6VG8rJxS1qgmRYiRQUa7m1vtoaZFRN5qKUeAfoFKkAVaNnMdwgsNqNH4dqBodTCJFs1LkYwhgRZdZGbwXTn1j7vpR3DSnv4g72i2H556srzK53jdUmdv6yfxt516XDSshqZtHnKZ1tudxKjBXwsqT3imDiZFVka9wKWUAYMCi4XZ79CY6Xpsd9c18U2e9TCngQmgkTATFgrqysfraokNffgqWxvsPMugksbvbPjJs3iCzByvphkC9p7hCf6LwbeF8XnVB91EAgRDA4VLE1f9wkcq5zjy879YWJ4r516h3PQszTz1EaJXNAXdbk5Em7eyuuabGP1Q3nijFTL2yhMDsXpgrjAuEAABNxFMd4J1JRMaic615mHrhwociksrsfQK"
    );
    const REFERENCE_DEACTIVATE_STAKE_TX: &str = "6x3fSstNz4GpPxmT5jHXwyD62uyJMKaPWeBDNNcwXZA9NJ3E7KavCXPNUd8ZYTX5VpkfHKGszkwzM6AdAp4giLD29jvWdNYjkV1Nvb42xFwGD6ryMPZzXkJijaRTrA7SvPTDSRU2haGVmorqkywAXLQUCw47NmBUfLTb5gDcKoBeaAsahckv1eCE746thJVTg2dQNvUTULKF6xckUg7kwFkcUuRe4HCcRgrKcNAUKLR2rEM3brVQkUyAaAtMMtc3gVDXxxpbtW5Fa9wGaEnh31FdRo4z5YBzAUaz7vcrvzF2j81KCPTVnYyTmeJzCzJafzCVCtw";
    const REFERENCE_WITHDRAW_STAKE_TX: &str = "gxr4o1trVP8DGG8UC21AA964YqAPFA3rBCF9MwmBQpn5fDtcujM9wp1gzT466MxWGR8wMciS6dSL771q29eURrEEuvhJzRaFDGPLgVB3UL4gd4T2amPQkR4Dzq5drKEtPJRBR86KVVc2kjDsbWNpdL8S7pZqW3VUijAbm9TS8ezG8NExSCkhxExKhUjXWWguEL4qXra7s2JZfhtmvuJneWnEY3isUVfC9knWtGNwpNFvRvzbH2sgHzwtSsD7mkYrBJoazLCwT8r9yypxycHL41XcGtH425MA16kVSunvvBfzG9PzBTS65YJBs64tzttasCU9uEphkwgmfrmoEC8iKt8xD47Ra79RyXd95yURsaxvpb1tVAH8kMNtj8iV1Pfm";

    fn stake_signer_input(private_key: &[u8], stake_type: StakeType, value: &str) -> SignerInput {
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Stake(Asset::mock_sol(), stake_type),
            sender_address: sender_address_for_key(private_key),
            destination_address: String::new(),
            value: value.to_string(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: solana_metadata(None, None, None),
        };
        SignerInput::new(input, TransactionFee::default())
    }

    fn stake_data(instruction: u32) -> Vec<u8> {
        instruction.to_le_bytes().to_vec()
    }

    fn withdraw_stake_data(lamports: u64) -> Vec<u8> {
        let mut data = stake_data(4);
        data.extend_from_slice(&lamports.to_le_bytes());
        data
    }

    #[test]
    fn test_sign_stake() {
        let signer = SolanaChainSigner;
        let validator = DelegationValidator::stake(Chain::Solana, TEST_RECIPIENT.to_string(), "validator".to_string(), true, 0.0, 0.0);
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Stake(Asset::mock_sol(), StakeType::Stake(validator)),
            sender_address: sender_address(),
            destination_address: String::new(),
            value: "42".to_string(),
            gas_price: GasPriceType::regular(0),
            memo: Some("stake memo".to_string()),
            is_max_value: false,
            metadata: solana_metadata(None, None, None),
        };
        let input = SignerInput::new(input, TransactionFee::default());

        let result = signer.sign_stake(&input, &TEST_PRIVATE_KEY).unwrap();

        let transaction = crate::decode_transaction(&result[0]).unwrap();
        let stake_account = stake_account::from_blockhash(&Pubkey::from_base58(&sender_address()).unwrap(), &input).unwrap();
        assert_eq!(transaction.signatures().len(), 1);
        assert_eq!(
            (0..transaction.instructions().len()).map(|index| program_id(&transaction, index)).collect::<Vec<_>>(),
            vec![SYSTEM_PROGRAM_ID_STRING, crate::STAKE_PROGRAM_ID, crate::STAKE_PROGRAM_ID, MEMO_PROGRAM_ID]
        );
        assert_eq!(account_key(&transaction, 0, 1), stake_account);
        assert_eq!(transaction.instructions()[0].data[0..4], 3u32.to_le_bytes());
        assert_eq!(transaction.instructions()[1].data, {
            let mut data = stake_data(0);
            let authority = Pubkey::from_base58(&sender_address()).unwrap();
            data.extend_from_slice(authority.as_bytes());
            data.extend_from_slice(authority.as_bytes());
            data.extend_from_slice(&0i64.to_le_bytes());
            data.extend_from_slice(&0u64.to_le_bytes());
            data.extend_from_slice(Pubkey::new([0u8; 32]).as_bytes());
            data
        });
        assert_eq!(transaction.instructions()[2].data, stake_data(2));
        assert_eq!(transaction.instructions()[3].accounts, Vec::<u8>::new());
        assert_eq!(transaction.instructions()[3].data, b"stake memo");

        let delegation = Delegation::mock_with_id(TEST_RECIPIENT.to_string());
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Stake(Asset::mock_sol(), StakeType::Unstake(delegation.clone())),
            sender_address: sender_address(),
            destination_address: String::new(),
            value: "0".to_string(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: solana_metadata(None, None, None),
        };
        let input = SignerInput::new(input, TransactionFee::default());
        let result = signer.sign_stake(&input, &TEST_PRIVATE_KEY).unwrap();
        let transaction = crate::decode_transaction(&result[0]).unwrap();
        assert_eq!(program_id(&transaction, 0), crate::STAKE_PROGRAM_ID);
        assert_eq!(transaction.instructions()[0].data, stake_data(5));
        assert_eq!(account_key(&transaction, 0, 0), Pubkey::from_base58(TEST_RECIPIENT).unwrap());

        let input = TransactionLoadInput {
            input_type: TransactionInputType::Stake(Asset::mock_sol(), StakeType::Withdraw(delegation)),
            sender_address: sender_address(),
            destination_address: String::new(),
            value: "55".to_string(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: solana_metadata(None, None, None),
        };
        let input = SignerInput::new(input, TransactionFee::default());
        let result = signer.sign_stake(&input, &TEST_PRIVATE_KEY).unwrap();
        let transaction = crate::decode_transaction(&result[0]).unwrap();
        assert_eq!(program_id(&transaction, 0), crate::STAKE_PROGRAM_ID);
        assert_eq!(transaction.instructions()[0].data, withdraw_stake_data(55));
        assert_eq!(account_key(&transaction, 0, 0), Pubkey::from_base58(TEST_RECIPIENT).unwrap());
        assert_eq!(account_key(&transaction, 0, 1), Pubkey::from_base58(&sender_address()).unwrap());
    }

    #[test]
    fn test_sign_reference_stake() {
        let signer = SolanaChainSigner;
        let private_key = private_key_base58(REFERENCE_STAKE_PRIVATE_KEY);
        let validator = DelegationValidator::stake(Chain::Solana, REFERENCE_VALIDATOR.to_string(), "validator".to_string(), true, 0.0, 0.0);
        let stake = stake_signer_input(&private_key, StakeType::Stake(validator), "42");
        let result = signer.sign_stake(&stake, &private_key).unwrap();
        assert_eq!(base58_transaction(&result[0]), REFERENCE_DELEGATE_STAKE_TX);

        let delegation = Delegation::mock_with_id(REFERENCE_STAKE_ACCOUNT.to_string());
        let unstake = stake_signer_input(&private_key, StakeType::Unstake(delegation.clone()), "0");
        let result = signer.sign_stake(&unstake, &private_key).unwrap();
        assert_eq!(base58_transaction(&result[0]), REFERENCE_DEACTIVATE_STAKE_TX);

        let withdraw = stake_signer_input(&private_key, StakeType::Withdraw(delegation), "42");
        let result = signer.sign_stake(&withdraw, &private_key).unwrap();
        assert_eq!(base58_transaction(&result[0]), REFERENCE_WITHDRAW_STAKE_TX);
    }
}
