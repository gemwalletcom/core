use super::DEFAULT_FILL_TIMEOUT;
use crate::{
    SwapperError, SwapperQuoteData,
    alien::RpcProvider,
    models::Quote,
    solana::tx_builder,
};
use alloy_primitives::FixedBytes;
use borsh::BorshSerialize;
use gem_evm::across::{
    contracts::V3SpokePoolInterface::V3RelayData,
    deployment::AcrossDeployment,
};
use gem_hash::keccak;
use sha2::{Digest, Sha256};
use solana_primitives::{
    instructions::{associated_token::get_associated_token_address, program_ids},
    types::{find_program_address, AccountMeta, Instruction as SolInstruction, Pubkey as SolanaPubkey},
};
use solana_primitives::instructions::token::TokenInstruction;
use std::{str::FromStr, sync::Arc};

const SVM_SPOKE_STATE_SEED: u64 = 0;
const SVM_DELEGATE_SEED: &[u8] = b"delegate";
const SVM_EVENT_AUTHORITY_SEED: &[u8] = b"__event_authority";

pub async fn build_deposit_tx(
    rpc_provider: Arc<dyn RpcProvider>,
    quote: &Quote,
    v3_relay_data: &V3RelayData,
) -> Result<SwapperQuoteData, SwapperError> {
    let depositor = SolanaPubkey::from_str(&quote.request.wallet_address)
        .map_err(|_| SwapperError::InvalidAddress(quote.request.wallet_address.clone()))?;

    let origin_chain = quote.request.from_asset.chain();
    let deployment = AcrossDeployment::deployment_by_chain(&origin_chain).ok_or(SwapperError::NotSupportedChain)?;
    let spoke_pool_program = SolanaPubkey::from_str(deployment.spoke_pool)
        .map_err(|_| SwapperError::InvalidAddress(deployment.spoke_pool.to_string()))?;

    let recipient = solana_pubkey_from_fixed_bytes(&v3_relay_data.recipient);
    let input_token = solana_pubkey_from_fixed_bytes(&v3_relay_data.inputToken);
    let output_token = solana_pubkey_from_fixed_bytes(&v3_relay_data.outputToken);

    let input_amount: u64 = v3_relay_data
        .inputAmount
        .try_into()
        .map_err(|_| SwapperError::InvalidAmount("Input amount overflow".into()))?;
    let output_amount = v3_relay_data.outputAmount.to_be_bytes::<32>();

    let destination_chain_id = AcrossDeployment::deployment_by_chain(&quote.request.to_asset.chain())
        .ok_or(SwapperError::NotSupportedChain)?
        .chain_id;
    let quote_timestamp = v3_relay_data
        .fillDeadline
        .checked_sub(DEFAULT_FILL_TIMEOUT)
        .ok_or(SwapperError::InvalidRoute)?;
    let fill_deadline = v3_relay_data.fillDeadline;
    let exclusivity_parameter = v3_relay_data.exclusivityDeadline;
    let exclusive_relayer = SolanaPubkey::new([0u8; 32]);
    let message = v3_relay_data.message.as_ref().to_vec();

    let deposit_seed_data = DepositSeedData {
        depositor,
        recipient,
        input_token,
        output_token,
        input_amount,
        output_amount,
        destination_chain_id,
        exclusive_relayer,
        quote_timestamp,
        fill_deadline,
        exclusivity_parameter,
        message: &message,
    };
    let seed_hash = deposit_seed_hash(&deposit_seed_data)?;
    let (delegate, _) = find_program_address(&spoke_pool_program, &[SVM_DELEGATE_SEED, &seed_hash])
        .map_err(|_| SwapperError::TransactionError("Failed to derive delegate PDA".into()))?;

    let (state, _) = find_program_address(&spoke_pool_program, &[b"state", SVM_SPOKE_STATE_SEED.to_le_bytes().as_ref()])
        .map_err(|_| SwapperError::TransactionError("Failed to derive state PDA".into()))?;
    let (event_authority, _) = find_program_address(&spoke_pool_program, &[SVM_EVENT_AUTHORITY_SEED])
        .map_err(|_| SwapperError::TransactionError("Failed to derive event authority PDA".into()))?;

    let depositor_token_account = get_associated_token_address(&depositor, &input_token);
    let vault = get_associated_token_address(&state, &input_token);

    let token_decimals = AcrossDeployment::asset_mappings()
        .into_iter()
        .find(|mapping| mapping.set.contains(&quote.request.from_asset.asset_id()))
        .and_then(|mapping| u8::try_from(mapping.capital_cost.decimals).ok())
        .ok_or_else(|| SwapperError::ComputeQuoteError("Unsupported token decimals".into()))?;

    let approve_ix = approve_checked_instruction(
        &depositor_token_account,
        &input_token,
        &delegate,
        &depositor,
        input_amount,
        token_decimals,
    );

    let deposit_args = borsh_encode(&deposit_seed_data)?;
    let mut deposit_data = Vec::with_capacity(8 + deposit_args.len());
    deposit_data.extend_from_slice(&anchor_discriminator("deposit"));
    deposit_data.extend_from_slice(&deposit_args);

    let deposit_ix = SolInstruction {
        program_id: spoke_pool_program,
        accounts: vec![
            AccountMeta {
                pubkey: depositor,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: delegate,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: depositor_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: input_token,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: SolanaPubkey::from_base58(program_ids::TOKEN_PROGRAM_ID).unwrap(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: SolanaPubkey::from_base58(program_ids::ASSOCIATED_TOKEN_PROGRAM_ID).unwrap(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: SolanaPubkey::from_base58(program_ids::SYSTEM_PROGRAM_ID).unwrap(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: event_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: spoke_pool_program,
                is_signer: false,
                is_writable: false,
            },
        ],
        data: deposit_data,
    };

    let tx_b64 = tx_builder::build_base64_transaction(depositor, vec![approve_ix, deposit_ix], rpc_provider)
        .await
        .map_err(SwapperError::TransactionError)?;

    Ok(SwapperQuoteData::new_contract(
        deployment.spoke_pool.to_string(),
        "".to_string(),
        tx_b64,
        None,
        None,
    ))
}

fn anchor_discriminator(name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("global:{name}"));
    let hash = hasher.finalize();
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&hash[..8]);
    discriminator
}

#[derive(BorshSerialize)]
struct DepositSeedData<'a> {
    depositor: SolanaPubkey,
    recipient: SolanaPubkey,
    input_token: SolanaPubkey,
    output_token: SolanaPubkey,
    input_amount: u64,
    output_amount: [u8; 32],
    destination_chain_id: u64,
    exclusive_relayer: SolanaPubkey,
    quote_timestamp: u32,
    fill_deadline: u32,
    exclusivity_parameter: u32,
    message: &'a [u8],
}

