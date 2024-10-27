use super::contract::IUniversalRouter;
use alloy_core::primitives::{Address, Bytes, U160, U256};
use alloy_sol_types::{sol_data, SolCall, SolType};

pub const MSG_SENDER: &str = "0x0000000000000000000000000000000000000001";
pub const ADDRESS_THIS: &str = "0x0000000000000000000000000000000000000002";

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum UniversalRouterCommand {
    V3_SWAP_EXACT_IN(V3SwapExactIn),
    V3_SWAP_EXACT_OUT(V3SwapExactOut),
    PERMIT2_TRANSFER_FROM,
    PERMIT2_PERMIT_BATCH,
    SWEEP(Sweep),
    TRANSFER(Transfer),
    PAY_PORTION(PayPortion),
    V2_SWAP_EXACT_IN,
    V2_SWAP_EXACT_OUT,
    PERMIT2_PERMIT,
    WRAP_ETH(WrapEth),
    UNWRAP_WETH(UnwrapWeth),
    PERMIT2_TRANSFER_FROM_BATCH,
}

type V3SwapExactType = (sol_data::Address, sol_data::Uint<256>, sol_data::Uint<256>, sol_data::Bytes, sol_data::Bool);

#[derive(Debug, PartialEq)]
pub struct V3SwapExactIn {
    pub recipient: Address,
    pub amount_in: U256,
    pub amount_out_min: U256,
    pub path: Bytes,
    pub payer_is_user: bool,
}

impl V3SwapExactIn {
    pub fn abi_encode(&self) -> Vec<u8> {
        let data = (self.recipient, self.amount_in, self.amount_out_min, self.path.clone(), self.payer_is_user);
        V3SwapExactType::abi_encode_sequence(&data)
    }
}

#[derive(Debug, PartialEq)]
pub struct V3SwapExactOut {
    pub recipient: Address,
    pub amount_out: U256,
    pub amount_in_max: U256,
    pub path: Bytes,
    pub payer_is_user: bool,
}

impl V3SwapExactOut {
    pub fn abi_encode(&self) -> Vec<u8> {
        let data = (self.recipient, self.amount_out, self.amount_in_max, self.path.clone(), self.payer_is_user);
        V3SwapExactType::abi_encode_sequence(&data)
    }
}

type SweepType = (sol_data::Address, sol_data::Address, sol_data::Uint<160>);
type PayPortionType = (sol_data::Address, sol_data::Address, sol_data::Uint<256>);
type WrapEthType = (sol_data::Address, sol_data::Uint<256>);
type UnwrapWethType = WrapEthType;

#[derive(Debug, PartialEq)]
pub struct Sweep {
    pub token: Address,
    pub recipient: Address,
    pub amount_min: U160,
}

impl Sweep {
    pub fn abi_encode(&self) -> Vec<u8> {
        let data = (self.token, self.recipient, self.amount_min);
        SweepType::abi_encode_sequence(&data)
    }
}

#[derive(Debug, PartialEq)]
pub struct Transfer {
    pub token: Address,
    pub recipient: Address,
    pub value: U256,
}

#[derive(Debug, PartialEq)]
pub struct PayPortion {
    pub token: Address,
    pub recipient: Address,
    pub bips: U256,
}

impl PayPortion {
    pub fn abi_encode(&self) -> Vec<u8> {
        let data = (self.token, self.recipient, self.bips);
        PayPortionType::abi_encode_sequence(&data)
    }
}

#[derive(Debug, PartialEq)]
pub struct WrapEth {
    pub recipient: Address,
    pub amount_min: U256,
}

impl WrapEth {
    pub fn abi_encode(&self) -> Vec<u8> {
        let data = (self.recipient, self.amount_min);
        WrapEthType::abi_encode_sequence(&data)
    }
}

#[derive(Debug, PartialEq)]
pub struct UnwrapWeth {
    pub recipient: Address,
    pub amount_min: U256,
}

impl UnwrapWeth {
    pub fn abi_encode(&self) -> Vec<u8> {
        let data = (self.recipient, self.amount_min);
        UnwrapWethType::abi_encode_sequence(&data)
    }
}

