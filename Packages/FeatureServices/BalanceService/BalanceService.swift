// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import BigInt
import Store
import ChainService
import Formatters
import AssetsService

public struct BalanceService: Sendable {
    private let balanceStore: BalanceStore
    private let assetsService: AssetsService
    private let fetcher: BalanceFetcher
    private let formatter = ValueFormatter(style: .full)

    public init(
        balanceStore: BalanceStore,
        assetsService: AssetsService,
        chainServiceFactory: any ChainServiceFactorable
    ) {
        self.balanceStore = balanceStore
        self.assetsService = assetsService
        self.fetcher = BalanceFetcher(chainServiceFactory: chainServiceFactory)
    }
}

// MARK: - Asset Manage

extension BalanceService {
    public func hideAsset(walletId: WalletId, assetId: AssetId) throws {
        try balanceStore.setIsEnabled(walletId: walletId, assetIds: [assetId], value: false)
    }

    public func setPinned(_ isPinned: Bool, walletId: WalletId, assetId: AssetId) throws {
        try balanceStore.pinAsset(walletId: walletId, assetId: assetId, value: isPinned)
    }
}

// MARK: - BalanceUpdater

extension BalanceService: BalanceUpdater {
    public func updateBalance(for wallet: Wallet, assetIds: [AssetId]) async {
        let walletId = wallet.walletId

        await withTaskGroup(of: Void.self) { group in
            for account in wallet.accounts {
                let chain = account.chain
                let address = account.address
                let ids = assetIds.filter { $0.identifier.hasPrefix(chain.rawValue) }
                let tokenIds = ids.filter { $0.identifier != chain.id }

                guard !ids.isEmpty else { continue }

                // coin balance
                if ids.contains(chain.assetId) {
                    group.addTask {
                        await updateCoinBalance(walletId: walletId, asset: chain.assetId, address: address)
                    }
                    group.addTask {
                        await updateCoinStakeBalance(walletId: walletId, asset: chain.assetId, address: address)
                    }
                    group.addTask {
                        await updateEarnBalance(walletId: walletId, chain: chain, address: address)
                    }
                }

                // token balance
                if !tokenIds.isEmpty {
                    group.addTask {
                        await updateTokenBalances(walletId: walletId, chain: chain, tokenIds: tokenIds, address: address)
                    }
                }
            }

            for await _ in group { }
        }
    }
}

// MARK: - Balances

extension BalanceService {
    public func getBalance(walletId: WalletId, assetId: AssetId) throws -> Balance? {
        try balanceStore.getBalance(walletId: walletId, assetId: assetId)
    }

    public func addAssetsBalancesIfMissing(assetIds: [AssetId], wallet: Wallet, isEnabled: Bool?) throws {
        let walletId = wallet.walletId
        let balancesAssetIds = try balanceStore
            .getBalances(walletId: walletId, assetIds: assetIds)
            .map { $0.assetId }
        let missingBalancesAssetIds = assetIds.asSet().subtracting(balancesAssetIds)

        try addBalance(
            walletId: walletId,
            balances: missingBalancesAssetIds.map {
                AddBalance(
                    assetId: $0,
                    isEnabled: isEnabled ?? false
                )
            }
        )
    }

    // MARK: - Private Helpers

    private func addBalance(walletId: WalletId, balances: [AddBalance]) throws {
        try balanceStore.addBalance(balances, for: walletId)
    }

    @discardableResult
    private func updateCoinBalance(walletId: WalletId, asset: AssetId, address: String) async -> [AssetBalanceChange] {
        let chain = asset.chain
        return await updateBalanceAsync(
            walletId: walletId,
            chain: chain,
            fetchBalance: { [try await fetcher.getCoinBalance(chain: chain, address: address).coinChange] },
            mapBalance: { $0 }
        )
    }

    @discardableResult
    private func updateCoinStakeBalance(walletId: WalletId, asset: AssetId, address: String) async -> [AssetBalanceChange] {
        let chain = asset.chain
        return await updateBalanceAsync(
            walletId: walletId,
            chain: chain,
            fetchBalance: { [try await fetcher.getCoinStakeBalance(chain: chain, address: address)?.stakeChange] },
            mapBalance: { $0 }
        )
    }

    @discardableResult
    private func updateEarnBalance(walletId: WalletId, chain: Chain, address: String) async -> [AssetBalanceChange] {
        await updateBalanceAsync(
            walletId: walletId,
            chain: chain,
            fetchBalance: { try await fetcher.getEarnBalance(chain: chain, address: address) },
            mapBalance: { $0.earnChange }
        )
    }

