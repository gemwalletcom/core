use alloy_primitives::U256;
use num_bigint::BigUint;

pub fn u256_to_biguint(value: &U256) -> BigUint {
    BigUint::from_bytes_be(&value.to_be_bytes::<32>())
}

pub fn biguint_to_u256(value: &BigUint) -> Option<U256> {
    let bytes = value.to_bytes_be();
    if bytes.len() > 32 {
        return None;
    }

    Some(U256::from_be_slice(&bytes))
}

pub fn u256_to_f64(value: U256) -> f64 {
    let limbs = value.as_limbs();
    let low = limbs[0] as f64;
    let mid_low = limbs[1] as f64 * 2f64.powi(64);
    let mid_high = limbs[2] as f64 * 2f64.powi(128);
    let high = limbs[3] as f64 * 2f64.powi(192);
    low + mid_low + mid_high + high
}