fn borsh_encode<T: BorshSerialize>(value: &T) -> Result<Vec<u8>, SwapperError> {
    let mut data = Vec::new();
    value.serialize(&mut data).map_err(|e| SwapperError::TransactionError(e.to_string()))?;
    Ok(data)
}

fn deposit_seed_hash(seed_data: &DepositSeedData<'_>) -> Result<[u8; 32], SwapperError> {
    let data = borsh_encode(seed_data)?;
    Ok(keccak::keccak256(&data))
}

fn solana_pubkey_from_fixed_bytes(bytes: &FixedBytes<32>) -> SolanaPubkey {
    SolanaPubkey::new(bytes.0)
}

fn approve_checked_instruction(
    source: &SolanaPubkey,
    mint: &SolanaPubkey,
    delegate: &SolanaPubkey,
    owner: &SolanaPubkey,
    amount: u64,
    decimals: u8,
) -> SolInstruction {
    let accounts = vec![
        AccountMeta {
            pubkey: *source,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *mint,
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: *delegate,
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: *owner,
            is_signer: true,
            is_writable: false,
        },
    ];
    let data = TokenInstruction::ApproveChecked { amount, decimals }.serialize();
    SolInstruction {
        program_id: SolanaPubkey::from_base58(program_ids::TOKEN_PROGRAM_ID).unwrap(),
        accounts,
        data,
    }
}
