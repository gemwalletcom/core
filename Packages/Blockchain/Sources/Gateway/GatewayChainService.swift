// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

internal import GemstonePrimitives

struct GatewayChainService: Sendable {

    private let chain: Chain
    let gateway: GatewayService

    init(
        chain: Chain,
        gateway: GatewayService,
    ) {
        self.chain = chain
        self.gateway = gateway
    }
}

// MARK: - ChainBalanceable

extension GatewayChainService: ChainBalanceable {
    public func coinBalance(for address: String) async throws -> AssetBalance {
        try await gateway.coinBalance(chain: chain, address: address)
    }

    public func tokenBalance(for address: String, tokenIds: [AssetId]) async throws -> [AssetBalance] {
        try await gateway.tokenBalance(chain: chain, address: address, tokenIds: tokenIds)
    }

    public func getStakeBalance(for address: String) async throws -> AssetBalance? {
        try await gateway.getStakeBalance(chain: chain, address: address)
    }

    public func getEarnBalance(for address: String) async throws -> [AssetBalance] {
        try await gateway.getEarnBalance(chain: chain, address: address)
    }
}

// MARK: - ChainFeeRateFetchable

extension GatewayChainService: ChainFeeRateFetchable {
    public func feeRates(type: TransferDataType) async throws -> [FeeRate] {
        try await gateway.feeRates(chain: chain, input: type)
    }
}

// MARK: - ChainBroadcastable

extension GatewayChainService: ChainBroadcastable {
    public func broadcast(data: String, options: BroadcastOptions) async throws -> String {
        try await gateway.transactionBroadcast(chain: chain, data: data, options: options)
    }
}

// MARK: - ChainTransactionStateFetchable

extension GatewayChainService: ChainTransactionStateFetchable {
    public func transactionState(for request: TransactionStateRequest) async throws -> TransactionChanges {
        try await gateway.transactionStatus(chain: chain, request: request)
    }
}

// MARK: - ChainTokenable

extension GatewayChainService: ChainTokenable {
    public func getTokenData(tokenId: String) async throws -> Asset {
        try await gateway.tokenData(chain: chain, tokenId: tokenId)
    }

    public func getIsTokenAddress(tokenId: String) async throws -> Bool {
        try await gateway.isTokenAddress(chain: chain, tokenId: tokenId)
    }
}

// MARK: - ChainIDFetchable

extension GatewayChainService: ChainIDFetchable {
    public func getChainID() async throws -> String {
        try await gateway.chainId(chain: chain)
    }
}

// MARK: - ChainLatestBlockFetchable

extension GatewayChainService: ChainLatestBlockFetchable {
    public func getLatestBlock() async throws -> BigInt {
        BigInt(try await gateway.latestBlock(chain: chain))
    }
}

// MARK: - ChainTransactionPreloadable

extension GatewayChainService: ChainTransactionPreloadable {
    public func preload(input: TransactionPreloadInput) async throws -> TransactionLoadMetadata {
        try await gateway.transactionPreload(chain: chain, input: input)
    }
}

// MARK: - ChainTransactionDataLoadable

extension GatewayChainService: ChainTransactionDataLoadable {
    public func load(input: TransactionInput) async throws -> TransactionData {
        try await gateway.transactionLoad(chain: chain, input: input.map())
    }
}

// MARK: - ChainStakable

extension GatewayChainService: ChainStakable {
    public func getValidators(apr: Double) async throws -> [DelegationValidator] {
        try await gateway.validators(chain: chain, apy: apr)
    }

    public func getStakeDelegations(address: String) async throws -> [DelegationBase] {
        try await gateway.delegations(chain: chain, address: address)
    }
}

// MARK: - ChainNodeStatusFetchable

extension GatewayChainService: ChainNodeStatusFetchable {
    public func getNodeStatus(url: String) async throws -> NodeStatus {
        try await gateway.nodeStatus(chain: chain, url: url)
    }
}

extension GatewayChainService: ChainAddressStatusFetchable {
    public func getAddressStatus(address: String) async throws -> [AddressStatus] {
        try await gateway.getAddressStatus(chain: chain, address: address)
    }
}
