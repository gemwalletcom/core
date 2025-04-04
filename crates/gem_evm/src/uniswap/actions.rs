use super::contracts::v4::IV4Router;
use alloy_primitives::{Address, U256};
use alloy_sol_types::SolValue;

// https://github.com/Uniswap/v4-periphery/blob/main/src/libraries/Actions.sol
#[allow(non_camel_case_types)]
pub enum V4Action {
    SWAP_EXACT_IN_SINGLE(IV4Router::ExactInputSingleParams),
    SWAP_EXACT_IN(IV4Router::ExactInputParams),
    SWAP_EXACT_OUT_SINGLE(IV4Router::ExactOutputSingleParams),
    SWAP_EXACT_OUT(IV4Router::ExactOutputParams),

    SETTLE { currency: Address, amount: U256, payer_is_user: bool },
    SETTLE_ALL { currency: Address, max_amount: U256 },
    TAKE { currency: Address, recipient: Address, amount: U256 },
    TAKE_ALL { currency: Address, min_amount: U256 },
    TAKE_PORTION { currency: Address, recipient: Address, bips: U256 },
}

pub fn encode_actions(actions: &[V4Action]) -> Vec<u8> {
    let encoded_actions = actions.iter().map(|x| x.byte()).collect::<Vec<_>>();
    let encoded_data = actions.iter().map(encode_action_data).collect::<Vec<_>>();
    (encoded_actions, encoded_data).abi_encode_sequence()
}

pub fn encode_action_data(action: &V4Action) -> Vec<u8> {
    match action {
        V4Action::SWAP_EXACT_IN_SINGLE(params) => params.abi_encode(),
        V4Action::SWAP_EXACT_IN(params) => params.abi_encode(),
        V4Action::SWAP_EXACT_OUT_SINGLE(params) => params.abi_encode(),
        V4Action::SWAP_EXACT_OUT(params) => params.abi_encode(),
        V4Action::SETTLE {
            currency,
            amount,
            payer_is_user,
        } => (currency.to_owned(), amount.to_owned(), payer_is_user.to_owned()).abi_encode(),
        V4Action::SETTLE_ALL { currency, max_amount } => (currency.to_owned(), max_amount.to_owned()).abi_encode(),
        V4Action::TAKE { currency, recipient, amount } => (currency.to_owned(), recipient.to_owned(), amount.to_owned()).abi_encode(),
        V4Action::TAKE_ALL { currency, min_amount } => (currency.to_owned(), min_amount.to_owned()).abi_encode(),
        V4Action::TAKE_PORTION { currency, recipient, bips } => (currency.to_owned(), recipient.to_owned(), bips.to_owned()).abi_encode(),
    }
}

#[rustfmt::skip]
impl V4Action {
    pub fn byte(&self) -> u8 {
        match self {
            Self::SWAP_EXACT_IN_SINGLE(_) =>    0x06,
            Self::SWAP_EXACT_IN(_) =>           0x07,
            Self::SWAP_EXACT_OUT_SINGLE(_) =>   0x08,
            Self::SWAP_EXACT_OUT(_) =>          0x09,

            Self::SETTLE { currency: _, amount: _, payer_is_user: _ }   => 0x0b,
            Self::SETTLE_ALL { currency: _, max_amount: _ }             => 0x0c,
            Self::TAKE { currency: _, recipient: _, amount: _, }        => 0x0e,
            Self::TAKE_ALL { currency: _, min_amount: _ }               => 0x0f,
            Self::TAKE_PORTION { currency: _, recipient: _, bips: _, }  => 0x10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uniswap::{command::ADDRESS_THIS, contracts::v4::PathKey};
    use alloy_primitives::{
        aliases::{I24, U24},
        hex::encode as HexEncode,
        Address, Bytes, U256,
    };
    use std::str::FromStr;

    #[test]
    fn test_encode_action() {
        let _1inch_token = Address::from_str("0x111111111117dC0aa78b770fA6A738034120C302").unwrap();

        let actions = vec![
            V4Action::SWAP_EXACT_IN(IV4Router::ExactInputParams {
                currencyIn: Address::ZERO,
                path: vec![PathKey {
                    intermediateCurrency: _1inch_token,
                    fee: U24::from(10000),
                    tickSpacing: I24::from_str("200").unwrap(),
                    hooks: Address::ZERO,
                    hookData: Bytes::new(),
                }],
                amountIn: 2000000000000000_u128,
                amountOutMinimum: 0,
            }),
            V4Action::SETTLE {
                currency: Address::ZERO,
                amount: U256::from(0),
                payer_is_user: true,
            },
            V4Action::TAKE {
                currency: _1inch_token,
                recipient: Address::from_str(ADDRESS_THIS).unwrap(),
                amount: U256::from(0),
            },
        ];

        let encoded_data = actions.iter().map(encode_action_data).collect::<Vec<_>>();

        assert_eq!(HexEncode(&encoded_data[0]), "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000071afd498d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000111111111117dc0aa78b770fa6a738034120c302000000000000000000000000000000000000000000000000000000000000271000000000000000000000000000000000000000000000000000000000000000c8000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(HexEncode(&encoded_data[1]), "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001");
        assert_eq!(HexEncode(&encoded_data[2]), "000000000000000000000000111111111117dc0aa78b770fa6a738034120c30200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000");

        let params = encode_actions(&actions);
        let expected = "000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000003070b0e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000002a000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000071afd498d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000111111111117dc0aa78b770fa6a738034120c302000000000000000000000000000000000000000000000000000000000000271000000000000000000000000000000000000000000000000000000000000000c8000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000111111111117dc0aa78b770fa6a738034120c30200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000";

        assert_eq!(params.len(), expected.len() / 2);
        assert_eq!(HexEncode(&params), expected);
    }
}
