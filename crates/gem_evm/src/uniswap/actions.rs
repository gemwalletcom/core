use alloy_primitives::{Address, U256};

use super::contracts::v4::{Currency, IV4Router};

// https://github.com/Uniswap/v4-periphery/blob/main/src/libraries/Actions.sol
#[allow(non_camel_case_types)]
pub enum V4Action {
    SWAP_EXACT_IN(IV4Router::ExactInputParams),
    SWAP_EXACT_IN_SINGLE(IV4Router::ExactInputSingleParams),
    SWAP_EXACT_OUT(IV4Router::ExactOutputParams),
    SWAP_EXACT_OUT_SINGLE(IV4Router::ExactOutputSingleParams),
    SETTLE { currency: Currency, amount: U256, payer_is_user: bool },
    SETTLE_ALL { currency: Currency, max_amount: U256 },
    TAKE { currency: Currency, recipient: Address, amount: U256 },
    TAKE_ALL { currency: Currency, min_amount: U256 },
    TAKE_PORTION { currency: Currency, recipient: Address, bips: U256 },
}

impl V4Action {
    pub fn byte(&self) -> u8 {
        match self {
            Self::SWAP_EXACT_IN(_) => 0x00,
            Self::SWAP_EXACT_IN_SINGLE(_) => 0x01,
            Self::SWAP_EXACT_OUT(_) => 0x02,
            Self::SWAP_EXACT_OUT_SINGLE(_) => 0x03,
            Self::SETTLE {
                currency: _,
                amount: _,
                payer_is_user: _,
            } => 0x0b,
            Self::SETTLE_ALL { currency: _, max_amount: _ } => 0x0c,
            Self::TAKE {
                currency: _,
                recipient: _,
                amount: _,
            } => 0x0e,
            Self::TAKE_ALL { currency: _, min_amount: _ } => 0x0f,
            Self::TAKE_PORTION {
                currency: _,
                recipient: _,
                bips: _,
            } => 0x10,
        }
    }
}
