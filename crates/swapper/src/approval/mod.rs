pub mod evm;

pub use evm::*;
use primitives::swap::ApprovalData;

/// Returns the swap transaction gas limit only when a separate approval transaction is required.
pub fn get_swap_gas_limit_with_approval(approval: &Option<ApprovalData>, swap_gas_limit: Option<String>, default_swap_gas_limit: u64) -> Option<String> {
    approval.as_ref().map(|_| swap_gas_limit.unwrap_or_else(|| default_swap_gas_limit.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_swap_gas_limit_with_approval() {
        let approval = Some(ApprovalData::make("0xtoken", "0xspender", "1000", true));

        assert_eq!(get_swap_gas_limit_with_approval(&approval, Some("250000".to_string()), 750_000), Some("250000".to_string()));
        assert_eq!(get_swap_gas_limit_with_approval(&approval, None, 750_000), Some("750000".to_string()));
        assert_eq!(get_swap_gas_limit_with_approval(&None, Some("250000".to_string()), 750_000), None);
    }
}
