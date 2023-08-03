#[typeshare]
struct BNBChainMessageFee {
    msg_type: String,
    fee: u32,
}

#[typeshare]
struct BNBChainFixedFee {
    fixed_fee_params: BNBChainMessageFee,
}

#[typeshare]
pub enum BNBChainMessageFeeType {
    send,
}