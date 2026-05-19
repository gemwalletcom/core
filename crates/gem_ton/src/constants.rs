// Transaction opcodes
pub const JETTON_TRANSFER_OPCODE: u32 = 0x0f8a7ea5;
#[cfg(feature = "signer")]
pub(crate) const NFT_TRANSFER_OPCODE: u32 = 0x5fcc3d14;

// NFT transfer message amounts
#[cfg(feature = "signer")]
pub(crate) const NFT_TRANSFER_FORWARD_AMOUNT: u64 = 10_000_000;
#[cfg(any(feature = "rpc", feature = "signer"))]
pub(crate) const NFT_TRANSFER_ATTACHMENT: u64 = 50_000_000;

// TON proxy jetton used by STON.fi for native TON swaps.
pub const TON_PROXY_JETTON_ADDRESS: &str = "EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c";

// Failed operation opcodes - operations that may show blockchain success but represent failed application operations
pub const JETTON_FAILED_OPERATION_OPCODE: &str = "0x93be2305";

// Additional potential failure opcodes found in test data
pub const FAILED_OPERATION_OPCODES: &[&str] = &[
    "0x93be2305", // Failed jetton operation
    "0xd6182fce", // Another failure pattern
    "0x77d0fee6", // Another failure pattern
    "0x98ce9044", // Failed jetton operation (insufficient funds)
];
