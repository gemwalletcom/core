// https://github.com/across-protocol/sdk/blob/master/src/lpFeeCalculator/lpFeeCalculator.ts#L10
use crate::ether_conv::EtherConv;
use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};
use std::cmp::max;

#[derive(Debug, Clone)]
pub struct RateModel {
    pub ubar: BigInt,
    pub r0: BigInt,
    pub r1: BigInt,
    pub r2: BigInt,
}

/// Converts an APY rate to a one-week rate.
/// R_week = (1 + apy)^(1/52) - 1
pub fn convert_apy_to_weekly_fee(apy: BigInt) -> BigInt {
    let fixed_point_adjustment = 10u64.pow(18) as f64;

    // Perform decimal calculations using floating-point for fractional exponents
    let apy_decimal = apy.to_f64().unwrap() / fixed_point_adjustment;
    let weekly_fee_pct = ((1.0 + apy_decimal).powf(1.0 / 52.0) - 1.0) * fixed_point_adjustment;

    BigInt::from(weekly_fee_pct.ceil() as u64)
}

/// Truncate a BigUint to a given number of decimal places (from 18).
pub fn truncate_18_decimal_bn(input: &BigInt, digits: u32) -> BigInt {
    let digits_to_drop = 18 - digits;
    let multiplier = BigInt::from(10).pow(digits_to_drop);
    (input / &multiplier) * multiplier
}

pub struct LpFeeCalculator {
    pub rate_model: RateModel,
}

impl LpFeeCalculator {
    pub fn new(rate_model: RateModel) -> Self {
        //! Rate model to be used in this calculation.
        Self { rate_model }
    }

    /// Calculate the instantaneous rate for a 0 sized deposit (infinitesimally small).
    ///
    /// # Parameters
    /// - util: the utilization rate of the pool
    ///
    /// # Returns
    /// The instantaneous rate for a 0 sized deposit.
    pub fn instantaneous_rate(&self, util: &BigInt) -> BigInt {
        let model = &self.rate_model;
        let one = EtherConv::one();
        let (ubar, r1, r2) = (model.ubar.clone(), model.r1.clone(), model.r2.clone());

        let before_kink = if model.ubar.is_zero() {
            BigInt::zero()
        } else {
            util.min(&model.ubar) * r1 / &ubar
        };

        let after_kink = max(util - &ubar, BigInt::zero()) * r2 / (one - &ubar);
        model.r0.clone() + before_kink + after_kink
    }

    /// Compute area under curve of the piece-wise linear rate model
    ///
    /// # Parameters
    /// - util: the utilization rate of the pool
    ///
    /// # Returns
    /// The area under the curve of the piece-wise linear rate model.the area under the curve
    pub fn area_under_curve(&self, util: &BigInt) -> BigInt {
        let model = &self.rate_model;
        let fixed_point_adjustment = EtherConv::one();
        let point_5 = EtherConv::one() / 2;

        let util_before_kink = util.min(&model.ubar);
        let rect_1 = util_before_kink * &model.r0 / &fixed_point_adjustment;
        let triangle_1 =
            &point_5 * (self.instantaneous_rate(util_before_kink) - &model.r0) * util_before_kink / &fixed_point_adjustment / &fixed_point_adjustment;

        let util_after = max(util - &model.ubar, BigInt::zero());
        let rect_2 = util_after.clone() * (model.r0.clone() + model.r1.clone()) / &fixed_point_adjustment;
        let triangle_2 =
            point_5 * (self.instantaneous_rate(util) - (model.r0.clone() + model.r1.clone())) * util_after / &fixed_point_adjustment / &fixed_point_adjustment;

        rect_1 + triangle_1 + rect_2 + triangle_2
    }

    /// Calculate the realized yearly LP Fee APY Percent for a given rate model, utilization before and after the deposit.
    ///
    /// # Parameters
    /// - util_before: the utilization rate of the pool before the deposit
    /// - util_after: the utilization rate of the pool after the deposit
    ///
    /// # Returns
    /// The realized LP fee APY percent.
    pub fn apy_from_utilization(&self, util_before: &BigInt, util_after: &BigInt) -> BigInt {
        if util_before == util_after {
            return self.instantaneous_rate(util_before);
        }

        let one = EtherConv::one();
        let area_before = self.area_under_curve(util_before);
        let area_after = self.area_under_curve(util_after);

        (area_after - area_before) * one / (util_after - util_before)
    }

    /// Calculate the realized LP Fee Percent for a given rate model, utilization before and after the deposit.
    ///
    /// # Parameters
    /// - util_before: The utilization of the pool before the deposit.
    /// - util_after: The utilization of the pool after the deposit.
    /// - truncate_decimals: Whether to truncate the result to 6 decimals.
    ///
    /// # Returns
    /// The realized LP fee percent.
    pub fn realized_lp_fee_pct(&self, util_before: &BigInt, util_after: &BigInt, truncate_decimals: bool) -> BigInt {
        let apy = self.apy_from_utilization(util_before, util_after);
        let weekly_fee = convert_apy_to_weekly_fee(apy);

        if truncate_decimals {
            truncate_18_decimal_bn(&weekly_fee, 6)
        } else {
            weekly_fee
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apy_from_utilization() {
        let eth_model = RateModel {
            ubar: EtherConv::parse_ether("0.65"),
            r0: BigInt::zero(),
            r1: EtherConv::parse_ether("0.08"),
            r2: EtherConv::parse_ether("1"),
        };
        let calculator = LpFeeCalculator::new(eth_model);
        let util_before = BigInt::from(0);
        let util_after = EtherConv::parse_ether("0.01");

        let apy = calculator.apy_from_utilization(&util_before, &util_after);
        let apy_fee_pct = calculator.realized_lp_fee_pct(&util_before, &util_after, false);

        assert_eq!(apy, BigInt::from(615384615384600u64));
        assert_eq!(apy_fee_pct, BigInt::from(11830749673481u64));
    }
}
