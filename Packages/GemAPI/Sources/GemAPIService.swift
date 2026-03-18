// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftHTTPClient
import Primitives

public protocol GemAPIConfigService: Sendable {
    func getConfig() async throws -> ConfigResponse
}

public protocol GemAPIFiatService: Sendable {
    func getQuotes(walletId: String, type: FiatQuoteType, assetId: AssetId, request: FiatQuoteRequest) async throws -> [FiatQuote]
    func getQuoteUrl(walletId: String, quoteId: String) async throws -> FiatQuoteUrl
}

public protocol GemAPIPricesService: Sendable {
    func getPrices(currency: String?, assetIds: [AssetId]) async throws -> [AssetPrice]
}

public protocol GemAPIAssetsListService: Sendable {
    func getDeviceAssets(walletId: String, fromTimestamp: Int) async throws -> [AssetId]
    func getBuyableFiatAssets() async throws -> FiatAssets
    func getSellableFiatAssets() async throws -> FiatAssets
    func getSwapAssets() async throws -> FiatAssets
}

public protocol GemAPIAssetsService: Sendable {
    func getAsset(assetId: AssetId) async throws -> AssetFull
    func getAssets(assetIds: [AssetId]) async throws -> [AssetBasic]
    func getSearchAssets(query: String, chains: [Chain], tags: [AssetTag]) async throws -> [AssetBasic]
}

public protocol GemAPINameService: Sendable {
    func getName(name: String, chain: String) async throws -> NameRecord?
}

public protocol GemAPIAddressNamesService: Sendable {
    func getAddressNames(requests: [ChainAddress]) async throws -> [AddressName]
}

public protocol GemAPIChartService: Sendable {
    func getCharts(assetId: AssetId, period: String) async throws -> Charts
}

public protocol GemAPIDeviceService: Sendable {
    func getDevice() async throws -> Device?
    func addDevice(device: Device) async throws -> Device
    func updateDevice(device: Device) async throws -> Device
    func isDeviceRegistered() async throws -> Bool
    func migrateDevice(request: MigrateDeviceIdRequest) async throws -> Device
    func getNodeAuthToken() async throws -> DeviceToken
}

public protocol GemAPISubscriptionService: Sendable {
    func getSubscriptions() async throws -> [WalletSubscriptionChains]
    func addSubscriptions(subscriptions: [WalletSubscription]) async throws
    func deleteSubscriptions(subscriptions: [WalletSubscriptionChains]) async throws
}

public protocol GemAPITransactionService: Sendable {
    func getDeviceTransactions(walletId: String, fromTimestamp: Int) async throws -> TransactionsResponse
    func getDeviceTransactionsForAsset(walletId: String, asset: AssetId, fromTimestamp: Int) async throws -> TransactionsResponse
}

public protocol GemAPIPriceAlertService: Sendable {
    func getPriceAlerts(assetId: String?) async throws -> [PriceAlert]
    func addPriceAlerts(priceAlerts: [PriceAlert]) async throws
    func deletePriceAlerts(priceAlerts: [PriceAlert]) async throws
}

public protocol GemAPINFTService: Sendable {
    func getDeviceNFTAssets(walletId: String) async throws -> [NFTData]
    func reportNft(report: ReportNft) async throws
}

public protocol GemAPIScanService: Sendable {
    func getScanTransaction(payload: ScanTransactionPayload) async throws -> ScanTransaction
}

public protocol GemAPIMarketService: Sendable {
    func getMarkets() async throws -> Markets
}

public protocol GemAPIAuthService: Sendable {
    func getAuthNonce() async throws -> AuthNonce
}

public protocol GemAPIRewardsService: Sendable {
    func getRewards(walletId: String) async throws -> Rewards
    func createReferral(walletId: String, request: AuthenticatedRequest<ReferralCode>) async throws -> Rewards
    func useReferralCode(walletId: String, request: AuthenticatedRequest<ReferralCode>) async throws
    func getRedemptionOption(code: String) async throws -> RewardRedemptionOption
    func redeem(walletId: String, request: AuthenticatedRequest<RedemptionRequest>) async throws -> RedemptionResult
}

public protocol GemAPISearchService: Sendable {
    func search(query: String, chains: [Chain], tags: [AssetTag]) async throws -> SearchResponse
}

public protocol GemAPIPortfolioService: Sendable {
    func getPortfolioAssets(period: ChartPeriod, request: PortfolioAssetsRequest) async throws -> PortfolioAssets
}

public protocol GemAPINotificationService: Sendable {
    func getNotifications(fromTimestamp: Int) async throws -> [Primitives.InAppNotification]
    func markNotificationsRead() async throws
}

public struct GemAPIService {

