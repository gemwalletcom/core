// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives
import Store
import Preferences
import AssetsService
import DeviceService

public final class TransactionsService: Sendable {
    let provider: any GemAPITransactionService
    public let transactionStore: TransactionStore
    let assetsService: AssetsService
    let walletStore: WalletStore
    private let deviceService: any DeviceServiceable
    private let addressStore: AddressStore
    
    public init(
        provider: any GemAPITransactionService,
        transactionStore: TransactionStore,
        assetsService: AssetsService,
        walletStore: WalletStore,
        deviceService: any DeviceServiceable,
        addressStore: AddressStore
    ) {
        self.provider = provider
        self.transactionStore = transactionStore
        self.assetsService = assetsService
        self.walletStore = walletStore
        self.deviceService = deviceService
        self.addressStore = addressStore
    }

    public func updateAll(walletId: WalletId) async throws {
        guard let wallet = try walletStore.getWallet(id: walletId) else {
            throw AnyError("Can't get a wallet, walletId: \(walletId.id)")
        }
        let store = WalletPreferences(walletId: walletId)
        let newTimestamp = Int(Date.now.timeIntervalSince1970)

        _ = try await deviceService.getSubscriptionsDeviceId()
        let response = try await provider.getDeviceTransactions(
            walletId: wallet.id,
            fromTimestamp: store.transactionsTimestamp
        )

        try await prefetchAssets(walletId: walletId, transactions: response.transactions)
        try transactionStore.addTransactions(walletId: walletId, transactions: response.transactions)
        try addressStore.addAddressNames(response.addressNames)

        store.transactionsTimestamp = newTimestamp
    }

    public func updateForAsset(wallet: Wallet, assetId: AssetId) async throws {
        let store = WalletPreferences(walletId: wallet.walletId)
        let newTimestamp = Int(Date.now.timeIntervalSince1970)
        _ = try await deviceService.getSubscriptionsDeviceId()
        let response = try await provider.getDeviceTransactionsForAsset(
            walletId: wallet.id,
            asset: assetId,
            fromTimestamp: store.transactionsForAssetTimestamp(assetId: assetId.identifier)
        )
        if response.transactions.isEmpty {
            return
        }

        try await prefetchAssets(walletId: wallet.walletId, transactions: response.transactions)
        try transactionStore.addTransactions(walletId: wallet.walletId, transactions: response.transactions)
        try addressStore.addAddressNames(response.addressNames)

        store.setTransactionsForAssetTimestamp(assetId: assetId.identifier, value: newTimestamp)
    }

    public func addTransaction(walletId: WalletId, transaction: Transaction) throws {
        try transactionStore.addTransactions(walletId: walletId, transactions: [transaction])
    }

    public func getTransaction(walletId: WalletId, transactionId: String) throws -> TransactionExtended {
        try transactionStore.getTransaction(walletId: walletId, transactionId: transactionId)
    }

    private func prefetchAssets(walletId: WalletId, transactions: [Transaction]) async throws {
        let assetIds = transactions.map { $0.assetIds }.flatMap { $0 }
        if assetIds.isEmpty {
            return
        }
        let newAssets = try await assetsService.prefetchAssets(assetIds: assetIds)
        try assetsService.addBalancesIfMissing(walletId: walletId, assetIds: newAssets)
    }
}
