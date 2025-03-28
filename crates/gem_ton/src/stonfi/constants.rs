/// Operation codes for DEX functions
pub const DEX_OP_CODES: DexOpCodes = DexOpCodes {
    swap: 0x6664de2a,
    cross_swap: 0x69cf1a5b,
    provide_lp: 0x37c096df,
    direct_add_liquidity: 0x0ff8bfc6,
    refund_me: 0x132b9a2c,
    reset_gas: 0x29d22935,
    collect_fees: 0x1ee4911e,
    burn: 0x595f07bc,
    withdraw_fee: 0x354bcdf4,
};

/// Constant deadline for transactions (in seconds)
pub const TX_DEADLINE: u64 = 15 * 60; // 15 minutes

pub struct DexOpCodes {
    pub swap: u32,
    pub cross_swap: u32,
    pub provide_lp: u32,
    pub direct_add_liquidity: u32,
    pub refund_me: u32,
    pub reset_gas: u32,
    pub collect_fees: u32,
    pub burn: u32,
    pub withdraw_fee: u32,
}
