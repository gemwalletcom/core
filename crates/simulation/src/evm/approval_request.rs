use num_bigint::BigInt;
use primitives::{
    AssetId, Chain, SimulationHeader, SimulationPayloadField, SimulationPayloadFieldDisplay, SimulationPayloadFieldKind, SimulationPayloadFieldType, SimulationResult,
    SimulationSeverity, SimulationWarning, SimulationWarningType,
};

use super::{approval_method::ApprovalMethod, approval_value::ApprovalValue};
use gem_evm::ethereum_address_checksum;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ApprovalRequest {
    asset_id: AssetId,
    contract_address: String,
    token_address: Option<String>,
    pub(crate) spender_address: String,
    method: ApprovalMethod,
    approval_value: Option<ApprovalValue>,
    expiration: Option<String>,
}

impl ApprovalRequest {
    pub(crate) fn erc20(chain: Chain, contract_address: &str, spender_address: String, approval_value: String) -> Option<Self> {
        let contract_address = ethereum_address_checksum(contract_address).ok()?;
        let spender_address = ethereum_address_checksum(&spender_address).ok()?;
        Some(Self {
            asset_id: AssetId::from_token(chain, &contract_address),
            contract_address,
            token_address: None,
            spender_address,
            method: ApprovalMethod::Approve,
            approval_value: ApprovalValue::from_raw(&approval_value),
            expiration: None,
        })
    }

    pub(crate) fn nft_collection(chain: Chain, contract_address: &str, spender_address: String) -> Option<Self> {
        let contract_address = ethereum_address_checksum(contract_address).ok()?;
        let spender_address = ethereum_address_checksum(&spender_address).ok()?;
        Some(Self {
            asset_id: AssetId::from_token(chain, &contract_address),
            contract_address,
            token_address: None,
            spender_address,
            method: ApprovalMethod::SetApprovalForAll,
            approval_value: None,
            expiration: None,
        })
    }

    pub(crate) fn permit(
        chain: Chain,
        contract_address: String,
        spender_address: String,
        approval_value: String,
        expiration: Option<String>,
        token_address: Option<String>,
        method: ApprovalMethod,
    ) -> Option<Self> {
        let contract_address = ethereum_address_checksum(&contract_address).ok()?;
        let spender_address = ethereum_address_checksum(&spender_address).ok()?;
        let token_address = token_address.map(|value| ethereum_address_checksum(&value)).transpose().ok()?;
        let asset_address = token_address.clone().unwrap_or_else(|| contract_address.clone());
        let token_address = (asset_address != contract_address).then_some(asset_address.clone());
        Some(Self {
            asset_id: AssetId::from_token(chain, &asset_address),
            contract_address,
            token_address,
            spender_address,
            method,
            approval_value: ApprovalValue::from_raw(&approval_value),
            expiration,
        })
    }

    pub(crate) fn permit_batch(chain: Chain, contract_address: String, spender_address: String, approval_value: ApprovalValue, token_address: Option<String>) -> Option<Self> {
        let contract_address = ethereum_address_checksum(&contract_address).ok()?;
        let spender_address = ethereum_address_checksum(&spender_address).ok()?;
        let token_address = token_address.map(|value| ethereum_address_checksum(&value)).transpose().ok()?;
        let asset_address = token_address.clone().unwrap_or_else(|| contract_address.clone());

        Some(Self {
            asset_id: AssetId::from_token(chain, &asset_address),
            contract_address,
            token_address,
            spender_address,
            method: ApprovalMethod::PermitBatch,
            approval_value: Some(approval_value),
            expiration: None,
        })
    }

    pub(crate) fn simulate(self) -> SimulationResult {
        let warning = self.primary_warning();
        self.build_simulation_result(vec![warning])
    }

    pub(crate) fn build_simulation_result(self, warnings: Vec<SimulationWarning>) -> SimulationResult {
        let mut result = SimulationResult::new(warnings, self.payload());

        if self.method.supports_value_display()
            && let Some(approval_value) = self.approval_value
        {
            result.header = Some(SimulationHeader {
                asset_id: self.asset_id,
                value: approval_value.display_value(),
            });
        }

        result
    }

    pub(crate) fn primary_warning(&self) -> SimulationWarning {
        let warning = match self.method {
            ApprovalMethod::Approve => SimulationWarningType::TokenApproval {
                asset_id: self.asset_id.clone(),
                value: self.warning_approval_value(),
            },
            ApprovalMethod::SetApprovalForAll => SimulationWarningType::NftCollectionApproval { asset_id: self.asset_id.clone() },
            ApprovalMethod::Permit | ApprovalMethod::PermitSingle => SimulationWarningType::PermitApproval {
                asset_id: self.asset_id.clone(),
                value: self.warning_approval_value(),
            },
            ApprovalMethod::PermitBatch => SimulationWarningType::PermitBatchApproval {
                value: self.warning_approval_value(),
            },
        };

        SimulationWarning::new(
            match self.method {
                ApprovalMethod::Approve => SimulationSeverity::Low,
                ApprovalMethod::SetApprovalForAll | ApprovalMethod::Permit | ApprovalMethod::PermitSingle | ApprovalMethod::PermitBatch => SimulationSeverity::Warning,
            },
            warning,
            None,
        )
    }

    fn warning_approval_value(&self) -> Option<BigInt> {
        match self.approval_value.as_ref() {
            Some(ApprovalValue::Exact(value)) => Some(BigInt::from(value.clone())),
            Some(ApprovalValue::Unlimited) | None => None,
        }
    }

    fn payload(&self) -> Vec<SimulationPayloadField> {
        let mut payload = vec![
            SimulationPayloadField::standard(
                SimulationPayloadFieldKind::Contract,
                &self.contract_address,
                SimulationPayloadFieldType::Address,
                SimulationPayloadFieldDisplay::Primary,
            ),
            SimulationPayloadField::standard(
                SimulationPayloadFieldKind::Method,
                self.method.to_string(),
                SimulationPayloadFieldType::Text,
                SimulationPayloadFieldDisplay::Primary,
            ),
        ];

        if let Some(token_address) = self.token_address.as_deref() {
            payload.push(SimulationPayloadField::standard(
                SimulationPayloadFieldKind::Token,
                token_address,
                SimulationPayloadFieldType::Address,
                SimulationPayloadFieldDisplay::Primary,
            ));
        }

        payload.push(SimulationPayloadField::standard(
            SimulationPayloadFieldKind::Spender,
            &self.spender_address,
            SimulationPayloadFieldType::Address,
            SimulationPayloadFieldDisplay::Primary,
        ));

        if self.method.supports_value_display()
            && let Some(approval_value) = self.approval_value.as_ref()
        {
            payload.push(SimulationPayloadField::standard(
                SimulationPayloadFieldKind::Value,
                approval_value.display_value(),
                SimulationPayloadFieldType::Text,
                SimulationPayloadFieldDisplay::Secondary,
            ));
        }

        if self.method.supports_value_display()
            && let Some(expiration) = self.expiration.as_deref()
        {
            payload.push(SimulationPayloadField::custom(
                "expiration",
                expiration,
                SimulationPayloadFieldType::Timestamp,
                SimulationPayloadFieldDisplay::Secondary,
            ));
        }

        payload
    }
}
