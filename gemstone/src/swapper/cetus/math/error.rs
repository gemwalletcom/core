use std::fmt;

/// Error codes for math operations
#[derive(Debug, Clone, Copy)]
pub enum MathErrorCode {
    IntegerDowncastOverflow,
    MulOverflow,
    MulDivOverflow,
    MulShiftRightOverflow,
    MulShiftLeftOverflow,
    DivideByZero,
    UnsignedIntegerOverflow,
    InvalidCoinAmount,
    InvalidLiquidityAmount,
    InvalidReserveAmount,
    InvalidSqrtPrice,
    NotSupportedThisCoin,
    InvalidTwoTickIndex,
}

/// Error codes for coin operations
#[derive(Debug, Clone, Copy)]
pub enum CoinErrorCode {
    CoinAmountMaxExceeded,
    CoinAmountMinSubceeded,
    SqrtPriceOutOfBounds,
}

/// Generic error type for CLMM operations
#[derive(Debug)]
pub struct ClmmpoolsError {
    pub message: String,
    pub error_code: Option<ErrorCode>,
}

/// Combined error code enum
#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    Math(MathErrorCode),
    Coin(CoinErrorCode),
}

impl fmt::Display for MathErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MathErrorCode::IntegerDowncastOverflow => write!(f, "IntegerDowncastOverflow"),
            MathErrorCode::MulOverflow => write!(f, "MultiplicationOverflow"),
            MathErrorCode::MulDivOverflow => write!(f, "MulDivOverflow"),
            MathErrorCode::MulShiftRightOverflow => write!(f, "MulShiftRightOverflow"),
            MathErrorCode::MulShiftLeftOverflow => write!(f, "MulShiftLeftOverflow"),
            MathErrorCode::DivideByZero => write!(f, "DivideByZero"),
            MathErrorCode::UnsignedIntegerOverflow => write!(f, "UnsignedIntegerOverflow"),
            MathErrorCode::InvalidCoinAmount => write!(f, "InvalidCoinAmount"),
            MathErrorCode::InvalidLiquidityAmount => write!(f, "InvalidLiquidityAmount"),
            MathErrorCode::InvalidReserveAmount => write!(f, "InvalidReserveAmount"),
            MathErrorCode::InvalidSqrtPrice => write!(f, "InvalidSqrtPrice"),
            MathErrorCode::NotSupportedThisCoin => write!(f, "NotSupportedThisCoin"),
            MathErrorCode::InvalidTwoTickIndex => write!(f, "InvalidTwoTickIndex"),
        }
    }
}

impl fmt::Display for CoinErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoinErrorCode::CoinAmountMaxExceeded => write!(f, "CoinAmountMaxExceeded"),
            CoinErrorCode::CoinAmountMinSubceeded => write!(f, "CoinAmountMinSubceeded"),
            CoinErrorCode::SqrtPriceOutOfBounds => write!(f, "SqrtPriceOutOfBounds"),
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::Math(code) => write!(f, "{}", code),
            ErrorCode::Coin(code) => write!(f, "{}", code),
        }
    }
}

impl From<MathErrorCode> for ErrorCode {
    fn from(code: MathErrorCode) -> Self {
        ErrorCode::Math(code)
    }
}

impl From<CoinErrorCode> for ErrorCode {
    fn from(code: CoinErrorCode) -> Self {
        ErrorCode::Coin(code)
    }
}

impl ClmmpoolsError {
    pub fn new<S: Into<String>, E: Into<Option<ErrorCode>>>(message: S, error_code: E) -> Self {
        Self {
            message: message.into(),
            error_code: error_code.into(),
        }
    }
    
    pub fn math_error<S: Into<String>>(message: S, error_code: MathErrorCode) -> Self {
        Self {
            message: message.into(),
            error_code: Some(ErrorCode::Math(error_code)),
        }
    }
    
    pub fn coin_error<S: Into<String>>(message: S, error_code: CoinErrorCode) -> Self {
        Self {
            message: message.into(),
            error_code: Some(ErrorCode::Coin(error_code)),
        }
    }
}

impl fmt::Display for ClmmpoolsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(error_code) = &self.error_code {
            write!(f, "{}: {}", error_code, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ClmmpoolsError {}