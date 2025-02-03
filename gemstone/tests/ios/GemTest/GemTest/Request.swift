// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone

let SWAP_OPTIONS = GemSwapOptions(
    slippage: getDefaultSlippage(chain: "solana"),
    fee: SwapReferralFees(
        evm: .init(address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7", bps: 50),
        evmBridge: .init(address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7", bps: 25),
        solana: .init(address: "97q7TdebuvmxXCM1JzgqzpB1i7Wgvk4ACUWanhiL6Dk1", bps: 25),
        solanaJupiter: .init(address: "97q7TdebuvmxXCM1JzgqzpB1i7Wgvk4ACUWanhiL6Dk1", bps: 25),
        thorchain: .init(address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7", bps: 25)
    ),
    preferredProviders: []
)

let TEST_ETH_WALLET = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7"
let TEST_SOL_WALLET = "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC"

let TEST_SOLANA_PAY_URI = "solana:https%3A%2F%2Fapi.spherepay.co%2Fv1%2Fpublic%2FpaymentLink%2Fpay%2FpaymentLink_7fa254717ba443fcb0f9f632f553934b%3Fstate%3DAL%26country%3DUSA%26lineItems%3D%255B%257B%2522id%2522%253A%2522lineItem_c488a0e552c44806afccc318e50506f8%2522%252C%2522quantity%2522%253A100%257D%255D%26solanaPayReference%3DEWuRMCeC7Y8mHYxsfeXFsBidTRLAvvWfTj2nie3B2nxf%26paymentReference%3D5jSAdFduqFVyOAcR5fFoj%26network%3Dsol%26skipPreflight%3Dfalse"

public extension SwapQuoteRequest {
    static let eth2usdc = SwapQuoteRequest(
        fromAsset: "ethereum",
        toAsset: "ethereum_0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "100000000000000000", // 0.01 ETH
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let usdc2eth = SwapQuoteRequest(
        fromAsset: "ethereum_0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        toAsset: "ethereum",
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "100000000", // 100 USDC
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let sol2usdc = SwapQuoteRequest(
        fromAsset: "solana",
        toAsset: "solana_EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        walletAddress: TEST_SOL_WALLET,
        destinationAddress: TEST_SOL_WALLET,
        value: "1000000000", // 1 SOL
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let jup2bonk = SwapQuoteRequest(
        fromAsset: "solana_JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",
        toAsset: "solana_DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
        walletAddress: TEST_SOL_WALLET,
        destinationAddress: TEST_SOL_WALLET,
        value: "1000000000", // 1000 JUP
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let uni2link = SwapQuoteRequest(
        fromAsset: "optimism_0x6fd9d7AD17242c41f7131d257212c54A0e816691",
        toAsset: "optimism_0x350a791Bfc2C21F9Ed5d10980Dad2e2638ffa7f6",
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "100000000000000000000", // 100 UNI
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let cake2btcb = SwapQuoteRequest(
        fromAsset: "smartchain_0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82",
        toAsset: "smartchain_0x7130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c",
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "1000000000000000000000", // 1000 Cake
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let op2Eth = SwapQuoteRequest(
        fromAsset: "optimism",
        toAsset: "ethereum",
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "20000000000000000", // 0.02 eth
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let op2Arb = SwapQuoteRequest(
        fromAsset: "optimism",
        toAsset: "arbitrum",
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "10000000000000000", // 0.01 eth
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let ethUSDC2Base = SwapQuoteRequest(
        fromAsset: "ethereum_0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        toAsset: "base_0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "10000000000", // 10000 USDC
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let baseUSDC2Eth = SwapQuoteRequest(
        fromAsset: "base_0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
        toAsset: "ethereum_0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "10000000000", // 10000 USDC
        mode: .exactIn,
        options: SWAP_OPTIONS
    )
}
