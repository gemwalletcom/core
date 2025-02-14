use super::contracts::v4::IV4Router;

// https://github.com/Uniswap/v4-periphery/blob/main/src/libraries/Actions.sol
#[allow(non_camel_case_types)]
pub enum V4Actions {
    SWAP_EXACT_IN(IV4Router::ExactInputParams),
    SWAP_EXACT_IN_SINGLE(IV4Router::ExactInputSingleParams),
    SWAP_EXACT_OUT(IV4Router::ExactOutputParams),
    SWAP_EXACT_OUT_SINGLE(IV4Router::ExactOutputSingleParams),
    SETTLE,
    SETTLE_ALL,
    SETTLE_PAIR,
    TAKE,
    TAKE_ALL,
    TAKE_PORTION,
    TAKE_PAIR,
    CLOSE_CURRENCY,
    CLEAR_OR_TAKE,
    SWEEP,
    WRAP,
    UNWRAP,
}

impl V4Actions {
    pub fn byte(&self) -> u8 {
        match self {
            Self::SWAP_EXACT_IN(_) => 0x00,
            Self::SWAP_EXACT_IN_SINGLE(_) => 0x01,
            Self::SWAP_EXACT_OUT(_) => 0x02,
            Self::SWAP_EXACT_OUT_SINGLE(_) => 0x03,
            Self::SETTLE => 0x0b,
            Self::SETTLE_ALL => 0x0c,
            Self::SETTLE_PAIR => 0x0d,
            Self::TAKE => 0x0e,
            Self::TAKE_ALL => 0x0f,
            Self::TAKE_PORTION => 0x10,
            Self::TAKE_PAIR => 0x11,
            Self::CLOSE_CURRENCY => 0x12,
            Self::CLEAR_OR_TAKE => 0x13,
            Self::SWEEP => 0x14,
            Self::WRAP => 0x15,
            Self::UNWRAP => 0x16,
        }
    }
}
