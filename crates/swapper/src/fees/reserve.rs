use alloy_primitives::U256;
use primitives::Chain;
use std::{collections::HashMap, str::FromStr, sync::LazyLock};

use crate::{QuoteRequest, SwapperError};

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
        (Chain::Xrp, "2000000"),                  // 2 XRP
        (Chain::Cardano, "2000000"),              // 2 ADA
        (Chain::Aptos, "20000000"),               // 0.2 APT
        (Chain::Stellar, "100000"),               // 0.01 XLM
        (Chain::Monad, "5000000000000000"),       // 0.005 MON
        (Chain::XLayer, "5000000000000000"),      // 0.005 OKB
        (Chain::Plasma, "5000000000000000"),      // 0.005 XPL
        (Chain::Cosmos, "39000"),                 // 0.039 ATOM
        (Chain::Osmosis, "130000"),               // 0.13 OSMO
        (Chain::Celestia, "39000"),               // 0.039 TIA
        (Chain::Injective, "1300000000000000"),   // 0.0013 INJ
        (Chain::Sei, "1300000"),                  // 1.3 SEI
        (Chain::Noble, "25000"),                  // 0.025 USDC
        // UTXO fee-vs-slippage buffer for amount-sensitive max swaps.
        (Chain::Bitcoin, "15000"),     // ~300 vB * ~50 sat/vB peak
        (Chain::Litecoin, "15000"),    // ~300 vB * ~50 lit/vB peak
        (Chain::BitcoinCash, "10000"), // ~300 B * ~30 sat/B peak
        (Chain::Doge, "10000000"),     // 0.1 DOGE
        (Chain::Zcash, "30000"),       // 3x ZIP-317 marginal fee
    ])
});

pub fn reserved_tx_fees(chain: Chain) -> Option<&'static str> {
    RESERVED_NATIVE_FEES.get(&chain).copied()
}

pub fn quote_value_after_reserve(request: &QuoteRequest, reserved: &str) -> Result<String, SwapperError> {
    if !request.options.use_max_amount || !request.from_asset.asset_id().is_native() {
        return Ok(request.value.clone());
    }
    let reserved_fee = U256::from_str(reserved).map_err(|_| SwapperError::ComputeQuoteError(format!("invalid reserved fee: {reserved}")))?;
    let amount = U256::from_str(&request.value).map_err(|_| SwapperError::ComputeQuoteError(format!("invalid amount: {}", request.value)))?;
    if amount <= reserved_fee {
        return Err(SwapperError::InputAmountError {
            min_amount: Some(reserved_fee.to_string()),
        });
    }
    Ok((amount - reserved_fee).to_string())
}

pub fn quote_value_after_reserve_by_chain(request: &QuoteRequest) -> Result<String, SwapperError> {
    let Some(reserved) = reserved_tx_fees(request.from_asset.chain()) else {
        return Ok(request.value.clone());
    };
    quote_value_after_reserve(request, reserved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SwapperQuoteAsset, testkit::mock_bitcoin_max_quote};

    #[test]
    fn btc_family_chains_have_buffer_reserve() {
        assert_eq!(reserved_tx_fees(Chain::Bitcoin), Some("15000"));
        assert_eq!(reserved_tx_fees(Chain::Litecoin), Some("15000"));
        assert_eq!(reserved_tx_fees(Chain::BitcoinCash), Some("10000"));
        assert_eq!(reserved_tx_fees(Chain::Doge), Some("10000000"));
        assert_eq!(reserved_tx_fees(Chain::Zcash), Some("30000"));
    }

    #[test]
    fn btc_max_quote_applies_reserve() {
        let request = mock_bitcoin_max_quote(SwapperQuoteAsset::from(Chain::Solana.as_asset_id()));

        assert_eq!(quote_value_after_reserve_by_chain(&request).unwrap(), "74100");
    }

    #[test]
    fn btc_reserve_exceeds_typical_planner_fee() {
        let reserve: u64 = reserved_tx_fees(Chain::Bitcoin).unwrap().parse().unwrap();
        assert!(reserve >= 12_000, "Bitcoin reserve {reserve} too small to absorb peak-fee planner cost");
    }
}
