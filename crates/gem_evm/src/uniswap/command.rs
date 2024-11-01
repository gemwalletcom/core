use super::contract::IUniversalRouter;
use crate::permit2::IAllowanceTransfer;
use alloy_core::primitives::{Address, Bytes, U256};
use alloy_sol_types::{sol_data, SolCall, SolType};

pub const MSG_SENDER: &str = "0x0000000000000000000000000000000000000001";
pub const ADDRESS_THIS: &str = "0x0000000000000000000000000000000000000002";

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
    PERMIT2_PERMIT(Permit2Permit),
    WRAP_ETH(WrapEth),
    UNWRAP_WETH(UnwrapWeth),
    PERMIT2_TRANSFER_FROM_BATCH,
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
            Self::PERMIT2_PERMIT(_) => 0x0a,
            Self::WRAP_ETH(_) => 0x0b,
            Self::UNWRAP_WETH(_) => 0x0c,
            Self::PERMIT2_TRANSFER_FROM_BATCH => 0x0d,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::V3_SWAP_EXACT_IN(payload) => payload.abi_encode(),
            Self::V3_SWAP_EXACT_OUT(payload) => payload.abi_encode(),
            Self::SWEEP(payload) => payload.abi_encode(),
            Self::TRANSFER(payload) => payload.abi_encode(),
            Self::PAY_PORTION(payload) => payload.abi_encode(),
            Self::WRAP_ETH(payload) => payload.abi_encode(),
            Self::UNWRAP_WETH(payload) => payload.abi_encode(),
            Self::PERMIT2_PERMIT(payload) => payload.abi_encode(),
            _ => {
                todo!()
            }
        }
    }
}

type V3SwapExactType = (sol_data::Address, sol_data::Uint<256>, sol_data::Uint<256>, sol_data::Bytes, sol_data::Bool);
type SweepType = (sol_data::Address, sol_data::Address, sol_data::Uint<256>);
type PayPortionType = (sol_data::Address, sol_data::Address, sol_data::Uint<256>);
type TransferType = PayPortionType;
type WrapEthType = (sol_data::Address, sol_data::Uint<256>);
type UnwrapWethType = WrapEthType;
type Permit2PermitType = (IAllowanceTransfer::PermitSingle, sol_data::Bytes);

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

#[derive(Debug, PartialEq)]
pub struct Sweep {
    pub token: Address,
    pub recipient: Address,
    pub amount_min: U256,
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

impl Transfer {
    pub fn abi_encode(&self) -> Vec<u8> {
        let data = (self.token, self.recipient, self.value);
        TransferType::abi_encode_sequence(&data)
    }
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

pub struct Permit2Permit {
    pub permit_single: IAllowanceTransfer::PermitSingle,
    pub signature: Bytes,
}

impl Permit2Permit {
    pub fn abi_encode(&self) -> Vec<u8> {
        let data = (self.permit_single.clone(), self.signature.clone());
        Permit2PermitType::abi_encode_sequence(&data)
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
    use alloy_primitives::aliases::U48;
    use std::str::FromStr;

    const OP_USDC: &str = "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85";
    const OP_AAVE: &str = "0x76fb31fb4af56892a25e32cfc43de717950c9278";
    const OP_WETH: &str = "0x4200000000000000000000000000000000000006";
    const OP_ROUTER: &str = "0xCb1355ff08Ab38bBCE60111F1bb2B784bE25D7e8";

    #[test]
    fn test_encode_eth_to_usdc() {
        // Replicate https://optimistic.etherscan.io/tx/0xcc56d922ad307e9ffff9935f7f28f8cdb7de7e1d0e83d3c6f8520c5eeed69e41
        let amount_in = U256::from(1000000000000000u64);
        // WETH / USDC 0.05% pool (5 bps)
        let path = HexDecode("0x42000000000000000000000000000000000000060001f40b2c639c533813f4aa9d7837caf62653d097ff85").unwrap();
        let fee_receiver = Address::from_str("0x7ffc3dbf3b2b50ff3a1d5523bc24bb5043837b14").unwrap();
        let token_usdc = Address::from_str(OP_USDC).unwrap();
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
                amount_min: U256::from(amount_min),
            }),
        ];

