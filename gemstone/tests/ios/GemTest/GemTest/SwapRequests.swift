// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone

let SWAP_OPTIONS = GemSwapOptions(
    slippage: getDefaultSlippage(chain: "solana"),
    fee: Config().getSwapConfig().referralFee,
    preferredProviders: []
)

let TEST_ETH_WALLET = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7"
let TEST_SOL_WALLET = "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC"
let TEST_BTC_WALLET = "bc1qe7qlndxgfv76c0ulnfhh7j0vdwkqdkkl4yf9gm"
let TEST_SUI_WALLET = "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1"

public extension SwapQuoteRequest {
    static let eth2usdc = SwapQuoteRequest(
        fromAsset: GemQuoteAsset(id: "ethereum", decimals: 18),
        toAsset: GemQuoteAsset(id: "ethereum_0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", decimals: 18),
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "100000000000000000", // 0.01 ETH
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let usdc2eth = SwapQuoteRequest(
        fromAsset: GemQuoteAsset(id: "ethereum_0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", decimals: 6),
        toAsset: GemQuoteAsset(id: "ethereum", decimals: 18),
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "100000000", // 100 USDC
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let sol2usdc = SwapQuoteRequest(
        fromAsset: GemQuoteAsset(id: "solana", decimals: 9),
        toAsset: GemQuoteAsset(id: "solana_EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", decimals: 6),
        walletAddress: TEST_SOL_WALLET,
        destinationAddress: TEST_SOL_WALLET,
        value: "1000000000", // 1 SOL
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let jup2bonk = SwapQuoteRequest(
        fromAsset: GemQuoteAsset(id: "solana_JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN", decimals: 6),
        toAsset: GemQuoteAsset(id: "solana_DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", decimals: 5),
        walletAddress: TEST_SOL_WALLET,
        destinationAddress: TEST_SOL_WALLET,
        value: "1000000000", // 1000 JUP
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let uni2link = SwapQuoteRequest(
        fromAsset: GemQuoteAsset(id: "optimism_0x6fd9d7AD17242c41f7131d257212c54A0e816691", decimals: 18),
        toAsset: GemQuoteAsset(id: "optimism_0x350a791Bfc2C21F9Ed5d10980Dad2e2638ffa7f6", decimals: 18),
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "100000000000000000000", // 100 UNI
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let cake2btcb = SwapQuoteRequest(
        fromAsset: GemQuoteAsset(id: "smartchain_0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82", decimals: 18),
        toAsset: GemQuoteAsset(id: "smartchain_0x7130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c", decimals: 18),
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "1000000000000000000000", // 1000 Cake
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let cake2bnb = SwapQuoteRequest(
        fromAsset: GemQuoteAsset(id: "smartchain_0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82", decimals: 18),
        toAsset: GemQuoteAsset(id: "smartchain", decimals: 18),
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "400000000000000000", // 0.4
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let op2Eth = SwapQuoteRequest(
        fromAsset: GemQuoteAsset(id: "optimism",decimals: 18),
        toAsset: GemQuoteAsset(id: "ethereum", decimals: 18),
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "20000000000000000", // 0.02 eth
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let op2Arb = SwapQuoteRequest(
        fromAsset: GemQuoteAsset(id: "optimism", decimals: 18),
        toAsset: GemQuoteAsset(id: "arbitrum", decimals: 18),
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "10000000000000000", // 0.01 eth
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let op2Ink = SwapQuoteRequest(
        fromAsset: GemQuoteAsset(id: "optimism", decimals: 18),
        toAsset: GemQuoteAsset(id: "ink", decimals: 18),
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "10000000000000000", // 0.01 eth
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let eth2Unichain = SwapQuoteRequest(
        fromAsset: GemQuoteAsset(id: "ethereum", decimals: 18),
        toAsset: GemQuoteAsset(id: "unichain", decimals: 18),
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "10000000000000000", // 0.01 eth
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let ethUSDC2Base = SwapQuoteRequest(
        fromAsset: GemQuoteAsset(id: "ethereum_0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", decimals: 6),
        toAsset: GemQuoteAsset(id: "base_0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", decimals: 18),
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "10000000000", // 10000 USDC
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let baseUSDC2Eth = SwapQuoteRequest(
        fromAsset: GemQuoteAsset(id: "base_0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", decimals: 6),
        toAsset: GemQuoteAsset(id: "ethereum_0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", decimals: 18),
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "10000000000", // 10000 USDC
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let absETH2USDC = SwapQuoteRequest(
        fromAsset: GemQuoteAsset(id: "abstract", decimals: 18),
        toAsset: GemQuoteAsset(id: "abstract_0x84A71ccD554Cc1b02749b35d22F684CC8ec987e1", decimals: 6),
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "1000000000000000000", // 1 ETH
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let eth2usdc_v4: SwapQuoteRequest = {
        var options = SWAP_OPTIONS
        options.preferredProviders = [SwapProvider.uniswapV4]

        return SwapQuoteRequest(
            fromAsset: GemQuoteAsset(id: "unichain", decimals: 18),
            toAsset: GemQuoteAsset(id: "unichain_0x078D782b760474a361dDA0AF3839290b0EF57AD6", decimals: 6),
            walletAddress: TEST_ETH_WALLET,
            destinationAddress: TEST_ETH_WALLET,
            value: "1000000000000000000", // 1 ETH
            mode: .exactIn,
            options: options
        )
    }()

    static let uni2dai_v4: SwapQuoteRequest = .init(
        fromAsset: GemQuoteAsset(id: "unichain_0x8f187aA05619a017077f5308904739877ce9eA21", decimals: 18),
        toAsset: GemQuoteAsset(id: "unichain_0x20CAb320A855b39F724131C69424240519573f81", decimals: 18),
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_ETH_WALLET,
        value: "1000000000000000000", // 1 ETH
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let eth2btc: SwapQuoteRequest = .init(
        fromAsset: GemQuoteAsset(id: "ethereum", decimals: 18),
        toAsset: GemQuoteAsset(id: "bitcoin", decimals: 8),
        walletAddress: TEST_ETH_WALLET,
        destinationAddress: TEST_BTC_WALLET,
        value: "1000000000000000000", // 1 ETH
        mode: .exactIn,
        options: SWAP_OPTIONS
    )

    static let sui2USDC = SwapQuoteRequest(
        fromAsset: GemQuoteAsset(id: "sui", decimals: 9),
        toAsset: GemQuoteAsset(id: "sui_0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC",decimals: 6),
        walletAddress: TEST_SUI_WALLET,
        destinationAddress: TEST_SUI_WALLET,
        value: "100000000000", // 100 SUI
        mode: .exactIn,
        options: SWAP_OPTIONS
    )
}
