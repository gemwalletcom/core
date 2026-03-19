// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import ChainService
import Primitives

struct BalanceFetcher: Sendable {
    private let chainServiceFactory: any ChainServiceFactorable
    
    init(chainServiceFactory: any ChainServiceFactorable) {
        self.chainServiceFactory = chainServiceFactory
    }
    
    func getCoinBalance(
        chain: Chain,
        address: String
    ) async throws -> AssetBalance {
        try await chainServiceFactory
            .service(for: chain)
            .coinBalance(for: address)
    }

    func getCoinStakeBalance(
        chain: Chain,
        address: String
    ) async throws -> AssetBalance? {
        try await chainServiceFactory
            .service(for: chain)
            .getStakeBalance(for: address)
    }

    func getEarnBalance(chain: Chain, address: String, tokenIds: [AssetId]) async throws -> [AssetBalance] {
        try await chainServiceFactory
            .service(for: chain)
            .getEarnBalance(for: address, tokenIds: tokenIds)
    }

    func getTokenBalance(
        chain: Chain,
        address: String,
        tokenIds: [String]
    ) async throws -> [AssetBalance] {
        try await chainServiceFactory
            .service(for: chain)
           .tokenBalance(for: address, tokenIds: tokenIds.compactMap { try? AssetId(id: $0) })
    }
}
