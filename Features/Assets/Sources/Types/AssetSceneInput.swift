// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Store

public struct AssetSceneInput: Sendable {
    public let wallet: Wallet
    public let asset: Asset

    public var assetRequest: ChainAssetRequest
    public var transactionsRequest: TransactionsRequest
    public var bannersRequest: BannersRequest

    public init(wallet: Wallet, asset: Asset) {
        self.wallet = wallet
        self.asset = asset

        self.assetRequest = ChainAssetRequest(
            walletId: wallet.walletId,
            assetId: asset.id
        )

        self.transactionsRequest = TransactionsRequest.assetScene(
            walletId: wallet.walletId,
            assetId: asset.id
        )

        self.bannersRequest = BannersRequest(
            walletId: wallet.walletId,
            assetId: asset.id,
            chain: asset.id.chain,
            events: BannerEvent.allCases
        )
    }
}
