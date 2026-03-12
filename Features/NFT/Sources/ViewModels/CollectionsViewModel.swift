// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Components
import NFTService
import Primitives
import Store
import Localization
import SwiftUI
import PrimitivesComponents
import WalletService
import Style

@Observable
@MainActor
public final class CollectionsViewModel: CollectionsViewable, Sendable {
    private let walletService: WalletService
    private let nftService: NFTService

    public let query: ObservableQuery<NFTRequest>
    public var nftDataList: [NFTData] { query.value }

    public var isPresentingReceiveSelectAssetType: SelectAssetType?

    public var wallet: Wallet

    public init(
        nftService: NFTService,
        walletService: WalletService,
        wallet: Wallet
    ) {
        self.nftService = nftService
        self.walletService = walletService
        self.wallet = wallet
        self.query = ObservableQuery(NFTRequest(walletId: wallet.walletId, filter: .all), initialValue: [])
    }

    public var title: String { Localized.Nft.collections }

    public var currentWallet: Wallet? {
        walletService.currentWallet
    }

    public var content: CollectionsContent {
        CollectionsContent(
            items: verifiedItems,
            unverifiedCount: unverifiedCount
        )
    }

    // MARK: - Private

    private var verifiedItems: [GridPosterViewItem] {
        nftDataList
            .filter { $0.collection.status == .verified }
            .map { buildGridItem(from: $0) }
    }

    private var unverifiedCount: String? {
        let unverified = nftDataList.filter { $0.collection.status != .verified }
        guard unverified.isNotEmpty else { return nil }
        return unverified.count.asString
    }

    // MARK: - Actions

    public func fetch() async {
        do {
            let count = try await nftService.updateAssets(wallet: wallet)
            debugLog("update nfts: \(count)")
        } catch {
            debugLog("update nfts error: \(error)")
        }
    }
}
