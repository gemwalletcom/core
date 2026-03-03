// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store
import Primitives
import ChainService
import GemAPI
import Blockchain

public struct StakeService: StakeServiceable {
    private let store: StakeStore
    private let addressStore: AddressStore
    private let chainServiceFactory: any ChainServiceFactorable
    private let assetsService: GemAPIStaticService
    
    public init(
        store: StakeStore,
        addressStore: AddressStore,
        chainServiceFactory: any ChainServiceFactorable,
        assetsService: GemAPIStaticService = GemAPIStaticService()
    ) {
        self.store = store
        self.addressStore = addressStore
        self.chainServiceFactory = chainServiceFactory
        self.assetsService = assetsService
    }
    
    public func stakeApr(assetId: AssetId) throws -> Double? {
        try store.getStakeApr(assetId: assetId)
    }
    
    public func update(walletId: WalletId, chain: Chain, address: String) async throws {
        let validators = try store.getValidators(assetId: chain.assetId, providerType: .stake)
        if validators.isEmpty {
            try await updateValidators(chain: chain)
            try await updateDelegations(walletId: walletId, chain: chain, address: address)
        } else {
            try await updateDelegations(walletId: walletId, chain: chain, address: address)
            try await updateValidators(chain: chain)
        }
    }

    public func getValidatorsActive(assetId: AssetId) throws -> [DelegationValidator] {
        try store.getValidatorsActive(assetId: assetId, providerType: .stake)
    }

    public func getValidator(assetId: AssetId, validatorId: String) throws -> DelegationValidator? {
        try store.getValidator(assetId: assetId, validatorId: validatorId)
    }
    
    public func clearDelegations() throws {
        try store.clearDelegations()
    }
    
    public func clearValidators() throws {
        try store.clearValidators()
    }
}

// MARK: - Private

extension StakeService {
    private func updateValidators(chain: Chain) async throws {
        let apr = try stakeApr(assetId: chain.assetId) ?? 0

        async let getValidators = chainServiceFactory.service(for: chain).getValidators(apr: apr)
        async let getValidatorsList = assetsService.getValidators(chain: chain)

        let (validators, validatorsList) = try await (
            getValidators,
            getValidatorsList.toMap { $0.id }
        )

        let updateValidators = validators.map {
            let name = $0.name.isEmpty ? validatorsList[$0.id]?.name ?? .empty : $0.name
            return DelegationValidator(
                chain: $0.chain,
                id: $0.id,
                name: name,
                isActive: $0.isActive,
                commission: $0.commission,
                apr: $0.apr,
                providerType: .stake
            )
        }
        try store.updateValidators(updateValidators)
        
        let addressNames = updateValidators.map { AddressName(chain: $0.chain, address: $0.id, name: $0.name, type: .validator) }
        try addressStore.addAddressNames(addressNames)
    }

    private func updateDelegations(walletId: WalletId, chain: Chain, address: String) async throws {
        let delegations = try await getDelegations(chain: chain, address: address)
        let existingDelegationsIds = try store.getDelegations(walletId: walletId, assetId: chain.assetId, providerType: .stake).map { $0.id }.asSet()
        let delegationsIds = delegations.map { $0.id }.asSet()
        let deleteDelegationsIds = existingDelegationsIds.subtracting(delegationsIds).asArray()

        // validators
        let validatorsIds = try store.getValidators(assetId: chain.assetId, providerType: .stake).map { $0.id }.asSet()
        let delegationsValidatorIds = delegations.map { $0.validatorId }.asSet()
        let missingValidatorIds = delegationsValidatorIds.subtracting(validatorsIds)

        //TODO: Might need to fetch in the future.
        if !missingValidatorIds.isEmpty {
            debugLog("missingValidatorIds \(missingValidatorIds)")
        }
        let updateDelegations = delegations.filter { validatorsIds.contains($0.validatorId) }

        try store.updateAndDelete(walletId: walletId, delegations: updateDelegations, deleteIds: deleteDelegationsIds)
    }

    private func getDelegations(chain: Chain, address: String) async throws -> [DelegationBase] {
        let service = chainServiceFactory.service(for: chain)
        return try await service.getStakeDelegations(address: address)
    }
}
