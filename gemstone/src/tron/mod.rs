pub mod client;
pub mod model;

use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use gem_evm::contracts::erc20::IERC20;

use crate::swapper::SwapperError;

pub fn bs58_to_hex(address: &str) -> Result<Vec<u8>, SwapperError> {
    bs58::decode(address)
        .with_check(None)
        .into_vec()
        .map_err(|e| SwapperError::InvalidAddress(format!("Failed to decode address '{address}': {e}")))
}

pub fn hex_to_utf8(hex: &str) -> Option<String> {
    hex::decode(hex).ok().and_then(|bytes| String::from_utf8(bytes).ok())
}

pub fn encode_parameters(owner: &[u8], spender: &[u8]) -> Vec<u8> {
    let owner_addr = Address::from_slice(&owner[1..]);
    let spender_addr = Address::from_slice(&spender[1..]);
    let parameter = IERC20::allowanceCall {
        owner: owner_addr,
        spender: spender_addr,
    }
    .abi_encode();
    parameter[4..].to_vec() // drop function selector
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tron_encoding() {
        let token_address = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";
        let owner_address = "TA7mCjHFfo68FG3wc6pDCeRGbJSPZkBfL7";
        let gateway_address = "TQjjYNyBmzCyDh5WumFJBhXFyE5PUKqVYZ";

        let token_hex = bs58_to_hex(token_address).unwrap();
        let owner_hex = bs58_to_hex(owner_address).unwrap();
        let gateway_hex = bs58_to_hex(gateway_address).unwrap();

        assert_eq!(hex::encode(&token_hex), "41a614f803b6fd780986a42c78ec9c7f77e6ded13c");
        assert_eq!(hex::encode(&owner_hex), "41019e353a35efaa8e27c2a602a791ae1b19d9c9fa");
        assert_eq!(hex::encode(&gateway_hex), "41a1fd8e8afc126545d76b4a9e905d5be1ccd392e1");

        let parameter = encode_parameters(&owner_hex, &gateway_hex);

        assert_eq!(
            hex::encode(&parameter),
            "000000000000000000000000019e353a35efaa8e27c2a602a791ae1b19d9c9fa000000000000000000000000a1fd8e8afc126545d76b4a9e905d5be1ccd392e1"
        );
    }
}
