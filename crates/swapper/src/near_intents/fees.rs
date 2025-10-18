use primitives::Chain;
use std::{collections::HashMap, sync::LazyLock};

// Reserved fees represent approximately two simple native transfers per chain, in base units.
pub static RESERVED_NATIVE_FEES: LazyLock<HashMap<Chain, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (Chain::Near, "50000000000000000000000"), // 0.05 NEAR
        (Chain::Ethereum, "1000000000000000"),    // 0.001 ETH
        (Chain::Arbitrum, "300000000000000"),     // 0.0003 ARB ETH
        (Chain::Base, "300000000000000"),         // 0.0003 BASE ETH
        (Chain::Optimism, "500000000000000"),     // 0.0005 OP ETH
        (Chain::AvalancheC, "3000000000000000"),  // 0.003 AVAX
        (Chain::SmartChain, "2000000000000000"),  // 0.002 BNB
        (Chain::Polygon, "20000000000000000"),    // 0.02 MATIC
        (Chain::Gnosis, "5000000000000000"),      // 0.005 XDAI
        (Chain::Berachain, "5000000000000000"),   // 0.005 BERA
        (Chain::Sui, "50000000"),                 // 0.05 SUI
        (Chain::Solana, "20000"),                 // 0.00002 SOL
        (Chain::Ton, "20000000"),                 // 0.02 TON
        (Chain::Tron, "20000000"),                // 20 TRX
        (Chain::Bitcoin, "40000"),                // 0.0004 BTC
        (Chain::Zcash, "1000000"),                // 0.01 ZEC
        (Chain::Doge, "500000000"),               // 5 DOGE
        (Chain::Xrp, "2000000"),                  // 2 XRP
        (Chain::Cardano, "2000000"),              // 2 ADA
        (Chain::Aptos, "20000000"),               // 0.2 APT
        (Chain::Stellar, "100000"),               // 0.01 XLM
    ])
});

pub fn reserved_tx_fees(chain: Chain) -> Option<&'static str> {
    RESERVED_NATIVE_FEES.get(&chain).copied()
}