    @discardableResult
    private func updateTokenBalances(walletId: WalletId, chain: Chain, tokenIds: [AssetId], address: String) async -> [AssetBalanceChange] {
        await updateBalanceAsync(
            walletId: walletId,
            chain: chain,
            fetchBalance: { try await fetcher.getTokenBalance(chain: chain, address: address, tokenIds: tokenIds.ids)
            },
            mapBalance: { $0.tokenChange }
        )
    }

    private func updateBalanceAsync<T: Sendable>(
        walletId: WalletId,
        chain: Chain,
        fetchBalance: () async throws -> [T],
        mapBalance: (T) -> AssetBalanceChange?
    ) async -> [AssetBalanceChange] {
        do {
            let balances = try await fetchBalance().compactMap { mapBalance($0) }
            try await addAssetsIfNeeded(balances: balances)
            try storeBalances(balances: balances, walletId: walletId)
            return balances
        } catch {
            debugLog("update balance error: chain: \(chain.id): \(error.localizedDescription)")
            return []
        }
    }

    private func createUpdateBalanceType(asset: Asset, change: AssetBalanceChange) throws -> UpdateBalanceType {
        let decimals = asset.decimals.asInt
        switch change.type {
        case .coin(let available, let reserved, let pendingUnconfirmed):
            return .coin(UpdateCoinBalance(
                available: try balanceValue(available, decimals: decimals),
                reserved: try balanceValue(reserved, decimals: decimals),
                pendingUnconfirmed: try balanceValue(pendingUnconfirmed, decimals: decimals)
            ))
        case .token(let available):
            return .token(UpdateTokenBalance(available: try balanceValue(available, decimals: decimals)))
        case .stake(let staked, let pending, let rewards, _, let locked, let frozen, let metadata):
            return .stake(UpdateStakeBalance(
                staked: try balanceValue(staked, decimals: decimals),
                pending: try balanceValue(pending, decimals: decimals),
                frozen: try balanceValue(frozen, decimals: decimals),
                locked: try balanceValue(locked, decimals: decimals),
                rewards: try balanceValue(rewards, decimals: decimals),
                metadata: metadata
            ))
        case .earn(let earn):
            return .earn(UpdateEarnBalance(balance: try balanceValue(earn, decimals: decimals)))
        }
    }

    private func balanceValue(_ value: BigInt, decimals: Int) throws -> UpdateBalanceValue {
        UpdateBalanceValue(value: value.description, amount: try formatter.double(from: value, decimals: decimals))
    }

    private func storeBalances(balances: [AssetBalanceChange], walletId: WalletId) throws {
        for balance in balances {
            debugLog("update balance: \(balance.assetId.identifier): \(balance.type)")
        }
        let assets = try assetsService.getAssets(for: balances.map { $0.assetId })
        let updates = createBalanceUpdate(assets: assets, balances: balances)

        try updateBalances(updates, walletId: walletId)
    }

    private func createBalanceUpdate(assets: [Asset], balances: [AssetBalanceChange]) -> [UpdateBalance] {
        let assets = assets.toMap { $0.id.identifier }
        return balances.compactMap { balance in
            guard
                let asset = assets[balance.assetId.identifier],
                let update = try? createUpdateBalanceType(asset: asset, change: balance)
            else {
                return .none
            }
            return UpdateBalance(
                assetId: balance.assetId,
                type: update,
                updatedAt: .now,
                isActive: balance.isActive
            )
        }
    }

    private func updateBalances(_ balances: [UpdateBalance], walletId: WalletId) throws {
        let assetIds = balances.map { $0.assetId }
        let existBalances = try balanceStore.getBalances(walletId: walletId, assetIds: assetIds)
        let missingBalances = assetIds.asSet().subtracting(existBalances.map { $0.assetId })
        let addBalances: [AddBalance] = missingBalances.map {
            AddBalance(assetId: $0, isEnabled: false)
        }

        try balanceStore.addBalance(addBalances, for: walletId)
        try balanceStore.updateBalances(balances, for: walletId)
    }
    
    private func addAssetsIfNeeded(balances: [AssetBalanceChange]) async throws {
        let assetIds = balances.map { $0.assetId }
        let existAssets = try assetsService.getAssets(for: assetIds)
        let missingIds = assetIds.asSet().subtracting(existAssets.map { $0.id }).asArray()
        if missingIds.isNotEmpty {
            try await assetsService.addAssets(assetIds: missingIds)
        }
    }
}
