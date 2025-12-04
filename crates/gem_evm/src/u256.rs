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
