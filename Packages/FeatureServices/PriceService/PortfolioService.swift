// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public struct PortfolioService: Sendable {
    private let apiService: any GemAPIPortfolioService

    public init(apiService: any GemAPIPortfolioService) {
        self.apiService = apiService
    }

    public func getPortfolioAssets(assets: [AssetData], period: ChartPeriod) async throws -> PortfolioAssets {
        let portfolioAssets = assets.map { PortfolioAsset(assetId: $0.asset.id, value: String($0.balance.total)) }
        let request = PortfolioAssetsRequest(assets: portfolioAssets)
        return try await apiService.getPortfolioAssets(period: period, request: request)
    }
}
