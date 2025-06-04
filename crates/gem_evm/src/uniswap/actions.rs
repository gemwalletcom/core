use super::contracts::v4::IV4Router;
use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::SolValue;

pub const SWAP_EXACT_IN_SINGLE_ACTION: u8 = 0x06;
pub const SWAP_EXACT_IN_ACTION: u8 = 0x07;
pub const SWAP_EXACT_OUT_SINGLE_ACTION: u8 = 0x08;
pub const SWAP_EXACT_OUT_ACTION: u8 = 0x09;
pub const SETTLE_ACTION: u8 = 0x0b;
pub const SETTLE_ALL_ACTION: u8 = 0x0c;
pub const TAKE_ACTION: u8 = 0x0e;
pub const TAKE_ALL_ACTION: u8 = 0x0f;
pub const TAKE_PORTION_ACTION: u8 = 0x10;

// https://github.com/Uniswap/v4-periphery/blob/main/src/libraries/Actions.sol
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
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

pub fn decode_action_data(data: &[u8]) -> Result<Vec<V4Action>, alloy_sol_types::Error> {
    // The ABI encoding for a sequence of actions is (bytes opcodes, bytes[] action_data)
    let (action_opcodes_bytes, action_data_bytes) = <(Bytes, Vec<Bytes>) as SolValue>::abi_decode_sequence(data)?;

    let action_opcodes: Vec<u8> = action_opcodes_bytes.to_vec();
    let action_data_list: Vec<Vec<u8>> = action_data_bytes.into_iter().map(|b| b.to_vec()).collect();

    if action_opcodes.len() != action_data_list.len() {
        return Err(alloy_sol_types::Error::Other("Mismatched opcodes and data lengths".into()));
    }

    let mut decoded_actions = Vec::with_capacity(action_opcodes.len());

    for (i, opcode) in action_opcodes.iter().enumerate() {
        let action_data = &action_data_list[i];
        let action_data_slice = action_data.as_slice();
        let action = match *opcode {
            SWAP_EXACT_IN_SINGLE_ACTION => V4Action::SWAP_EXACT_IN_SINGLE(<IV4Router::ExactInputSingleParams as SolValue>::abi_decode(action_data_slice)?),
            SWAP_EXACT_IN_ACTION => V4Action::SWAP_EXACT_IN(<IV4Router::ExactInputParams as SolValue>::abi_decode(action_data_slice)?),
            SWAP_EXACT_OUT_SINGLE_ACTION => V4Action::SWAP_EXACT_OUT_SINGLE(<IV4Router::ExactOutputSingleParams as SolValue>::abi_decode(action_data_slice)?),
            SWAP_EXACT_OUT_ACTION => V4Action::SWAP_EXACT_OUT(<IV4Router::ExactOutputParams as SolValue>::abi_decode(action_data_slice)?),
            SETTLE_ACTION => {
                let (currency, amount, payer_is_user) = <(Address, U256, bool) as SolValue>::abi_decode(action_data_slice)?;
                V4Action::SETTLE {
                    currency,
                    amount,
                    payer_is_user,
                }
            }
            SETTLE_ALL_ACTION => {
                let (currency, max_amount) = <(Address, U256) as SolValue>::abi_decode(action_data_slice)?;
                V4Action::SETTLE_ALL { currency, max_amount }
            }
            TAKE_ACTION => {
                let (currency, recipient, amount) = <(Address, Address, U256) as SolValue>::abi_decode(action_data_slice)?;
                V4Action::TAKE { currency, recipient, amount }
            }
            TAKE_ALL_ACTION => {
                let (currency, min_amount) = <(Address, U256) as SolValue>::abi_decode(action_data_slice)?;
                V4Action::TAKE_ALL { currency, min_amount }
            }
            TAKE_PORTION_ACTION => {
                let (currency, recipient, bips) = <(Address, Address, U256) as SolValue>::abi_decode(action_data_slice)?;
                V4Action::TAKE_PORTION { currency, recipient, bips }
            }
            _ => return Err(alloy_sol_types::Error::Other(format!("Unknown action opcode: {}", opcode).into())),
        };
        decoded_actions.push(action);
    }

    Ok(decoded_actions)
}

#[rustfmt::skip]
impl V4Action {
    pub fn byte(&self) -> u8 {
        match self {
            Self::SWAP_EXACT_IN_SINGLE(_) =>    SWAP_EXACT_IN_SINGLE_ACTION,
            Self::SWAP_EXACT_IN(_) =>           SWAP_EXACT_IN_ACTION,
            Self::SWAP_EXACT_OUT_SINGLE(_) =>   SWAP_EXACT_OUT_SINGLE_ACTION,
            Self::SWAP_EXACT_OUT(_) =>          SWAP_EXACT_OUT_ACTION,

            Self::SETTLE { .. }   => SETTLE_ACTION,
            Self::SETTLE_ALL { .. }             => SETTLE_ALL_ACTION,
            Self::TAKE { .. }        => TAKE_ACTION,
            Self::TAKE_ALL { .. }               => TAKE_ALL_ACTION,
            Self::TAKE_PORTION { .. }  => TAKE_PORTION_ACTION,
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

        // Test decode_action_data
        let decoded_actions = decode_action_data(&params).unwrap();

        assert_eq!(actions, decoded_actions, "Decoded actions do not match original actions");
    }
}
