use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::address::TronAddress;
use crate::signer::transaction::TronPayload;

const TRIGGER_SMART_CONTRACT: &str = "TriggerSmartContract";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronSmartContractCall {
    pub contract_address: String,
    pub function_selector: String,
    pub parameter: Option<String>,
    pub fee_limit: Option<u32>,
    pub call_value: Option<u32>,
    pub owner_address: String,
    pub visible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronSmartContractResult {
    pub result: TronSmartContractResultMessage,
    pub constant_result: Vec<String>,
    pub energy_used: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronSmartContractResultMessage {
    pub result: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerSmartContractData {
    pub contract_address: String,
    pub data: String,
    pub owner_address: String,
    pub fee_limit: Option<u64>,
    pub call_value: Option<u64>,
}

impl TriggerSmartContractData {
    pub fn from_payload(
        data: Option<&[u8]>,
        sender_address: &str,
    ) -> Result<Option<Self>, Box<dyn Error + Send + Sync>> {
        let Some(data) = data else {
            return Ok(None);
        };
        let Ok(payload) = serde_json::from_slice::<TronPayload>(data) else {
            return Ok(None);
        };
        let Some(raw_data) = payload.transaction.raw_data.as_ref() else {
            return Ok(None);
        };
        let Some(contract) = raw_data.contract.first() else {
            return Ok(None);
        };
        if contract.contract_type != TRIGGER_SMART_CONTRACT {
            return Ok(None);
        }

        let value = &contract.parameter.value;
        let Some(contract_address) = value.contract_address.as_deref().and_then(TronAddress::from_hex) else {
            return Err("Invalid Tron contract address".into());
        };
        let Some(data) = value.data.as_deref() else {
            return Ok(None);
        };
        let owner_address = if payload.address.is_empty() {
            match value.owner_address.as_deref() {
                Some(address) => TronAddress::from_hex(address).ok_or("Invalid Tron owner address")?,
                None => sender_address.to_string(),
            }
        } else {
            payload.address
        };

        Ok(Some(Self {
            contract_address,
            data: data.to_string(),
            owner_address,
            fee_limit: raw_data.fee_limit,
            call_value: value.call_value,
        }))
    }
}