    let provider: Provider<GemAPI>
    let deviceProvider: Provider<GemDeviceAPI>
    private let walletRequestPreflight: (@Sendable () async throws -> Void)?

    public static let shared = GemAPIService()
    public static let sharedProvider = Provider<GemAPI>()
    public static let sharedDeviceProvider = Provider<GemDeviceAPI>()

    public init(
        provider: Provider<GemAPI> = Self.sharedProvider,
        deviceProvider: Provider<GemDeviceAPI> = Self.sharedDeviceProvider,
        walletRequestPreflight: (@Sendable () async throws -> Void)? = nil
    ) {
        self.provider = provider
        self.deviceProvider = deviceProvider
        self.walletRequestPreflight = walletRequestPreflight
    }

    private func requestDevice(_ target: GemDeviceAPI) async throws -> Response {
        if target.walletId != nil {
            try await walletRequestPreflight?()
        }
        return try await deviceProvider.request(target)
    }
}

extension GemAPIService: GemAPIFiatService {
    public func getQuotes(walletId: String, type: FiatQuoteType, assetId: AssetId, request: FiatQuoteRequest) async throws -> [FiatQuote] {
        try await requestDevice(.getFiatQuotes(walletId: walletId, type: type, assetId: assetId, request: request))
            .mapResponse(as: FiatQuotes.self)
            .quotes
    }

    public func getQuoteUrl(walletId: String, quoteId: String) async throws -> FiatQuoteUrl {
        try await requestDevice(.getFiatQuoteUrl(walletId: walletId, quoteId: quoteId))
            .mapResponse(as: FiatQuoteUrl.self)
    }
}

extension GemAPIService: GemAPIConfigService {
    public func getConfig() async throws -> ConfigResponse {
        try await provider
            .request(.getConfig)
            .mapResponse(as: ConfigResponse.self)
    }
}

extension GemAPIService: GemAPINameService {
    public func getName(name: String, chain: String) async throws -> NameRecord? {
        try await requestDevice(.getNameRecord(name: name, chain: chain))
            .mapResponse(as: NameRecord?.self)
    }
}

extension GemAPIService: GemAPIAddressNamesService {
    public func getAddressNames(requests: [ChainAddress]) async throws -> [AddressName] {
        try await requestDevice(.getAddressNames(requests: requests))
            .mapResponse(as: [AddressName].self)
    }
}

extension GemAPIService: GemAPIChartService {
    public func getCharts(assetId: AssetId, period: String) async throws -> Charts {
        try await provider
            .request(.getCharts(assetId, period: period))
            .mapResponse(as: Charts.self)
    }
}

extension GemAPIService: GemAPITransactionService {
    public func getDeviceTransactionsForAsset(walletId: String, asset: Primitives.AssetId, fromTimestamp: Int) async throws -> TransactionsResponse {
        try await requestDevice(.getTransactions(walletId: walletId, assetId: asset.identifier, fromTimestamp: fromTimestamp))
            .mapResponse(as: TransactionsResponse.self)
    }

    public func getDeviceTransactions(walletId: String, fromTimestamp: Int) async throws -> TransactionsResponse {
        try await requestDevice(.getTransactions(walletId: walletId, assetId: nil, fromTimestamp: fromTimestamp))
            .mapResponse(as: TransactionsResponse.self)
    }
}

extension GemAPIService: GemAPIAssetsListService {
    public func getDeviceAssets(walletId: String, fromTimestamp: Int) async throws -> [Primitives.AssetId] {
        try await requestDevice(.getAssetsList(walletId: walletId, fromTimestamp: fromTimestamp))
            .mapResponse(as: [String].self)
            .compactMap { try? AssetId(id: $0) }
    }

    public func getBuyableFiatAssets() async throws -> FiatAssets {
        try await requestDevice(.getFiatAssets(.buy))
            .mapResponse(as: FiatAssets.self)
    }

    public func getSellableFiatAssets() async throws -> FiatAssets {
        try await requestDevice(.getFiatAssets(.sell))
            .mapResponse(as: FiatAssets.self)
    }

    public func getSwapAssets() async throws -> FiatAssets {
        try await provider
            .request(.getSwapAssets)
            .mapResponse(as: FiatAssets.self)
    }
}

extension GemAPIService: GemAPIAssetsService {
    public func getAsset(assetId: AssetId) async throws -> AssetFull {
        try await provider
            .request(.getAsset(assetId))
            .mapResponse(as: AssetFull.self)
    }

    public func getAssets(assetIds: [AssetId]) async throws -> [AssetBasic] {
        try await provider
            .request(.getAssets(assetIds))
            .mapResponse(as: [AssetBasic].self)
    }

    public func getSearchAssets(query: String, chains: [Chain], tags: [AssetTag]) async throws -> [AssetBasic] {
        try await provider
            .request(.getSearchAssets(query: query, chains: chains, tags: tags))
            .mapResponse(as: [AssetBasic].self)
    }
}

