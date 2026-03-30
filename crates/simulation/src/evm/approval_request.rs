use num_bigint::BigInt;
use primitives::{
    AssetId, Chain, SimulationPayloadField, SimulationPayloadFieldDisplay, SimulationPayloadFieldKind, SimulationPayloadFieldType, SimulationResult, SimulationSeverity,
    SimulationWarning, SimulationWarningApproval, SimulationWarningType,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{approval_method::ApprovalMethod, approval_value::ApprovalValue};
use gem_evm::ethereum_address_checksum;

const EXCESSIVE_EXPIRATION_WINDOW: Duration = Duration::from_secs(60 * 60 * 24 * 30);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ApprovalRequest {
    asset_id: AssetId,
    contract_address: String,
    token_address: Option<String>,
    pub(crate) spender_address: String,
    method: ApprovalMethod,
    approval_value: Option<ApprovalValue>,
    display_expiration: Option<u64>,
    warning_expiration: Option<u64>,
}

impl ApprovalRequest {
    pub(crate) fn erc20(chain: Chain, contract_address: &str, spender_address: String, approval_value: String) -> Option<Self> {
        Self::new(
            chain,
            ApprovalContext {
                contract_address: contract_address.to_string(),
                spender_address,
                token_address: None,
                approval_value: ApprovalValue::from_raw(&approval_value),
                display_expiration: None,
                warning_expiration: None,
                method: ApprovalMethod::Approve,
                token_field: TokenField::AlwaysHide,
            },
        )
    }

    pub(crate) fn nft_collection(chain: Chain, contract_address: &str, spender_address: String) -> Option<Self> {
        Self::new(
            chain,
            ApprovalContext {
                contract_address: contract_address.to_string(),
                spender_address,
                token_address: None,
                approval_value: None,
                display_expiration: None,
                warning_expiration: None,
                method: ApprovalMethod::SetApprovalForAll,
                token_field: TokenField::AlwaysHide,
            },
        )
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
        Self::new(
            chain,
            ApprovalContext {
                contract_address,
                spender_address,
                token_address,
                approval_value: ApprovalValue::from_raw(&approval_value),
                display_expiration: expiration.as_deref().map(str::parse).transpose().ok()?,
                warning_expiration: expiration.as_deref().map(str::parse).transpose().ok()?,
                method,
                token_field: TokenField::HideWhenMatchingContract,
            },
        )
    }

    pub(crate) fn permit_batch(
        chain: Chain,
        contract_address: String,
        spender_address: String,
        approval_value: ApprovalValue,
        token_address: Option<String>,
        warning_expiration: Option<u64>,
    ) -> Option<Self> {
        Self::new(
            chain,
            ApprovalContext {
                contract_address,
                spender_address,
                token_address,
                approval_value: Some(approval_value),
                display_expiration: None,
                warning_expiration,
                method: ApprovalMethod::PermitBatch,
                token_field: TokenField::ShowWhenPresent,
            },
        )
    }

    fn new(chain: Chain, context: ApprovalContext) -> Option<Self> {
        let contract_address = ethereum_address_checksum(&context.contract_address).ok()?;
        let spender_address = ethereum_address_checksum(&context.spender_address).ok()?;
        let token_address = context.token_address.map(|value| ethereum_address_checksum(&value)).transpose().ok()?;
        let asset_address = token_address.clone().unwrap_or_else(|| contract_address.clone());
        let token_address = match context.token_field {
            TokenField::AlwaysHide => None,
            TokenField::HideWhenMatchingContract if asset_address == contract_address => None,
            TokenField::HideWhenMatchingContract | TokenField::ShowWhenPresent => token_address,
        };

        Some(Self {
            asset_id: AssetId::from_token(chain, &asset_address),
            contract_address,
            token_address,
            spender_address,
            method: context.method,
            approval_value: context.approval_value,
            display_expiration: context.display_expiration,
            warning_expiration: context.warning_expiration,
        })
    }

    pub(crate) fn simulate(self) -> SimulationResult {
        let warnings = self.warnings();
        self.build_simulation_result(warnings)
    }

    pub(crate) fn build_simulation_result(self, warnings: Vec<SimulationWarning>) -> SimulationResult {
        let mut result = SimulationResult::new(warnings, self.payload());

        if self.method.supports_value_display()
            && let Some(approval_value) = self.approval_value
        {
            result.header = Some(approval_value.to_simulation_header(self.asset_id));
        }

        result
    }

    pub(crate) fn primary_warning(&self) -> SimulationWarning {
        let warning = match self.method {
            ApprovalMethod::Approve => SimulationWarningType::TokenApproval(SimulationWarningApproval {
                asset_id: self.asset_id.clone(),
                value: self.warning_approval_value(),
            }),
            ApprovalMethod::SetApprovalForAll => SimulationWarningType::NftCollectionApproval(self.asset_id.clone()),
            ApprovalMethod::Permit | ApprovalMethod::PermitSingle => SimulationWarningType::PermitApproval(SimulationWarningApproval {
                asset_id: self.asset_id.clone(),
                value: self.warning_approval_value(),
            }),
            ApprovalMethod::PermitBatch => SimulationWarningType::PermitBatchApproval(self.warning_approval_value()),
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

    pub(crate) fn warnings(&self) -> Vec<SimulationWarning> {
        let mut warnings = vec![self.primary_warning()];
        if let Some(warning) = self.expiration_warning() {
            warnings.push(warning);
        }
        warnings
    }

    fn warning_approval_value(&self) -> Option<BigInt> {
        match self.approval_value.as_ref() {
            Some(ApprovalValue::Exact(value)) => Some(BigInt::from(value.clone())),
            Some(ApprovalValue::Unlimited) | None => None,
        }
    }

    pub(crate) fn expiration_warning(&self) -> Option<SimulationWarning> {
        let expiration = self.warning_expiration?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
        if expiration <= now.saturating_add(EXCESSIVE_EXPIRATION_WINDOW.as_secs()) {
            return None;
        }

        Some(SimulationWarning::new(
            SimulationSeverity::Warning,
            SimulationWarningType::ValidationError,
            Some("Excessive expiration".to_string()),
        ))
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

        if let Some(expiration) = self.display_expiration {
            payload.push(SimulationPayloadField::custom(
                "expiration",
                expiration.to_string(),
                SimulationPayloadFieldType::Timestamp,
                SimulationPayloadFieldDisplay::Secondary,
            ));
        }

        payload
    }
}

#[derive(Debug, Clone)]
struct ApprovalContext {
    contract_address: String,
    spender_address: String,
    token_address: Option<String>,
    approval_value: Option<ApprovalValue>,
    display_expiration: Option<u64>,
    warning_expiration: Option<u64>,
    method: ApprovalMethod,
    token_field: TokenField,
}

#[derive(Debug, Clone, Copy)]
enum TokenField {
    AlwaysHide,
    HideWhenMatchingContract,
    ShowWhenPresent,
}
