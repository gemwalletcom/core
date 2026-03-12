use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::AssetId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SimulationSeverity {
    Low,
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SimulationWarningType {
    TokenApproval { asset_id: AssetId, value: Option<BigInt> },
    SuspiciousSpender,
    ExternallyOwnedSpender,
    NftCollectionApproval { asset_id: AssetId },
    PermitApproval { asset_id: AssetId, value: Option<BigInt> },
    PermitBatchApproval { value: Option<BigInt> },
    ValidationError,
}

impl SimulationWarningType {
    fn requires_spender_verification(&self) -> bool {
        match self {
            Self::SuspiciousSpender | Self::ExternallyOwnedSpender | Self::ValidationError => false,
            Self::TokenApproval { .. } | Self::NftCollectionApproval { .. } | Self::PermitApproval { .. } | Self::PermitBatchApproval { .. } => true,
        }
    }

    fn approval_value(&self) -> Option<&Option<BigInt>> {
        match self {
            Self::TokenApproval { value, .. } | Self::PermitApproval { value, .. } | Self::PermitBatchApproval { value } => Some(value),
            Self::SuspiciousSpender | Self::ExternallyOwnedSpender | Self::NftCollectionApproval { .. } | Self::ValidationError => None,
        }
    }

    fn collapse_priority(&self, severity: SimulationSeverity) -> u8 {
        match self {
            Self::ExternallyOwnedSpender => 2,
            Self::ValidationError if severity == SimulationSeverity::Critical => 2,
            _ if self.approval_value().is_some_and(Option::is_none) => 1,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SimulationWarning {
    pub severity: SimulationSeverity,
    pub warning: SimulationWarningType,
    pub message: Option<String>,
}

impl SimulationWarning {
    pub fn new(severity: SimulationSeverity, warning: SimulationWarningType, message: Option<String>) -> Self {
        Self { severity, warning, message }
    }

    fn collapse_priority(&self) -> u8 {
        self.warning.collapse_priority(self.severity)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SimulationBalanceChange {
    pub asset_id: AssetId,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SimulationPayloadFieldType {
    Text,
    Address,
    Timestamp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SimulationPayloadFieldDisplay {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SimulationPayloadFieldKind {
    Contract,
    Method,
    Token,
    Spender,
    Value,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SimulationPayloadField {
    pub kind: SimulationPayloadFieldKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub value: String,
    pub field_type: SimulationPayloadFieldType,
    pub display: SimulationPayloadFieldDisplay,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SimulationHeader {
    pub asset_id: AssetId,
    pub value: String,
}

impl SimulationPayloadField {
    pub fn standard(kind: SimulationPayloadFieldKind, value: impl Into<String>, field_type: SimulationPayloadFieldType, display: SimulationPayloadFieldDisplay) -> Self {
        debug_assert!(kind != SimulationPayloadFieldKind::Custom);
        Self {
            kind,
            label: None,
            value: value.into(),
            field_type,
            display,
        }
    }

    pub fn custom(label: impl Into<String>, value: impl Into<String>, field_type: SimulationPayloadFieldType, display: SimulationPayloadFieldDisplay) -> Self {
        let label = label.into();
        debug_assert!(!label.is_empty());
        Self {
            kind: SimulationPayloadFieldKind::Custom,
            label: Some(label),
            value: value.into(),
            field_type,
            display,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SimulationResult {
    pub warnings: Vec<SimulationWarning>,
    pub balance_changes: Vec<SimulationBalanceChange>,
    pub payload: Vec<SimulationPayloadField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<SimulationHeader>,
}

impl Default for SimulationResult {
    fn default() -> Self {
        Self::new(vec![], vec![])
    }
}

impl SimulationResult {
    pub fn new(warnings: Vec<SimulationWarning>, payload: Vec<SimulationPayloadField>) -> Self {
        Self {
            warnings: Self::collapse_warnings(warnings),
            balance_changes: vec![],
            payload: promote_single_secondary_payload_field(payload),
            header: None,
        }
    }

    pub fn prepend_warnings(mut self, warnings: Vec<SimulationWarning>) -> Self {
        self.warnings = Self::collapse_warnings(warnings.into_iter().chain(self.warnings).collect());
        self
    }

    pub fn requires_spender_verification(&self) -> bool {
        self.warnings.iter().any(|warning| warning.warning.requires_spender_verification())
    }

    fn collapse_warnings(warnings: Vec<SimulationWarning>) -> Vec<SimulationWarning> {
        if let Some(warning) = warnings.iter().find(|warning| warning.collapse_priority() == 2).cloned() {
            return vec![warning];
        }

        if let Some(warning) = warnings.iter().find(|warning| warning.collapse_priority() == 1).cloned() {
            return vec![warning];
        }

        warnings
    }
}

pub fn promote_single_secondary_payload_field(payload: Vec<SimulationPayloadField>) -> Vec<SimulationPayloadField> {
    let secondary_count = payload.iter().filter(|field| field.display == SimulationPayloadFieldDisplay::Secondary).count();

    if secondary_count != 1 {
        return payload;
    }

    payload
        .into_iter()
        .map(|field| {
            if field.display == SimulationPayloadFieldDisplay::Secondary {
                SimulationPayloadField {
                    display: SimulationPayloadFieldDisplay::Primary,
                    ..field
                }
            } else {
                field
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;

    use super::{
        SimulationPayloadField, SimulationPayloadFieldDisplay, SimulationPayloadFieldKind, SimulationPayloadFieldType, SimulationResult, SimulationSeverity, SimulationWarning,
        SimulationWarningType, promote_single_secondary_payload_field,
    };

    #[test]
    fn simulation_result_keeps_only_blocking_warning() {
        let result = SimulationResult::new(
            vec![
                SimulationWarning::new(
                    SimulationSeverity::Warning,
                    SimulationWarningType::PermitApproval {
                        asset_id: "ethereum_0x123".into(),
                        value: Some(BigInt::from(100)),
                    },
                    None,
                ),
                SimulationWarning::new(SimulationSeverity::Critical, SimulationWarningType::ExternallyOwnedSpender, None),
            ],
            Vec::<SimulationPayloadField>::new(),
        );

        assert_eq!(
            result.warnings,
            vec![SimulationWarning::new(SimulationSeverity::Critical, SimulationWarningType::ExternallyOwnedSpender, None,)]
        );
    }

    #[test]
    fn approval_simulation_requires_spender_verification() {
        let result = SimulationResult::new(
            vec![SimulationWarning::new(
                SimulationSeverity::Warning,
                SimulationWarningType::PermitApproval {
                    asset_id: "ethereum_0x123".into(),
                    value: Some(BigInt::from(100)),
                },
                None,
            )],
            vec![],
        );

        assert!(result.requires_spender_verification());
    }

    #[test]
    fn validation_warning_suppresses_secondary_warnings() {
        let result = SimulationResult::new(
            vec![
                SimulationWarning::new(
                    SimulationSeverity::Warning,
                    SimulationWarningType::PermitApproval {
                        asset_id: "ethereum_0x123".into(),
                        value: Some(BigInt::from(100)),
                    },
                    None,
                ),
                SimulationWarning::new(
                    SimulationSeverity::Critical,
                    SimulationWarningType::ValidationError,
                    Some("Unable to verify spender is a contract".to_string()),
                ),
            ],
            Vec::<SimulationPayloadField>::new(),
        );

        assert_eq!(
            result.warnings,
            vec![SimulationWarning::new(
                SimulationSeverity::Critical,
                SimulationWarningType::ValidationError,
                Some("Unable to verify spender is a contract".to_string()),
            )]
        );
    }

    #[test]
    fn unlimited_warning_wins_when_present() {
        let result = SimulationResult::new(
            vec![SimulationWarning::new(
                SimulationSeverity::Warning,
                SimulationWarningType::PermitApproval {
                    asset_id: "ethereum_0x123".into(),
                    value: None,
                },
                None,
            )],
            Vec::<SimulationPayloadField>::new(),
        );

        assert_eq!(
            result.warnings,
            vec![SimulationWarning::new(
                SimulationSeverity::Warning,
                SimulationWarningType::PermitApproval {
                    asset_id: "ethereum_0x123".into(),
                    value: None,
                },
                None,
            )]
        );
    }

    #[test]
    fn unlimited_secondary_warning_suppresses_redundant_token_approval_warning() {
        let result = SimulationResult::new(
            vec![
                SimulationWarning::new(
                    SimulationSeverity::Warning,
                    SimulationWarningType::TokenApproval {
                        asset_id: "ethereum_0x123".into(),
                        value: Some(BigInt::from(1000)),
                    },
                    None,
                ),
                SimulationWarning::new(
                    SimulationSeverity::Warning,
                    SimulationWarningType::TokenApproval {
                        asset_id: "ethereum_0x123".into(),
                        value: None,
                    },
                    None,
                ),
            ],
            Vec::<SimulationPayloadField>::new(),
        );

        assert_eq!(
            result.warnings,
            vec![SimulationWarning::new(
                SimulationSeverity::Warning,
                SimulationWarningType::TokenApproval {
                    asset_id: "ethereum_0x123".into(),
                    value: None,
                },
                None,
            )]
        );
    }

    #[test]
    fn single_secondary_payload_field_is_promoted_to_primary() {
        let payload = promote_single_secondary_payload_field(vec![
            SimulationPayloadField::standard(
                SimulationPayloadFieldKind::Contract,
                "0x123",
                SimulationPayloadFieldType::Address,
                SimulationPayloadFieldDisplay::Primary,
            ),
            SimulationPayloadField::standard(
                SimulationPayloadFieldKind::Value,
                "Unlimited",
                SimulationPayloadFieldType::Text,
                SimulationPayloadFieldDisplay::Secondary,
            ),
        ]);

        assert_eq!(payload.len(), 2);
        assert!(payload.iter().all(|field| field.display == SimulationPayloadFieldDisplay::Primary));
    }

    #[test]
    fn multiple_secondary_payload_fields_stay_secondary() {
        let payload = promote_single_secondary_payload_field(vec![
            SimulationPayloadField::standard(
                SimulationPayloadFieldKind::Contract,
                "0x123",
                SimulationPayloadFieldType::Address,
                SimulationPayloadFieldDisplay::Primary,
            ),
            SimulationPayloadField::standard(
                SimulationPayloadFieldKind::Value,
                "Unlimited",
                SimulationPayloadFieldType::Text,
                SimulationPayloadFieldDisplay::Secondary,
            ),
            SimulationPayloadField::custom("expiration", "123", SimulationPayloadFieldType::Timestamp, SimulationPayloadFieldDisplay::Secondary),
        ]);

        assert_eq!(payload[1].display, SimulationPayloadFieldDisplay::Secondary);
        assert_eq!(payload[2].display, SimulationPayloadFieldDisplay::Secondary);
    }
}
