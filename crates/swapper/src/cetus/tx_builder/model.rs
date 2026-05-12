use crate::{
    Quote, SwapperError,
    cetus::model::{FlattenedPath, Path, RouterData},
    fees::{ReferralFee, apply_slippage_in_bp},
};
use sui_types::Address;

pub(super) struct SwapStep<'a> {
    pub(super) path: &'a Path,
    pub(super) coin_a: &'a str,
    pub(super) coin_b: &'a str,
    pub(super) amount_in: u64,
    pub(super) published_at: &'a str,
}

impl<'a> TryFrom<&'a FlattenedPath> for SwapStep<'a> {
    type Error = SwapperError;

    fn try_from(flattened_path: &'a FlattenedPath) -> Result<Self, Self::Error> {
        let path = &flattened_path.path;
        let (coin_a, coin_b) = if path.direction {
            (path.from.as_str(), path.target.as_str())
        } else {
            (path.target.as_str(), path.from.as_str())
        };
        Ok(Self {
            path,
            coin_a,
            coin_b,
            amount_in: flattened_path.amount_in(),
            published_at: path.published_at.as_deref().ok_or(SwapperError::InvalidRoute)?,
        })
    }
}

pub(super) struct SwapLimits {
    pub(super) expected_amount_out: u64,
    pub(super) amount_out_limit: u64,
    pub(super) fee_rate: u32,
    pub(super) fee_recipient: Address,
}

impl SwapLimits {
    pub(super) fn new(quote: &Quote, router: &RouterData, referral_fee: &ReferralFee) -> Result<Self, SwapperError> {
        let expected_amount_out = apply_slippage_in_bp(&router.amount_out, referral_fee.bps);
        let amount_out_limit = apply_slippage_in_bp(&expected_amount_out, quote.data.slippage_bps);
        let fee_rate = referral_fee
            .bps
            .checked_mul(100)
            .ok_or_else(|| SwapperError::ComputeQuoteError("Cetus referral fee overflow".to_string()))?;
        let fee_recipient = if referral_fee.address.is_empty() {
            if fee_rate > 0 {
                return Err(SwapperError::ComputeQuoteError("Cetus referral address is required".to_string()));
            }
            Address::ZERO
        } else {
            referral_fee
                .address
                .parse()
                .map_err(|err| SwapperError::TransactionError(format!("Invalid Sui address {}: {err}", referral_fee.address)))?
        };

        Ok(Self {
            expected_amount_out,
            amount_out_limit,
            fee_rate,
            fee_recipient,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cetus::testkit::{flattened_path, quote, referral_fee, route_path, router};

    #[test]
    fn test_swap_step_and_limits() {
        let fee = referral_fee(50);
        let limits = SwapLimits::new(&quote(100), &router(10000), &fee).unwrap();
        assert_eq!(limits.expected_amount_out, 9950);
        assert_eq!(limits.amount_out_limit, 9850);
        assert_eq!(limits.fee_rate, 5000);
        assert_eq!(limits.fee_recipient, Address::ZERO);

        let base_path = route_path(false, Some("0x1".to_string()));
        let flattened_path_value = flattened_path(base_path.clone(), false);
        let step = SwapStep::try_from(&flattened_path_value).unwrap();
        assert_eq!(step.coin_a, "0xabc::coin::A");
        assert_eq!(step.coin_b, "0x2::sui::SUI");
        assert_eq!(step.amount_in, 123);
        assert_eq!(step.published_at, "0x1");

        let last_intermediate_use = flattened_path(base_path, true);
        let step = SwapStep::try_from(&last_intermediate_use).unwrap();
        assert_eq!(step.amount_in, u64::MAX);

        let missing_published_at_path = flattened_path(route_path(true, None), false);
        let missing_published_at = SwapStep::try_from(&missing_published_at_path);
        assert!(matches!(missing_published_at, Err(SwapperError::InvalidRoute)));
    }
}
