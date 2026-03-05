// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftHTTPClient
import Primitives

public enum GemDeviceAPI: TargetType {
    case getDevice
    case addDevice(device: Device)
    case updateDevice(device: Device)
    case deleteDevice
    case isDeviceRegistered
    case migrateDevice(request: MigrateDeviceIdRequest)

    case getSubscriptions
    case addSubscriptions(subscriptions: [WalletSubscription])
    case deleteSubscriptions(subscriptions: [WalletSubscriptionChains])

    case getPriceAlerts(assetId: String?)
    case addPriceAlerts(priceAlerts: [PriceAlert])
    case deletePriceAlerts(priceAlerts: [PriceAlert])

    case getTransactions(walletId: String, assetId: String?, fromTimestamp: Int)
    case getAssetsList(walletId: String, fromTimestamp: Int)
    case getDeviceNFTAssets(walletId: String)

    case reportNft(report: ReportNft)
    case scanTransaction(payload: ScanTransactionPayload)

    case getAuthNonce
    case getDeviceToken

    case getDeviceRewards(walletId: String)
    case getDeviceRewardsEvents(walletId: String)
    case getDeviceRedemptionOption(code: String)
    case createDeviceReferral(walletId: String, request: AuthenticatedRequest<ReferralCode>)
    case useDeviceReferralCode(walletId: String, request: AuthenticatedRequest<ReferralCode>)
    case redeemDeviceRewards(walletId: String, request: AuthenticatedRequest<RedemptionRequest>)

    case getNotifications(fromTimestamp: Int)
    case markNotificationsRead

    case getFiatAssets(FiatQuoteType)
    case getFiatQuotes(walletId: String, type: FiatQuoteType, assetId: AssetId, request: FiatQuoteRequest)
    case getFiatQuoteUrl(walletId: String, quoteId: String)

    case getNameRecord(name: String, chain: String)

    case getPortfolioAssets(period: ChartPeriod, request: PortfolioAssetsRequest)

    public var baseUrl: URL {
        Constants.apiURL
    }

    public var method: HTTPMethod {
        switch self {
        case .getDevice,
            .getSubscriptions,
            .getTransactions,
            .getAssetsList,
            .getPriceAlerts,
            .getDeviceNFTAssets,
            .getAuthNonce,
            .getDeviceToken,
            .getDeviceRewards,
            .getDeviceRewardsEvents,
            .getDeviceRedemptionOption,
            .getNotifications,
            .isDeviceRegistered,
            .getFiatAssets,
            .getFiatQuotes,
            .getFiatQuoteUrl,
            .getNameRecord:
            return .GET
        case .addDevice,
            .addSubscriptions,
            .addPriceAlerts,
            .scanTransaction,
            .reportNft,
            .migrateDevice,
            .createDeviceReferral,
            .useDeviceReferralCode,
            .redeemDeviceRewards,
            .markNotificationsRead,
            .getPortfolioAssets:
            return .POST
        case .updateDevice:
            return .PUT
        case .deleteDevice,
            .deleteSubscriptions,
            .deletePriceAlerts:
            return .DELETE
        }
    }

