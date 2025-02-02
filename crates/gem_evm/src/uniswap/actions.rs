use super::contracts::v4::IV4Router;

// https://github.com/Uniswap/v4-periphery/blob/main/src/libraries/Actions.sol

#[allow(non_camel_case_types)]
pub enum V4Actions {
    SWAP_EXACT_IN(IV4Router::ExactInputParams),
    SWAP_EXACT_IN_SINGLE(IV4Router::ExactInputSingleParams),
    SWAP_EXACT_OUT(IV4Router::ExactOutputParams),
    SWAP_EXACT_OUT_SINGLE(IV4Router::ExactOutputSingleParams),
}

impl V4Actions {
    pub fn byte(&self) -> u8 {
        match self {
            Self::SWAP_EXACT_IN(_) => 0x00,
            Self::SWAP_EXACT_IN_SINGLE(_) => 0x01,
            Self::SWAP_EXACT_OUT(_) => 0x02,
            Self::SWAP_EXACT_OUT_SINGLE(_) => 0x03,
        }
    }
}
