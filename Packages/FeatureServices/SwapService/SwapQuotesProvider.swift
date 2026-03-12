// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import BigInt
import Primitives

import struct Gemstone.SwapperQuote

public protocol SwapQuotesProvidable: Sendable {
    func supportedAssets(for assetId: AssetId) -> ([Primitives.Chain], [Primitives.AssetId])
    func fetchQuotes(wallet: Wallet, fromAsset: Asset, toAsset: Asset, amount: BigInt, useMaxAmount: Bool) async throws -> [Gemstone.SwapperQuote]
}

public struct SwapQuotesProvider: SwapQuotesProvidable {
    private let swapService: SwapService

    public init(swapService: SwapService) {
        self.swapService = swapService
    }

    public func supportedAssets(for assetId: AssetId) -> ([Primitives.Chain], [Primitives.AssetId]) {
        swapService.supportedAssets(for: assetId)
    }

    public func fetchQuotes(wallet: Wallet, fromAsset: Asset, toAsset: Asset, amount: BigInt, useMaxAmount: Bool) async throws -> [Gemstone.SwapperQuote] {
        let walletAddress = try wallet.account(for: fromAsset.chain).address
        let destinationAddress = try wallet.account(for: toAsset.chain).address
        let quotes = try await swapService.getQuotes(
            fromAsset: fromAsset,
            toAsset: toAsset,
            value: amount.description,
            walletAddress: walletAddress,
            destinationAddress: destinationAddress,
            useMaxAmount: useMaxAmount
        )
        return try quotes.sorted { try BigInt.from(string: $0.toValue) > BigInt.from(string: $1.toValue) }
    }
}
