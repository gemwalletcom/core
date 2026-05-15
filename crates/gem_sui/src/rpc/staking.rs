use std::{error::Error, str::FromStr};

use num_bigint::BigUint;
use sui_types::Address;

use super::client::{PATH_LIST_OWNED_OBJECTS, PATH_SIMULATE_TRANSACTION, SuiClient};
use super::mapper::timestamp_millis;
use super::proto::{
    self as proto, Argument, FieldMask, Input, ListOwnedObjectsRequest, ListOwnedObjectsResponse, MoveCall, ProgrammableTransaction, SimulateTransactionRequest,
    SimulateTransactionResponse, Transaction as GrpcTransaction, TransactionChecks, TransactionKind, WithMut,
};
use crate::models::staking::{SuiStake, SuiStakeDelegation, SuiStakeStatus, SuiSystemState, SuiValidator, SuiValidators};
use crate::{SUI_SYSTEM_ID, sui_system_package_address, sui_system_state_object_id};

const STAKED_SUI_TYPE: &str = "0x3::staking_pool::StakedSui";

#[derive(Debug, serde::Deserialize)]
struct StakedSuiObject {
    id: Address,
    pool_id: Address,
    stake_activation_epoch: u64,
    principal: u64,
}

struct DelegatedStake {
    staked_sui_id: String,
    validator_address: String,
    staking_pool: String,
    activation_epoch: u64,
    principal: u64,
    rewards: u64,
}

impl SuiClient {
    pub async fn get_stake_delegations(&self, address: String) -> Result<Vec<SuiStakeDelegation>, Box<dyn Error + Send + Sync>> {
        let address = Address::from_str(&address)?;
        let delegations = self.list_delegated_stake(&address).await?;
        Ok(delegations
            .into_iter()
            .map(|delegation| {
                let rewards = delegation.rewards;
                SuiStakeDelegation {
                    validator_address: delegation.validator_address,
                    staking_pool: delegation.staking_pool,
                    stakes: vec![SuiStake {
                        staked_sui_id: delegation.staked_sui_id,
                        status: SuiStakeStatus::Active,
                        principal: BigUint::from(delegation.principal),
                        stake_request_epoch: delegation.activation_epoch.to_string(),
                        stake_active_epoch: delegation.activation_epoch.to_string(),
                        estimated_reward: Some(BigUint::from(rewards)),
                    }],
                }
            })
            .collect())
    }

