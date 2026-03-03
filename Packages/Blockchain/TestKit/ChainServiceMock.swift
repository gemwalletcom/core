// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Blockchain
import Primitives
import BigInt

public final class ChainServiceMock: ChainServiceable, @unchecked Sendable {
    
    // Injected data
    public var addressStatuses: [AddressStatus] = []
    public var coinBalances: [String: AssetBalance] = [:]
    public var tokenBalances: [String: [AssetBalance]] = [:]
    public var stakeBalance: AssetBalance?
    public var broadcastResponses: [String] = []
    public var fee: Fee = Fee(fee: .zero, gasPriceType: .regular(gasPrice: .zero), gasLimit: .zero)
    public var feeRates: [FeeRate] = []
    public var chainID: String?
    public var latestBlock: BigInt = .zero
    public var validators: [DelegationValidator] = []
    public var delegations: [DelegationBase] = []
    public var inSync: Bool = true
    public var tokenData: [String: Asset] = [:]
    public var transactionData: TransactionData = TransactionData(fee: Fee(fee: .zero, gasPriceType: .regular(gasPrice: .zero), gasLimit: .zero))
    public var transactionPreload: TransactionLoadMetadata = .none
    public var transactionState: TransactionChanges = TransactionChanges(state: .pending, changes: [])
    public var nodeStatus: NodeStatus = NodeStatus(chainId: "1", latestBlockNumber: .zero, latency: .from(duration: 1000))

    public init() {}
}

extension ChainServiceMock {
    public func getAddressStatus(address: String) async throws -> [AddressStatus] {
        addressStatuses
    }
    
    public func coinBalance(for address: String) async throws -> AssetBalance {
        coinBalances[address] ?? .init(assetId: AssetId(chain: .ethereum, tokenId: nil), balance: .zero)
    }
    
    public func tokenBalance(for address: String, tokenIds: [AssetId]) async throws -> [AssetBalance] {
        tokenBalances[address] ?? tokenIds.map { AssetBalance(assetId: $0, balance: .zero) }
    }
    
    public func getStakeBalance(for address: String) async throws -> AssetBalance? {
        stakeBalance
    }

    public func getEarnBalance(for address: String) async throws -> [AssetBalance] {
        []
    }
    
    public func broadcast(data: String, options: BroadcastOptions) async throws -> String {
        broadcastResponses.removeFirst()
    }
    
    public func getFee(asset: Asset, input: FeeInput) async throws -> Fee {
        fee
    }
    
    public func feeRates(type: TransferDataType) async throws -> [FeeRate] {
        feeRates
    }
    
    public func getChainID() async throws -> String {
        chainID ?? ""
    }
    
    public func getLatestBlock() async throws -> BigInt {
        latestBlock
    }
    
    public func getValidators(apr: Double) async throws -> [DelegationValidator] {
        validators
    }
    
    public func getStakeDelegations(address: String) async throws -> [DelegationBase] {
        delegations
    }
    
    public func getInSync() async throws -> Bool {
        inSync
    }
    
    public func getTokenData(tokenId: String) async throws -> Asset {
        tokenData[tokenId] ?? Asset(
            id: AssetId(chain: .ethereum, tokenId: nil),
            name: "Ethereum",
            symbol: "ETH",
            decimals: 18,
            type: .native
        )
    }
    
    public func getIsTokenAddress(tokenId: String) -> Bool {
        tokenData[tokenId] != nil
    }
    
    public func load(input: TransactionInput) async throws -> TransactionData {
        transactionData
    }
    
    public func preload(input: TransactionPreloadInput) async throws -> TransactionLoadMetadata {
        transactionPreload
    }
    
    public func transactionState(for request: TransactionStateRequest) async throws -> TransactionChanges {
        transactionState
    }
    
    public func getNodeStatus(url: String) async throws -> NodeStatus {
        nodeStatus
    }
}
