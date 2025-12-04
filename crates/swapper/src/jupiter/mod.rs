mod client;
mod default;
mod model;
mod provider;
pub use provider::Jupiter;

pub const PROGRAM_ADDRESS: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";

// https://dev.jup.ag/api-reference/swap/program-id-to-label
pub const DEFAULT_DEXES: &str = "Raydium,Orca V2,Meteora DLMM,Raydium CLMM,Whirlpool,Phoenix,Meteora,Lifinity V2,Pump.fun,Pump.fun Amm";