extension GemAPIService: GemAPIPriceAlertService {
    public func getPriceAlerts(assetId: String?) async throws -> [PriceAlert] {
        try await requestDevice(.getPriceAlerts(assetId: assetId))
            .mapResponse(as: [PriceAlert].self)
    }

    public func addPriceAlerts(priceAlerts: [PriceAlert]) async throws {
        let _ = try await requestDevice(.addPriceAlerts(priceAlerts: priceAlerts))
            .mapResponse(as: Int.self)
    }

    public func deletePriceAlerts(priceAlerts: [PriceAlert]) async throws {
        let _ = try await requestDevice(.deletePriceAlerts(priceAlerts: priceAlerts))
            .mapResponse(as: Int.self)
    }
}

extension GemAPIService: GemAPINFTService {
    public func getDeviceNFTAssets(walletId: String) async throws -> [NFTData] {
        try await requestDevice(.getDeviceNFTAssets(walletId: walletId))
            .mapResponse(as: [NFTData].self)
    }

    public func reportNft(report: ReportNft) async throws {
        _ = try await requestDevice(.reportNft(report: report))
    }
}

extension GemAPIService: GemAPIScanService {
    public func getScanTransaction(payload: ScanTransactionPayload) async throws -> ScanTransaction {
        try await requestDevice(.scanTransaction(payload: payload))
            .mapResponse(as: ScanTransaction.self)
    }
}

extension GemAPIService: GemAPIMarketService {
    public func getMarkets() async throws -> Markets {
        try await provider
            .request(.markets)
            .mapResponse(as: Markets.self)
    }
}

extension GemAPIService: GemAPIPricesService {
    public func getPrices(currency: String?, assetIds: [AssetId]) async throws -> [AssetPrice] {
        try await provider
            .request(.getPrices(AssetPricesRequest(currency: currency, assetIds: assetIds)))
            .mapResponse(as: AssetPrices.self).prices
    }
}

extension GemAPIService: GemAPIAuthService {
    public func getAuthNonce() async throws -> AuthNonce {
        try await requestDevice(.getAuthNonce)
            .mapResponse(as: AuthNonce.self)
    }
}

extension GemAPIService: GemAPIRewardsService {
    public func getRewards(walletId: String) async throws -> Rewards {
        try await requestDevice(.getDeviceRewards(walletId: walletId))
            .mapResponse(as: Rewards.self)
    }

    public func createReferral(walletId: String, request: AuthenticatedRequest<ReferralCode>) async throws -> Rewards {
        try await requestDevice(.createDeviceReferral(walletId: walletId, request: request))
            .mapResponse(as: Rewards.self)
    }

    public func useReferralCode(walletId: String, request: AuthenticatedRequest<ReferralCode>) async throws {
        _ = try await requestDevice(.useDeviceReferralCode(walletId: walletId, request: request))
            .mapResponse(as: [RewardEvent].self)
    }

    public func getRedemptionOption(code: String) async throws -> RewardRedemptionOption {
        try await requestDevice(.getDeviceRedemptionOption(code: code))
            .mapResponse(as: RewardRedemptionOption.self)
    }

    public func redeem(walletId: String, request: AuthenticatedRequest<RedemptionRequest>) async throws -> RedemptionResult {
        try await requestDevice(.redeemDeviceRewards(walletId: walletId, request: request))
            .mapResponse(as: RedemptionResult.self)
    }
}

extension GemAPIService: GemAPISearchService {
    public func search(query: String, chains: [Chain], tags: [AssetTag]) async throws -> SearchResponse {
        try await provider
            .request(.getSearch(query: query, chains: chains, tags: tags))
            .mapResponse(as: SearchResponse.self)
    }
}

extension GemAPIService: GemAPINotificationService {
    public func getNotifications(fromTimestamp: Int) async throws -> [Primitives.InAppNotification] {
        try await requestDevice(.getNotifications(fromTimestamp: fromTimestamp))
            .mapResponse(as: [Primitives.InAppNotification].self)
    }

    public func markNotificationsRead() async throws {
        _ = try await requestDevice(.markNotificationsRead)
    }
}

extension GemAPIService: GemAPIPortfolioService {
    public func getPortfolioAssets(period: ChartPeriod, request: PortfolioAssetsRequest) async throws -> PortfolioAssets {
        try await requestDevice(.getPortfolioAssets(period: period, request: request))
            .mapResponse(as: PortfolioAssets.self)
    }
}

extension SwiftHTTPClient.Response {
    @discardableResult
    public func mapResponse<T: Decodable>(as type: T.Type) throws -> T {
        try self.mapOrError(as: type, asError: ResponseError.self)
    }
}