    public var path: String {
        switch self {
        case .addDevice,
            .getDevice,
            .deleteDevice,
            .updateDevice:
            return "/v2/devices"
        case .isDeviceRegistered:
            return "/v2/devices/is_registered"
        case .migrateDevice:
            return "/v2/devices/migrate"
        case .getSubscriptions,
            .addSubscriptions,
            .deleteSubscriptions:
            return "/v2/devices/subscriptions"
        case .getPriceAlerts,
            .addPriceAlerts,
            .deletePriceAlerts:
            return "/v2/devices/price_alerts"
        case .getTransactions(_, let assetId, let fromTimestamp):
            var path = "/v2/devices/transactions?from_timestamp=\(fromTimestamp)"
            if let assetId {
                path += "&asset_id=\(assetId)"
            }
            return path
        case .getAssetsList(_, let fromTimestamp):
            return "/v2/devices/assets?from_timestamp=\(fromTimestamp)"
        case .getDeviceNFTAssets:
            return "/v2/devices/nft_assets"
        case .reportNft:
            return "/v2/devices/nft/report"
        case .scanTransaction:
            return "/v2/devices/scan/transaction"
        case .getAuthNonce:
            return "/v2/devices/auth/nonce"
        case .getDeviceToken:
            return "/v2/devices/token"
        case .getDeviceRewards:
            return "/v2/devices/rewards"
        case .getDeviceRewardsEvents:
            return "/v2/devices/rewards/events"
        case .getDeviceRedemptionOption(let code):
            return "/v2/devices/rewards/redemptions/\(code)"
        case .createDeviceReferral:
            return "/v2/devices/rewards/referrals/create"
        case .useDeviceReferralCode:
            return "/v2/devices/rewards/referrals/use"
        case .redeemDeviceRewards:
            return "/v2/devices/rewards/redeem"
        case .getNotifications(let fromTimestamp):
            return "/v2/devices/notifications?from_timestamp=\(fromTimestamp)"
        case .markNotificationsRead:
            return "/v2/devices/notifications/read"
        case .getFiatAssets(let type):
            return "/v2/devices/fiat/assets/\(type.rawValue)"
        case .getFiatQuotes(_, let type, let assetId, _):
            return "/v2/devices/fiat/quotes/\(type.rawValue)/\(assetId.identifier)"
        case .getFiatQuoteUrl(_, let quoteId):
            return "/v2/devices/fiat/quotes/\(quoteId)/url"
        case .getNameRecord(let name, let chain):
            return "/v2/devices/name/resolve/\(name)?chain=\(chain)"
        case .getPortfolioAssets(let period, _):
            return "/v2/devices/portfolio/assets?period=\(period.rawValue)"
        }
    }

    public var headers: [String: String] {
        [:]
    }

    public var walletId: String? {
        switch self {
        case .getTransactions(let walletId, _, _),
            .getAssetsList(let walletId, _),
            .getDeviceNFTAssets(let walletId),
            .getDeviceRewards(let walletId),
            .getDeviceRewardsEvents(let walletId),
            .createDeviceReferral(let walletId, _),
            .useDeviceReferralCode(let walletId, _),
            .redeemDeviceRewards(let walletId, _),
            .getFiatQuotes(let walletId, _, _, _),
            .getFiatQuoteUrl(let walletId, _):
            return walletId
        default:
            return nil
        }
    }

    public var data: RequestData {
        switch self {
        case .getDevice,
            .deleteDevice,
            .getSubscriptions,
            .getAssetsList,
            .getDeviceNFTAssets,
            .getAuthNonce,
            .getDeviceToken,
            .getDeviceRewards,
            .getDeviceRewardsEvents,
            .getDeviceRedemptionOption,
            .getNotifications,
            .markNotificationsRead,
            .getTransactions,
            .isDeviceRegistered,
            .getFiatAssets,
            .getFiatQuoteUrl,
            .getNameRecord:
            return .plain
        case .getPriceAlerts(let assetId):
            let params: [String: Any] = [
                "asset_id": assetId,
            ].compactMapValues { $0 }
            return .params(params)
        case .getFiatQuotes(_, _, _, let request):
            let params: [String: Any] = [
                "amount": request.amount,
                "currency": request.currency
            ]
            return .params(params)
        case .addDevice(let device),
            .updateDevice(let device):
            return .encodable(device)
        case .migrateDevice(let request):
            return .encodable(request)
        case .addSubscriptions(let subscriptions):
            return .encodable(subscriptions)
        case .deleteSubscriptions(let subscriptions):
            return .encodable(subscriptions)
        case .addPriceAlerts(let priceAlerts),
            .deletePriceAlerts(let priceAlerts):
            return .encodable(priceAlerts)
        case .scanTransaction(let payload):
            return .encodable(payload)
        case .reportNft(let report):
            return .encodable(report)
        case .createDeviceReferral(_, let request):
            return .encodable(request)
        case .useDeviceReferralCode(_, let request):
            return .encodable(request)
        case .redeemDeviceRewards(_, let request):
            return .encodable(request)
        case .getPortfolioAssets(_, let request):
            return .encodable(request)
        }
    }
}
