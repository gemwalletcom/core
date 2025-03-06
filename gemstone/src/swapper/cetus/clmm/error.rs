use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
pub enum ErrorCode {
    #[error("Integer downcast overflow")]
    IntegerDowncastOverflow,
    #[error("Multiplication overflow")]
    MulOverflow,
    #[error("Multiplication/division overflow")]
    MulDivOverflow,
    #[error("Multiplication right shift overflow")]
    MulShiftRightOverflow,
    #[error("Multiplication left shift overflow")]
    MulShiftLeftOverflow,
    #[error("Division by zero")]
    DivideByZero,
    #[error("Unsigned integer overflow")]
    UnsignedIntegerOverflow,
    #[error("Invalid coin amount")]
    InvalidCoinAmount,
    #[error("Invalid liquidity amount")]
    InvalidLiquidityAmount,
    #[error("Invalid reserve amount")]
    InvalidReserveAmount,
    #[error("Invalid sqrt price")]
    InvalidSqrtPrice,
    #[error("This coin is not supported")]
    NotSupportedThisCoin,
    #[error("Invalid two tick index")]
    InvalidTwoTickIndex,
    #[error("Coin amount exceeded maximum")]
    CoinAmountMaxExceeded,
    #[error("Coin amount below minimum")]
    CoinAmountMinSubceeded,
    #[error("Square root price out of bounds")]
    SqrtPriceOutOfBounds,
}
