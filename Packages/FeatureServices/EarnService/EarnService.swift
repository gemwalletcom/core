// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Primitives
import Store

public protocol EarnDataProvidable: Sendable {
    func getEarnData(assetId: AssetId, address: String, value: String, earnType: EarnType) async throws -> ContractCallData
}

public struct EarnService: Sendable {
    private let store: StakeStore
    private let gatewayService: GatewayService

    public init(store: StakeStore, gatewayService: GatewayService) {
        self.store = store
        self.gatewayService = gatewayService
    }

    public func update(walletId: WalletId, assetId: AssetId, address: String) async throws {
        let apr = try store.getEarnApr(assetId: assetId) ?? 0
        let providers = try await gatewayService.earnProviders(assetId: assetId)
            .map { DelegationValidator(chain: $0.chain, id: $0.id, name: $0.name, isActive: $0.isActive, commission: $0.commission, apr: apr, providerType: $0.providerType) }
        try store.updateValidators(providers)

        let positions = try await gatewayService.earnPositions(address: address, assetId: assetId)
        try updatePositions(walletId: walletId, assetId: assetId, positions: positions)
    }

    private func updatePositions(walletId: WalletId, assetId: AssetId, positions: [DelegationBase]) throws {
        let existingIds = try store
            .getDelegations(walletId: walletId, assetId: assetId, providerType: .earn)
            .map(\.id)
            .asSet()
        let deleteIds = existingIds.subtracting(positions.map(\.id).asSet()).asArray()

        try store.updateAndDelete(walletId: walletId, delegations: positions, deleteIds: deleteIds)
    }
}

// MARK: - EarnDataProvidable

extension EarnService: EarnDataProvidable {
    public func getEarnData(assetId: AssetId, address: String, value: String, earnType: EarnType) async throws -> ContractCallData {
        try await gatewayService.getEarnData(assetId: assetId, address: address, value: value, earnType: earnType)
    }
}
