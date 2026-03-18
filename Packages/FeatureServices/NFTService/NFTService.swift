// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import GemAPI
import Store

public struct NFTService: Sendable {
    private let apiService: any GemAPINFTService
    private let nftStore: NFTStore

    public init(
        apiService: any GemAPINFTService,
        nftStore: NFTStore
    ) {
        self.apiService = apiService
        self.nftStore = nftStore
    }

    @discardableResult
    public func updateAssets(wallet: Wallet) async throws -> Int {
        let nfts = try await apiService.getDeviceNFTAssets(walletId: wallet.id)
        try nftStore.save(nfts, for: wallet.walletId)
        return nfts.count
    }

    public func report(collectionId: String, assetId: String?, reason: String?) async throws {
        let report = ReportNft(
            collectionId: collectionId,
            assetId: assetId,
            reason: reason
        )
        try await apiService.reportNft(report: report)
    }
}
