// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PriceService
import WebSocketClient

public actor StreamSubscriptionService: Sendable {
    private let priceService: PriceService
    private let webSocket: any WebSocketConnectable
    private let encoder = JSONEncoder()

    private var subscribedAssetIds: Set<AssetId> = []
    private var currentWalletId: WalletId?

    public init(
        priceService: PriceService,
        webSocket: any WebSocketConnectable
    ) {
        self.priceService = priceService
        self.webSocket = webSocket
    }

    public func setupAssets(walletId: WalletId) async throws {
        currentWalletId = walletId
        let assets = try priceService.observableAssets(walletId: walletId)
        let message = StreamMessage.subscribePrices(StreamMessagePrices(assets: assets))
        try await sendMessage(message)
        subscribedAssetIds = Set(assets)
    }

    public func resubscribe() async {
        guard let walletId = currentWalletId else { return }
        do {
            try await setupAssets(walletId: walletId)
        } catch {
            debugLog("stream subscription: resubscribe failed: \(error)")
        }
    }

    private func sendMessage(_ message: StreamMessage) async throws {
        let data = try encoder.encode(message)
        try await webSocket.send(data)
        debugLog("stream subscription send message: \(message)")
    }
}

// MARK: - PriceUpdater

extension StreamSubscriptionService: PriceUpdater {
    public func addPrices(assetIds: [AssetId]) async throws {
        let newAssets = Set(assetIds).subtracting(subscribedAssetIds).asArray()
        guard newAssets.isNotEmpty else { return }

        try await sendMessage(StreamMessage.addPrices(StreamMessagePrices(assets: newAssets)))
        subscribedAssetIds.formUnion(newAssets)
    }
}
