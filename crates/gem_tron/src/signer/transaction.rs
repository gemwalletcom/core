use gem_hash::sha2::sha256;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use primitives::{Address as _, SignerError, SignerInput, StakeType, TransactionLoadMetadata, TronStakeData};
use signer::{SignatureScheme, Signer};

use crate::address::TronAddress;
use crate::models::{SignedTransactionJson, TronContract, TronRawData, TronResource, WalletConnectPayload};

const ABI_WORD_LEN: usize = 32;
const TRC20_TRANSFER_SELECTOR: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];

pub(crate) fn sign_transfer(input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
    sign_native_transfer(input, &input.destination_address, private_key)
}

pub(crate) fn sign_token_transfer(input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
    sign_token_transfer_to(input, &input.destination_address, private_key)
}

pub(crate) fn sign_swap(input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
    let swap = input.input_type.get_swap_data().map_err(SignerError::invalid_input)?;
    let from_asset = input.input_type.get_asset();

    let result = if from_asset.id.is_token() {
        sign_token_transfer_to(input, &swap.data.to, private_key)?
    } else {
        sign_native_transfer(input, &swap.data.to, private_key)?
    };

    Ok(vec![result])
}

fn sign_native_transfer(input: &SignerInput, destination: &str, private_key: &[u8]) -> Result<String, SignerError> {
    let owner = validate_sender(input, private_key)?;
    let contract = TronContract::Transfer {
        owner,
        to: TronAddress::parse(destination)?,
        amount: input.value_as_u64()?,
    };
    let fee_limit = input.fee.fee.to_u64().ok_or_else(|| SignerError::invalid_input("invalid Tron fee"))?;
    sign_contract(input, contract, fee_limit, private_key)
}

fn sign_token_transfer_to(input: &SignerInput, destination: &str, private_key: &[u8]) -> Result<String, SignerError> {
    let owner = validate_sender(input, private_key)?;
    let token_id = input.input_type.get_asset().id.get_token_id()?;
    let destination = TronAddress::parse(destination)?;
    let contract = TronContract::TriggerSmart {
        owner,
        contract: TronAddress::parse(token_id)?,
        data: encode_trc20_transfer(&destination, &input.value)?,
        call_value: None,
        call_token_value: None,
        token_id: None,
    };
    let fee_limit = input.fee.gas_limit.to_u64().ok_or_else(|| SignerError::invalid_input("invalid Tron fee limit"))?;
    sign_contract(input, contract, fee_limit, private_key)
}

pub(crate) fn sign_stake(input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
    let stake_type = input.input_type.get_stake_type().map_err(SignerError::invalid_input)?;
    let owner = validate_sender(input, private_key)?;
    let fee_limit = input.fee.fee.to_u64().ok_or_else(|| SignerError::invalid_input("invalid Tron fee"))?;
    let TransactionLoadMetadata::Tron { stake_data, .. } = &input.metadata else {
        return SignerError::invalid_input_err("Missing tron metadata");
    };

    let contracts = match stake_type {
        StakeType::Stake(_) | StakeType::Redelegate(_) => match stake_data {
            TronStakeData::Votes(votes) => vec![TronContract::vote_witness(owner, votes)?],
            TronStakeData::Unfreeze(_) => return SignerError::invalid_input_err("Expected Tron vote stake data"),
        },
        StakeType::Unstake(_) => match stake_data {
            TronStakeData::Votes(votes) => vec![TronContract::vote_witness(owner, votes)?],
            TronStakeData::Unfreeze(unfreezes) => unfreezes
                .iter()
                .map(|unfreeze| TronContract::UnfreezeBalanceV2 {
                    owner,
                    unfreeze_balance: unfreeze.amount,
                    resource: TronResource::from(&unfreeze.resource),
                })
                .collect(),
        },
        StakeType::Rewards(_) => vec![TronContract::WithdrawBalance { owner }],
        StakeType::Withdraw(_) => vec![TronContract::WithdrawExpireUnfreeze { owner }],
        StakeType::Freeze(resource) => vec![TronContract::FreezeBalanceV2 {
            owner,
            frozen_balance: input.value_as_u64()?,
            resource: TronResource::from(resource),
        }],
        StakeType::Unfreeze(resource) => vec![TronContract::UnfreezeBalanceV2 {
            owner,
            unfreeze_balance: input.value_as_u64()?,
            resource: TronResource::from(resource),
        }],
    };

    contracts.into_iter().map(|contract| sign_contract(input, contract, fee_limit, private_key)).collect()
}

pub(crate) fn sign_data(input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
    validate_sender(input, private_key)?;
    let payload = WalletConnectPayload::parse(input)?;
    let transaction_hash = payload.transaction_hash()?;
    let signature = sign_raw_hash(&transaction_hash, private_key)?;
    payload.into_output(transaction_hash, signature)
}

fn validate_sender(input: &SignerInput, private_key: &[u8]) -> Result<TronAddress, SignerError> {
    let sender = TronAddress::parse(&input.sender_address)?;
    if sender != TronAddress::from_private_key(private_key)? {
        return SignerError::invalid_input_err("Tron sender address does not match private key");
    }
    Ok(sender)
}

fn sign_contract(input: &SignerInput, contract: TronContract, fee_limit: u64, private_key: &[u8]) -> Result<String, SignerError> {
    let raw_data = TronRawData::from_input(input, contract, fee_limit)?;
    let raw_data_bytes = raw_data.encode();
    let transaction_id = sha256(&raw_data_bytes);
    let signature = sign_raw_hash(&transaction_id, private_key)?;

    serde_json::to_string(&SignedTransactionJson::new(raw_data.json(), &raw_data_bytes, &transaction_id, signature)).map_err(Into::into)
}

fn sign_raw_hash(hash: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
    Ok(hex::encode(Signer::sign_digest(SignatureScheme::Secp256k1, hash, private_key)?))
}

fn encode_trc20_transfer(destination: &TronAddress, value: &str) -> Result<Vec<u8>, SignerError> {
    let mut data = TRC20_TRANSFER_SELECTOR.to_vec();
    data.extend(pad_left(destination.as_bytes(), ABI_WORD_LEN)?);
    data.extend(pad_left(
        &value.parse::<BigUint>().map_err(|_| SignerError::invalid_input("invalid TRC20 amount"))?.to_bytes_be(),
        ABI_WORD_LEN,
    )?);
    Ok(data)
}

fn pad_left(data: &[u8], len: usize) -> Result<Vec<u8>, SignerError> {
    if data.len() > len {
        return SignerError::invalid_input_err("value does not fit padded length");
    }
    let mut padded = vec![0u8; len - data.len()];
    padded.extend_from_slice(data);
    Ok(padded)
}
