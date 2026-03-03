// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import NativeProviderService
import Primitives

internal import GemstonePrimitives

public actor GatewayService: Sendable {
    let gateway: GemGateway

    public init(
        provider: NativeProvider
    ) {
        self.gateway = GemGateway(
            provider: provider,
            preferences: GemstonePreferences(namespace: "gateway"),
            securePreferences: GemstoneSecurePreferences(namespace: "gateway"),
            apiUrl: Constants.apiURL.absoluteString
        )
    }
}

extension GatewayService: GemGatewayEstimateFee {
    public func getFee(chain: Gemstone.Chain, input: Gemstone.GemTransactionLoadInput) async throws -> Gemstone.GemTransactionLoadFee? {
        try await EstimateFeeService().getFee(chain: chain, input: input)
    }
    
    public func getFeeData(chain: Gemstone.Chain, input: GemTransactionLoadInput) async throws -> String? {
        try await EstimateFeeService().getFeeData(chain: chain, input: input)
    }
}

// MARK: - Balances

extension GatewayService {
    public func coinBalance(chain: Primitives.Chain, address: String) async throws -> AssetBalance {
        try await gateway.getBalanceCoin(chain: chain.rawValue, address: address).map()
    }

    public func tokenBalance(chain: Primitives.Chain, address: String, tokenIds: [Primitives.AssetId]) async throws -> [AssetBalance] {
        try await gateway
            .getBalanceTokens(chain: chain.rawValue, address: address, tokenIds: tokenIds.compactMap(\.tokenId))
            .map { try $0.map() }
    }

    public func getStakeBalance(chain: Primitives.Chain, address: String) async throws -> AssetBalance? {
        try await gateway.getBalanceStaking(chain: chain.rawValue, address: address)?.map()
    }

    public func getEarnBalance(chain: Primitives.Chain, address: String) async throws -> [AssetBalance] {
        try await gateway.getBalanceEarn(chain: chain.rawValue, address: address)
            .map { try $0.map() }
    }
}

// MARK: - Transactions

extension GatewayService {
    public func transactionBroadcast(chain: Primitives.Chain, data: String, options: Primitives.BroadcastOptions = Primitives.BroadcastOptions(skipPreflight: false)) async throws -> String {
        try await gateway.transactionBroadcast(chain: chain.rawValue, data: data, options: options.map())
    }
    
    public func transactionStatus(chain: Primitives.Chain, request: TransactionStateRequest) async throws -> TransactionChanges {
        let update = try await gateway.getTransactionStatus(chain: chain.rawValue, request: request.map())
        let changes: [Primitives.TransactionChange] = try update.changes.compactMap {
            switch $0 {
            case .hashChange(old: let old, new: let new):
                return .hashChange(old: old, new: new)
            case .metadata(let metadata):
                guard let value = metadata.mapToAnyCodableValue() else { return nil }
                return .metadata(value)
            case .blockNumber(let number):
                return .blockNumber(try Int.from(string: number))
            case .networkFee(let fee):
                return .networkFee(try BigInt.from(string: fee))
            }
        }
        return TransactionChanges(
            state: update.state.map(),
            changes: changes
        )
    }
}

// MARK: - Account

extension GatewayService {
    func utxos(chain: Primitives.Chain, address: String) async throws -> [UTXO] {
        try await gateway.getUtxos(chain: chain.rawValue, address: address).map {
            try $0.map()
        }
    }
}

// TransactionPreload

// MARK: - State

extension GatewayService {
    public func chainId(chain: Primitives.Chain) async throws -> String {
        try await gateway.getChainId(chain: chain.rawValue)
    }
    
    public func latestBlock(chain: Primitives.Chain) async throws -> BigInt {
        try await gateway.getBlockNumber(chain: chain.rawValue).asBigInt
    }
    
    public func feeRates(chain: Primitives.Chain, input: TransferDataType) async throws -> [FeeRate] {
        try await gateway.getFeeRates(chain: chain.rawValue, input: input.map()).map { try $0.map() }
    }
    
