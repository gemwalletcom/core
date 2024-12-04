// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone

let SWAP_OPTIONS = GemSwapOptions(
    slippageBps: 100,
    fee: SwapReferralFees(
        evm: .init(address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7", bps: 25),
        solana: .init(address: "97q7TdebuvmxXCM1JzgqzpB1i7Wgvk4ACUWanhiL6Dk1", bps: 25),
        solanaJupiter: .init(address: "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC", bps: 25),
        thorchain: .init(address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7", bps: 25)
    ),
    preferredProviders: []
)

public extension SwapQuoteRequest {
    static let eth2usdc = SwapQuoteRequest(
        fromAsset: "ethereum",
        toAsset: "ethereum_0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        walletAddress: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
        destinationAddress: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
        value: "100000000000000000", // 0.01 ETH
        mode: .exactIn,
        options: nil
    )

    static let usdc2eth = SwapQuoteRequest(
        fromAsset: "ethereum_0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        toAsset: "ethereum",
        walletAddress: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
        destinationAddress: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
        value: "100000000", // 100 USDC
        mode: .exactIn,
        options: nil
    )

    static let sol2usdc = SwapQuoteRequest(
        fromAsset: "solana",
        toAsset: "solana_EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        walletAddress: "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC",
        destinationAddress: "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC",
        value: "1000000000", // 1 SOL
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let uni2link = SwapQuoteRequest(
        fromAsset: "optimism_0x6fd9d7AD17242c41f7131d257212c54A0e816691",
        toAsset: "optimism_0x350a791Bfc2C21F9Ed5d10980Dad2e2638ffa7f6",
        walletAddress: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
        destinationAddress: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
        value: "100000000000000000000", // 100 UNI
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let cake2btcb = SwapQuoteRequest(
        fromAsset: "smartchain_0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82",
        toAsset: "smartchain_0x7130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c",
        walletAddress: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
        destinationAddress: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
        value: "1000000000000000000000", // 1000 Cake
        mode: .exactIn,
        options: SWAP_OPTIONS
    )
}