        let deadline = U256::from(1729227095);
        let expected = "0x3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006711e95700000000000000000000000000000000000000000000000000000000000000040b000604000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002800000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000038d7ea4c680000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000038d7ea4c68000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002b42000000000000000000000000000000000000060001f40b2c639c533813f4aa9d7837caf62653d097ff8500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000b2c639c533813f4aa9d7837caf62653d097ff850000000000000000000000007ffc3dbf3b2b50ff3a1d5523bc24bb5043837b14000000000000000000000000000000000000000000000000000000000000001900000000000000000000000000000000000000000000000000000000000000600000000000000000000000000b2c639c533813f4aa9d7837caf62653d097ff850000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000027a2d9";
        let encoded = encode_commands(&commands, deadline);

        assert_eq!(HexEncode(encoded), expected);
    }

    #[test]
    fn test_encode_usdc_to_usdt() {
        // https://optimistic.etherscan.io/tx/0xe1d0cc4e6c25c836166dd50daa32c670cc690e4fd7538fe8a709bfda5ce26db8
        // 0.01% pool
        let token_usdc = Address::from_str(OP_USDC).unwrap();
        let path = Bytes::from(hex::decode("0b2c639c533813f4aa9d7837caf62653d097ff8500006494b008aa00579c1307b0ef2c499ad98a8ce58e58").unwrap());
        let router = Address::from_str(OP_ROUTER).unwrap();
        let commands = vec![
            UniversalRouterCommand::PERMIT2_PERMIT(Permit2Permit {
                permit_single: IAllowanceTransfer::PermitSingle {
                    details: IAllowanceTransfer::PermitDetails {
                        token: token_usdc,
                        amount: U160::from_str("1461501637330902918203684832716283019655932542975").unwrap(),
                        expiration: U48::from(1732667593),
                        nonce: U48::from(0),
                    },
                    spender: router,
                    sigDeadline: U256::from(1730077393),
                },
                signature: Bytes::from(
                    hex::decode(
                        "8f32d2e66506a4f424b1b23309ed75d338534d0912129a8aa3381fab4eb8032f160e0988f10f512b19a58c2a689416366c61cc0c483c3b5322dc91f8b60107671b",
                    )
                    .unwrap(),
                ),
            }),
            UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                recipient: Address::from_str("0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7").unwrap(),
                amount_in: U256::from(6500000),
                amount_out_min: U256::from(6443500),
                path,
                payer_is_user: true,
            }),
        ];
        let deadline = U256::from(1730075326139u64);
        let encoded = encode_commands(&commands, deadline);
        // drop last 0c byte
        let expected = "0x3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000192d08676bb00000000000000000000000000000000000000000000000000000000000000020a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000b2c639c533813f4aa9d7837caf62653d097ff85000000000000000000000000ffffffffffffffffffffffffffffffffffffffff00000000000000000000000000000000000000000000000000000000674668c90000000000000000000000000000000000000000000000000000000000000000000000000000000000000000cb1355ff08ab38bbce60111f1bb2b784be25d7e800000000000000000000000000000000000000000000000000000000671ee2d100000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000418f32d2e66506a4f424b1b23309ed75d338534d0912129a8aa3381fab4eb8032f160e0988f10f512b19a58c2a689416366c61cc0c483c3b5322dc91f8b60107671b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000514bcb1f9aabb904e6106bd1052b66d2706dbbb70000000000000000000000000000000000000000000000000000000000632ea000000000000000000000000000000000000000000000000000000000006251ec00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002b0b2c639c533813f4aa9d7837caf62653d097ff8500006494b008aa00579c1307b0ef2c499ad98a8ce58e58000000000000000000000000000000000000000000";

        assert_eq!(HexEncode(encoded), expected);
    }

    #[test]
    fn test_encode_usdc_to_aave() {
        // https://optimistic.etherscan.io/tx/0x68ecc3014bf65dbfdd135bf1922165732bd9d5b95de797dd818d36aec279d3c8
        let token_aave = Address::from_str(OP_AAVE).unwrap();
        let commands: Vec<UniversalRouterCommand> = vec![
            UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                recipient: Address::from_str(ADDRESS_THIS).unwrap(),
                amount_in: U256::from(5064985),
                amount_out_min: U256::from(0),
                path: Bytes::from(HexDecode("0b2c639c533813f4aa9d7837caf62653d097ff85000bb876fb31fb4af56892a25e32cfc43de717950c9278").unwrap()),
                payer_is_user: true,
            }),
            UniversalRouterCommand::PAY_PORTION(PayPortion {
                token: token_aave,
                recipient: Address::from_str("0x3d83ec320541ae96c4c91e9202643870458fb290").unwrap(),
                bips: U256::from(25),
            }),
            UniversalRouterCommand::SWEEP(Sweep {
                token: token_aave,
                recipient: Address::from_str("0x514bcb1f9aabb904e6106bd1052b66d2706dbbb7").unwrap(),
                amount_min: U256::from(32964572478499319u64),
            }),
        ];

        let deadline = U256::from(1730115968256u64);
        let encoded = encode_commands(&commands, deadline);
        // drop last 0c byte
        let expected = "0x3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000192d2f29d000000000000000000000000000000000000000000000000000000000000000003000604000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000004d4919000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002b0b2c639c533813f4aa9d7837caf62653d097ff85000bb876fb31fb4af56892a25e32cfc43de717950c9278000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000076fb31fb4af56892a25e32cfc43de717950c92780000000000000000000000003d83ec320541ae96c4c91e9202643870458fb2900000000000000000000000000000000000000000000000000000000000000019000000000000000000000000000000000000000000000000000000000000006000000000000000000000000076fb31fb4af56892a25e32cfc43de717950c9278000000000000000000000000514bcb1f9aabb904e6106bd1052b66d2706dbbb700000000000000000000000000000000000000000000000000751d1aa0c0e9f7";

        assert_eq!(HexEncode(encoded), expected);
    }

    #[test]
    fn test_encode_usdce_to_eth() {
        // https://optimistic.etherscan.io/tx/0x4a81ba47adfb9720f792eb08cef9a4d444db7f6ff574c9adc4870188acb1cb18
        let token_usdce = Address::from_str("0x7F5c764cBc14f9669B88837ca1490cCa17c31607").unwrap();
        let token_weth = Address::from_str(OP_WETH).unwrap();
        let op_router = Address::from_str(OP_ROUTER).unwrap();
        let commands: Vec<UniversalRouterCommand> = vec![
            UniversalRouterCommand::PERMIT2_PERMIT(Permit2Permit {
                permit_single: IAllowanceTransfer::PermitSingle {
                    details: IAllowanceTransfer::PermitDetails {
                        token: token_usdce,
                        amount: U160::from_str("1461501637330902918203684832716283019655932542975").unwrap(),
                        expiration: U48::from(1732667502),
                        nonce: U48::from(0u64),
                    },
                    spender: op_router,
                    sigDeadline: U256::from(1730077302),
                },
                signature: Bytes::from(
                    hex::decode(
                        "00e96ed0f5bf5cca62dc9d9753960d83c8be83224456559a1e93a66d972a019f6f328a470f8257d3950b4cb7cd0024d789b4fcd9e80c4eb43d82a38d9e5332f31b",
                    )
                    .unwrap(),
                ),
            }),
            UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                recipient: Address::from_str(ADDRESS_THIS).unwrap(),
                amount_in: U256::from(10000000),
                amount_out_min: U256::from(0),
                path: Bytes::from(hex::decode("7f5c764cbc14f9669b88837ca1490cca17c316070001f44200000000000000000000000000000000000006").unwrap()),
                payer_is_user: true,
            }),
            UniversalRouterCommand::PAY_PORTION(PayPortion {
                token: token_weth,
                recipient: Address::from_str("0x3d83ec320541aE96C4C91E9202643870458fB290").unwrap(),
                bips: U256::from(25),
            }),
            UniversalRouterCommand::UNWRAP_WETH(UnwrapWeth {
                recipient: Address::from_str("0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7").unwrap(),
                amount_min: U256::from(3947534142938833u64),
            }),
        ];
        let deadline = U256::from(1730071269789u64);
        let encoded = encode_commands(&commands, deadline);
        // drop last 0c byte
        let expected = "0x3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000192d048919d00000000000000000000000000000000000000000000000000000000000000040a00060c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000032000000000000000000000000000000000000000000000000000000000000003a000000000000000000000000000000000000000000000000000000000000001600000000000000000000000007f5c764cbc14f9669b88837ca1490cca17c31607000000000000000000000000ffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000000000006746686e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000cb1355ff08ab38bbce60111f1bb2b784be25d7e800000000000000000000000000000000000000000000000000000000671ee27600000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000004100e96ed0f5bf5cca62dc9d9753960d83c8be83224456559a1e93a66d972a019f6f328a470f8257d3950b4cb7cd0024d789b4fcd9e80c4eb43d82a38d9e5332f31b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000989680000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002b7f5c764cbc14f9669b88837ca1490cca17c316070001f44200000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000042000000000000000000000000000000000000060000000000000000000000003d83ec320541ae96c4c91e9202643870458fb29000000000000000000000000000000000000000000000000000000000000000190000000000000000000000000000000000000000000000000000000000000040000000000000000000000000514bcb1f9aabb904e6106bd1052b66d2706dbbb7000000000000000000000000000000000000000000000000000e0642ea541ed1";

        assert_eq!(HexEncode(encoded), expected);
    }

    #[test]
    fn test_encode_exact_out_eth_to_usdc() {
        // https://optimistic.etherscan.io/tx/0x5e23648378c8461972730a55b3110242aef350d7e188bcc1df7007050926731d
        let commands: Vec<UniversalRouterCommand> = vec![
            UniversalRouterCommand::WRAP_ETH(WrapEth {
                recipient: Address::from_str(ADDRESS_THIS).unwrap(),
                amount_min: U256::from(2024000164272186u64),
            }),
            UniversalRouterCommand::V3_SWAP_EXACT_OUT(V3SwapExactOut {
                recipient: Address::from_str(ADDRESS_THIS).unwrap(),
                amount_out: U256::from(5012500),
                amount_in_max: U256::from(2024000164272186u64),
                path: Bytes::from(hex::decode("0b2c639c533813f4aa9d7837caf62653d097ff850001f44200000000000000000000000000000000000006").unwrap()),
                payer_is_user: false,
            }),
            UniversalRouterCommand::TRANSFER(Transfer {
                token: Address::from_str(OP_USDC).unwrap(),
                recipient: Address::from_str("0x7FFC3DBF3B2b50Ff3A1D5523bc24Bb5043837B14").unwrap(),
                value: U256::from(12500),
            }),
            UniversalRouterCommand::SWEEP(Sweep {
                token: Address::from_str(OP_USDC).unwrap(),
                recipient: Address::from_str("0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7").unwrap(),
                amount_min: U256::from(5000000),
            }),
            UniversalRouterCommand::UNWRAP_WETH(UnwrapWeth {
                recipient: Address::from_str("0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7").unwrap(),
                amount_min: U256::from(0),
            }),
        ];

        let deadline = U256::from(1730069397558u64);
        let encoded = encode_commands(&commands, deadline);
        // drop last 0c byte
        let expected = "0x3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000192d02c003600000000000000000000000000000000000000000000000000000000000000050b0105040c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000000000000000000000000000000000000000032000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000730d142d1183a0000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000004c7c14000000000000000000000000000000000000000000000000000730d142d1183a00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002b0b2c639c533813f4aa9d7837caf62653d097ff850001f4420000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000b2c639c533813f4aa9d7837caf62653d097ff850000000000000000000000007ffc3dbf3b2b50ff3a1d5523bc24bb5043837b1400000000000000000000000000000000000000000000000000000000000030d400000000000000000000000000000000000000000000000000000000000000600000000000000000000000000b2c639c533813f4aa9d7837caf62653d097ff85000000000000000000000000514bcb1f9aabb904e6106bd1052b66d2706dbbb700000000000000000000000000000000000000000000000000000000004c4b400000000000000000000000000000000000000000000000000000000000000040000000000000000000000000514bcb1f9aabb904e6106bd1052b66d2706dbbb70000000000000000000000000000000000000000000000000000000000000000";

        assert_eq!(HexEncode(encoded), expected);
    }
}
