// Copyright (c). Gem Wallet. All rights reserved.

import GemAPI
import Store
import Primitives

public struct AddressNameService: Sendable {
    private let addressStore: AddressStore
    private let apiService: any GemAPIAddressNamesService
    
    public init(
        addressStore: AddressStore,
        apiService: any GemAPIAddressNamesService = GemAPIService()
    ) {
        self.addressStore = addressStore
        self.apiService = apiService
    }
    
    public func getAddressName(chain: Chain, address: String) throws -> AddressName? {
        try addressStore.getAddressName(chain: chain, address: address)
    }

    public func getAddressNames(requests: [ChainAddress]) async throws -> [ChainAddress: AddressName] {
        let requests = uniqueRequests(requests)
        guard !requests.isEmpty else {
            return [:]
        }

        let cachedNames = try cachedAddressNames(requests: requests)
        let missingRequests = requests.filter { cachedNames[$0] == nil }
        guard !missingRequests.isEmpty else {
            return cachedNames
        }

        let remoteNames = try await remoteAddressNames(requests: missingRequests)
        return cachedNames.merging(remoteNames) { _, remote in remote }
    }
}

private extension AddressNameService {
    func cachedAddressNames(requests: [ChainAddress]) throws -> [ChainAddress: AddressName] {
        try Dictionary(uniqueKeysWithValues: requests.compactMap { request in
            try addressStore
                .getAddressName(chain: request.chain, address: request.address)
                .map { (request, $0) }
        })
    }

    func remoteAddressNames(requests: [ChainAddress]) async throws -> [ChainAddress: AddressName] {
        let remoteAddressNames: [AddressName]
        do {
            remoteAddressNames = try await apiService.getAddressNames(requests: requests)
        } catch {
            guard !error.isCancelled else {
                throw error
            }
            return [:]
        }

        try addressStore.addAddressNames(remoteAddressNames)
        return Dictionary(uniqueKeysWithValues: remoteAddressNames.map {
            (ChainAddress(chain: $0.chain, address: $0.address), $0)
        })
    }

    func uniqueRequests(_ requests: [ChainAddress]) -> [ChainAddress] {
        var seen = Set<ChainAddress>()
        return requests.filter {
            $0.address.isNotEmpty && seen.insert($0).inserted
        }
    }
}
