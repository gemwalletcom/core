// Transaction opcodes
pub const JETTON_TRANSFER_OPCODE: &str = "0x0f8a7ea5";

// Failed operation opcodes - operations that may show blockchain success but represent failed application operations
pub const JETTON_FAILED_OPERATION_OPCODE: &str = "0x93be2305";

// Additional potential failure opcodes found in test data
pub const FAILED_OPERATION_OPCODES: &[&str] = &[
    "0x93be2305", // Failed jetton operation
    "0xd6182fce", // Another failure pattern
    "0x77d0fee6", // Another failure pattern
];
