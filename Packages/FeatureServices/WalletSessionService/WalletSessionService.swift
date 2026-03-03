// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Store
import Preferences

public struct WalletSessionService: WalletSessionManageable {
    private let walletStore: WalletStore
    private let preferences: ObservablePreferences

    public init(
        walletStore: WalletStore,
        preferences: ObservablePreferences
    ) {
        self.walletStore = walletStore
        self.preferences = preferences
    }

    public var currentWallet: Wallet? {
        guard let currentWalletId else { return nil }
        return wallets.first(where: {$0.walletId == currentWalletId })
    }

    public var currentWalletId: Primitives.WalletId? {
        guard let id = preferences.currentWalletId else { return nil }
        return try? WalletId.from(id: id)
    }

    public func setCurrent(index: Int) -> WalletId? {
        guard let wallet = wallets.first(where: { $0.index == index }) else {
            return nil
        }
        if let currentWallet, currentWallet == wallet {
            return currentWallet.walletId
        }
        preferences.currentWalletId = wallet.id

        return wallet.walletId
    }

    public func setCurrent(walletId: WalletId?) {
        preferences.currentWalletId = walletId?.id
    }

    public func getWallets() throws -> [Wallet] {
        try walletStore.getWallets()
    }
}