    public func nodeStatus(chain: Primitives.Chain, url: String) async throws -> Primitives.NodeStatus {
        try await gateway.getNodeStatus(chain: chain.rawValue, url: url).map()
    }
}

// MARK: - Token

extension GatewayService {
    public func tokenData(chain: Primitives.Chain, tokenId: String) async throws -> Asset {
        try await gateway.getTokenData(chain: chain.rawValue, tokenId: tokenId).map()
    }
    
    public func isTokenAddress(chain: Primitives.Chain, tokenId: String) async throws -> Bool {
        try await gateway.getIsTokenAddress(chain: chain.rawValue, tokenId: tokenId)
    }
}

// MARK: - Transaction Preload

extension GatewayService {
    public func transactionPreload(chain: Primitives.Chain, input: TransactionPreloadInput) async throws -> TransactionLoadMetadata {
        try await gateway.getTransactionPreload(chain: chain.rawValue, input: input.map()).map()
    }

    public func transactionScan(chain: Primitives.Chain, input: TransactionPreloadInput) async throws -> Primitives.ScanTransaction? {
        try await gateway.getTransactionScan(chain: chain.rawValue, input: input.map())?.map()
    }

    public func transactionLoad(chain: Primitives.Chain, input: GemTransactionLoadInput) async throws -> TransactionData {
        try await gateway.getTransactionLoad(chain: chain.rawValue, input: input, provider: self).map()
    }
}

// MARK: - Staking

extension GatewayService {
    public func validators(chain: Primitives.Chain, apy: Double) async throws -> [DelegationValidator] {
        try await gateway.getStakingValidators(chain: chain.rawValue, apy: apy)
            .map { try $0.map() }
    }

    public func delegations(chain: Primitives.Chain, address: String) async throws -> [DelegationBase] {
        try await gateway.getStakingDelegations(chain: chain.rawValue, address: address)
            .map { try $0.map() }
    }
}

// MARK: - Earn

extension GatewayService {
    public func earnProviders(assetId: Primitives.AssetId) -> [DelegationValidator] {
        gateway.getEarnProviders(assetId: assetId.identifier).compactMap { try? $0.map() }
    }

    public func earnPositions(chain: Primitives.Chain, address: String, assetIds: [Primitives.AssetId]) async throws -> [DelegationBase] {
        try await gateway.getEarnPositions(chain: chain.rawValue, address: address, assetIds: assetIds.ids)
            .map { try $0.map() }
    }

    public func getEarnData(
        assetId: Primitives.AssetId,
        address: String,
        value: String,
        earnType: EarnType
    ) async throws -> ContractCallData {
        try await gateway.getEarnData(assetId: assetId.identifier, address: address, value: value, earnType: earnType.map())
            .map()
    }
}

// MARK: - Perpetual

extension GatewayService {
    public func getPositions(chain: Primitives.Chain, address: String) async throws -> PerpetualPositionsSummary {
        try await gateway.getPositions(chain: chain.rawValue, address: address).map()
    }
    
    public func getPerpetualsData(chain: Primitives.Chain) async throws -> [PerpetualData] {
        try await gateway.getPerpetualsData(chain: chain.rawValue).map {
            try $0.map()
        }
    }
    
    public func getPerpetualCandlesticks(chain: Primitives.Chain, symbol: String, period: ChartPeriod) async throws -> [ChartCandleStick] {
        try await gateway.getPerpetualCandlesticks(chain: chain.rawValue, symbol: symbol, period: period.rawValue).map {
            $0.map()
        }
    }

    public func getPerpetualPortfolio(chain: Primitives.Chain, address: String) async throws -> PerpetualPortfolio {
        try await gateway.getPerpetualPortfolio(chain: chain.rawValue, address: address).map()
    }
}

extension GatewayService {
    public func getAddressStatus(chain: Primitives.Chain, address: String) async throws -> [Primitives.AddressStatus] {
        try await gateway.getAddressStatus(chain: chain.rawValue, address: address).map { $0.map() }
    }
}