    pub async fn get_validators(&self) -> Result<SuiValidators, Box<dyn Error + Send + Sync>> {
        let epoch = self.get_epoch(Some("system_state.validators".to_string())).await?;
        let apys = epoch
            .system_state
            .and_then(|state| state.validators)
            .map(|validators| {
                validators
                    .active_validators
                    .into_iter()
                    .filter_map(|validator| {
                        Some(SuiValidator {
                            address: validator.address?,
                            apy: 0.0,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(SuiValidators { apys })
    }

    pub async fn get_system_state(&self) -> Result<SuiSystemState, Box<dyn Error + Send + Sync>> {
        let epoch = self.get_epoch(Some("epoch,start,end".to_string())).await?;
        let start_ms = epoch.start.as_ref().map(timestamp_millis).unwrap_or_default().to_string();
        let duration_ms = match (epoch.start.as_ref(), epoch.end.as_ref()) {
            (Some(start), Some(end)) => (timestamp_millis(end) - timestamp_millis(start)).max(0).to_string(),
            _ => "0".to_string(),
        };
        Ok(SuiSystemState {
            epoch: epoch.epoch.unwrap_or_default().to_string(),
            epoch_start_timestamp_ms: start_ms,
            epoch_duration_ms: duration_ms,
        })
    }

    async fn list_delegated_stake(&self, address: &Address) -> Result<Vec<DelegatedStake>, Box<dyn Error + Send + Sync>> {
        let mut request = ListOwnedObjectsRequest {
            owner: Some(address.to_string()),
            page_size: Some(500),
            read_mask: Some(FieldMask::from_path_string("contents")),
            object_type: Some(STAKED_SUI_TYPE.to_string()),
            ..Default::default()
        };
        let mut objects = Vec::new();

        loop {
            let response: ListOwnedObjectsResponse = self.grpc_unary(PATH_LIST_OWNED_OBJECTS, request.clone()).await?;
            objects.extend(response.objects);
            if response.next_page_token.is_none() {
                break;
            }
            request.page_token = response.next_page_token;
        }

        self.create_delegated_stakes(objects).await
    }

    async fn create_delegated_stakes(&self, objects: Vec<proto::Object>) -> Result<Vec<DelegatedStake>, Box<dyn Error + Send + Sync>> {
        let staked_sui = objects
            .into_iter()
            .map(|object| {
                object
                    .contents
                    .ok_or("missing Sui staked object contents")?
                    .deserialize::<StakedSuiObject>()
                    .map_err(|error| error.into())
            })
            .collect::<Result<Vec<_>, Box<dyn Error + Send + Sync>>>()?;
        let ids = staked_sui.iter().map(|stake| stake.id).collect::<Vec<_>>();
        let pool_ids = staked_sui.iter().map(|stake| stake.pool_id).collect::<Vec<_>>();
        let rewards = self.calculate_rewards(&ids).await?;
        let validator_addresses = self.get_validator_address_by_pool_id(&pool_ids).await?;

        Ok(staked_sui
            .into_iter()
            .zip(rewards)
            .zip(validator_addresses)
            .map(|((stake, (_id, rewards)), (_pool_id, validator_address))| DelegatedStake {
                staked_sui_id: stake.id.to_string(),
                validator_address: validator_address.to_string(),
                staking_pool: stake.pool_id.to_string(),
                activation_epoch: stake.stake_activation_epoch,
                principal: stake.principal,
                rewards,
            })
            .collect())
    }

    async fn calculate_rewards(&self, staked_sui_ids: &[Address]) -> Result<Vec<(Address, u64)>, Box<dyn Error + Send + Sync>> {
        let mut ptb = ProgrammableTransaction {
            inputs: vec![Input::object_id(sui_system_state_object_id())],
            ..Default::default()
        };
        let system_object = Argument::new_input(0);

        for id in staked_sui_ids {
            let staked_sui = Argument::new_input(ptb.inputs.len() as u16);
            ptb.inputs.push(Input::object_id(id));
            ptb.commands
                .push(MoveCall::from_parts(sui_system_package_address(), SUI_SYSTEM_ID, "calculate_rewards", vec![system_object, staked_sui]).into());
        }

        let response = self.simulate_staking_transaction(ptb).await?;
        if staked_sui_ids.len() != response.command_outputs.len() {
            return Err("missing Sui rewards command outputs".into());
        }

        staked_sui_ids
            .iter()
            .zip(response.command_outputs)
            .map(|(id, output)| {
                let rewards = output.return_values.first().and_then(|value| value.value.as_ref()).ok_or("missing Sui rewards BCS value")?;
                if rewards.name.as_deref() != Some("u64") || rewards.value.as_ref().map(|value| value.len()) != Some(size_of::<u64>()) {
                    return Err("invalid Sui rewards BCS value".into());
                }
                let value = rewards.value.as_ref().ok_or("missing Sui rewards bytes")?;
                let bytes: [u8; size_of::<u64>()] = value.as_slice().try_into()?;
                Ok((*id, u64::from_le_bytes(bytes)))
            })
            .collect()
    }

    async fn get_validator_address_by_pool_id(&self, pool_ids: &[Address]) -> Result<Vec<(Address, Address)>, Box<dyn Error + Send + Sync>> {
        let mut ptb = ProgrammableTransaction {
            inputs: vec![Input::object_id(sui_system_state_object_id())],
            ..Default::default()
        };
        let system_object = Argument::new_input(0);

        for id in pool_ids {
            let pool_id = Argument::new_input(ptb.inputs.len() as u16);
            ptb.inputs.push(Input::pure(id.into_inner().to_vec()));
            ptb.commands
                .push(MoveCall::from_parts(sui_system_package_address(), SUI_SYSTEM_ID, "validator_address_by_pool_id", vec![system_object, pool_id]).into());
        }

        let response = self.simulate_staking_transaction(ptb).await?;
        if pool_ids.len() != response.command_outputs.len() {
            return Err("missing Sui validator address command outputs".into());
        }

        pool_ids
            .iter()
            .zip(response.command_outputs)
            .map(|(id, output)| {
                let address = output
                    .return_values
                    .first()
                    .and_then(|value| value.value.as_ref())
                    .ok_or("missing Sui validator address BCS value")?;
                if address.name.as_deref() != Some("address") || address.value.as_ref().map(|value| value.len()) != Some(Address::LENGTH) {
                    return Err("invalid Sui validator address BCS value".into());
                }
                let value = address.value.as_ref().ok_or("missing Sui validator address bytes")?;
                Ok((*id, Address::from_bytes(value)?))
            })
            .collect()
    }

    async fn simulate_staking_transaction(&self, ptb: ProgrammableTransaction) -> Result<proto::SimulateTransactionResponse, Box<dyn Error + Send + Sync>> {
        let transaction = GrpcTransaction::from_kind(TransactionKind::programmable_transaction(ptb), "0x0");
        let request = SimulateTransactionRequest::new(transaction).with(|request| {
            request.read_mask = Some(FieldMask::from_paths(["command_outputs.return_values.value", "transaction.effects.status"]));
            request.checks = Some(TransactionChecks::Disabled);
        });
        let response: SimulateTransactionResponse = self.grpc_unary(PATH_SIMULATE_TRANSACTION, request).await?;
        if !response
            .transaction
            .as_ref()
            .and_then(|transaction| transaction.effects.as_ref())
            .and_then(|effects| effects.status.as_ref())
            .and_then(|status| status.success)
            .unwrap_or(false)
        {
            return Err("Sui staking transaction simulation failed".into());
        }
        Ok(response)
    }
}