impl UniversalRouterCommand {
    pub fn raw_value(&self) -> u8 {
        match self {
            Self::V3_SWAP_EXACT_IN(_) => 0x00,
            Self::V3_SWAP_EXACT_OUT(_) => 0x01,
            Self::PERMIT2_TRANSFER_FROM => 0x02,
            Self::PERMIT2_PERMIT_BATCH => 0x03,
            Self::SWEEP(_) => 0x04,
            Self::TRANSFER(_) => 0x05,
            Self::PAY_PORTION(_) => 0x06,
            Self::V2_SWAP_EXACT_IN => 0x08,
            Self::V2_SWAP_EXACT_OUT => 0x09,
            Self::PERMIT2_PERMIT => 0x0a,
            Self::WRAP_ETH(_) => 0x0b,
            Self::UNWRAP_WETH(_) => 0x0c,
            Self::PERMIT2_TRANSFER_FROM_BATCH => 0x0d,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::V3_SWAP_EXACT_IN(command) => command.abi_encode(),
            Self::SWEEP(command) => command.abi_encode(),
            Self::PAY_PORTION(command) => command.abi_encode(),
            Self::WRAP_ETH(command) => command.abi_encode(),
            _ => {
                todo!()
            }
        }
    }
}

pub fn encode_commands(commands: &[UniversalRouterCommand], deadline: U256) -> Vec<u8> {
    let commands_bytes: Vec<u8> = commands.iter().map(|command| command.raw_value()).collect();
    let inputs: Vec<Bytes> = commands.iter().map(|command| Bytes::from_iter(command.encode().iter())).collect();
    let call = IUniversalRouter::executeCall {
        commands: Bytes::from_iter(commands_bytes.iter()),
        inputs,
        deadline,
    };
    call.abi_encode()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_core::primitives::{
        hex::{decode as HexDecode, encode_prefixed as HexEncode},
        Address, Bytes, U160, U256,
    };
    use std::str::FromStr;

    #[test]
    fn test_encode_commands() {
        // Replicate https://optimistic.etherscan.io/tx/0xcc56d922ad307e9ffff9935f7f28f8cdb7de7e1d0e83d3c6f8520c5eeed69e41
        let amount_in = U256::from(1000000000000000u64);
        // WETH / USDC 0.05% pool (5 bps)
        let path = HexDecode("0x42000000000000000000000000000000000000060001f40b2c639c533813f4aa9d7837caf62653d097ff85").unwrap();
        let fee_receiver = Address::from_str("0x7ffc3dbf3b2b50ff3a1d5523bc24bb5043837b14").unwrap();
        let token_usdc = Address::from_str("0x0b2c639c533813f4aa9d7837caf62653d097ff85").unwrap();
        let amount_min = 2597593; // quote amount x (1 - slippage 0.5% - ref fee 0.25%)
        let ref_fee_bp = 25; // 0.25%

        let commands: Vec<UniversalRouterCommand> = vec![
            UniversalRouterCommand::WRAP_ETH(WrapEth {
                recipient: Address::from_str(ADDRESS_THIS).unwrap(),
                amount_min: amount_in,
            }),
            UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                recipient: Address::from_str(ADDRESS_THIS).unwrap(),
                amount_in,
                amount_out_min: U256::from(0),
                path: Bytes::from(path),
                payer_is_user: false,
            }),
            UniversalRouterCommand::PAY_PORTION(PayPortion {
                token: token_usdc,
                recipient: fee_receiver,
                bips: U256::from(ref_fee_bp),
            }),
            UniversalRouterCommand::SWEEP(Sweep {
                token: token_usdc,
                recipient: Address::from_str(MSG_SENDER).unwrap(),
                amount_min: U160::from(amount_min),
            }),
        ];

        let deadline = U256::from(1729227095);
        let expected = "0x3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006711e95700000000000000000000000000000000000000000000000000000000000000040b000604000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002800000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000038d7ea4c680000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000038d7ea4c68000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002b42000000000000000000000000000000000000060001f40b2c639c533813f4aa9d7837caf62653d097ff8500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000b2c639c533813f4aa9d7837caf62653d097ff850000000000000000000000007ffc3dbf3b2b50ff3a1d5523bc24bb5043837b14000000000000000000000000000000000000000000000000000000000000001900000000000000000000000000000000000000000000000000000000000000600000000000000000000000000b2c639c533813f4aa9d7837caf62653d097ff850000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000027a2d9";
        let encoded = encode_commands(&commands, deadline);

        assert_eq!(HexEncode(encoded), expected);
    }
}
