use alloy_core::primitives::{Address, Bytes, U256};

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

#[derive(Debug, PartialEq)]
pub struct V3SwapExactIn {
    pub recipient: Address,
    pub amount_in: U256,
    pub amount_out_min: U256,
    pub path: Bytes,
    pub payer_is_user: bool,
}

#[derive(Debug, PartialEq)]
pub struct V3SwapExactOut {
    pub recipient: Address,
    pub amount_out: U256,
    pub amount_in_max: U256,
    pub path: Bytes,
    pub payer_is_user: bool,
}

#[derive(Debug, PartialEq)]
pub struct Sweep {
    pub token: Address,
    pub recipient: Address,
    pub amount_min: U256,
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

#[derive(Debug, PartialEq)]
pub struct WrapEth {
    pub recipient: Address,
    pub amount_min: U256,
}

#[derive(Debug, PartialEq)]
pub struct UnwrapWeth {
    pub recipient: Address,
    pub amount_min: U256,
}

impl From<UniversalRouterCommand> for u8 {
    fn from(val: UniversalRouterCommand) -> Self {
        match val {
            UniversalRouterCommand::V3_SWAP_EXACT_IN(_) => 0x00,
            UniversalRouterCommand::V3_SWAP_EXACT_OUT(_) => 0x01,
            UniversalRouterCommand::PERMIT2_TRANSFER_FROM => 0x02,
            UniversalRouterCommand::PERMIT2_PERMIT_BATCH => 0x03,
            UniversalRouterCommand::SWEEP(_) => 0x04,
            UniversalRouterCommand::TRANSFER(_) => 0x05,
            UniversalRouterCommand::PAY_PORTION(_) => 0x06,
            UniversalRouterCommand::V2_SWAP_EXACT_IN => 0x08,
            UniversalRouterCommand::V2_SWAP_EXACT_OUT => 0x09,
            UniversalRouterCommand::PERMIT2_PERMIT => 0x0a,
            UniversalRouterCommand::WRAP_ETH(_) => 0x0b,
            UniversalRouterCommand::UNWRAP_WETH(_) => 0x0c,
            UniversalRouterCommand::PERMIT2_TRANSFER_FROM_BATCH => 0x0d,
        }
    }
}
