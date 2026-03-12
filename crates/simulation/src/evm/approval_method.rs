use gem_evm::eip712::EIP712Message;
use strum::EnumString;

const PERMIT2_DOMAIN_NAME: &str = "Permit2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
pub(crate) enum ApprovalMethod {
    #[strum(serialize = "approve")]
    Approve,
    #[strum(serialize = "setApprovalForAll")]
    SetApprovalForAll,
    #[strum(serialize = "Permit")]
    Permit,
    #[strum(serialize = "PermitSingle")]
    PermitSingle,
    #[strum(serialize = "PermitBatch")]
    PermitBatch,
}

impl ApprovalMethod {
    pub(crate) fn supports_value_display(&self) -> bool {
        match self {
            Self::Approve | Self::Permit | Self::PermitSingle => true,
            Self::SetApprovalForAll | Self::PermitBatch => false,
        }
    }

    pub(crate) fn from_eip712(message: &EIP712Message) -> Option<Self> {
        if let Ok(method) = message.primary_type.parse::<Self>()
            && method.is_eip712_approval_method()
        {
            return Some(method);
        }

        message.domain.name.eq_ignore_ascii_case(PERMIT2_DOMAIN_NAME).then_some(Self::PermitSingle)
    }

    fn is_eip712_approval_method(&self) -> bool {
        match self {
            Self::Permit | Self::PermitSingle | Self::PermitBatch => true,
            Self::Approve | Self::SetApprovalForAll => false,
        }
    }
}

impl std::fmt::Display for ApprovalMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Approve => "Approve",
            Self::SetApprovalForAll => "Set Approval For All",
            Self::Permit => "Permit",
            Self::PermitSingle => "Permit Single",
            Self::PermitBatch => "Permit Batch",
        };

        f.write_str(value)
    }
}
