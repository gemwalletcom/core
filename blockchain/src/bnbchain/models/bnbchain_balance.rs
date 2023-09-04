use typeshare::typeshare;

#[typeshare]
#[allow(dead_code)]
pub struct BNBChainBalance {
    pub free: String,
    pub frozen: String,
    pub locked: String,
    pub symbol: String
}