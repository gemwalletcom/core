// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum PushNotification: Equatable, Sendable {
    case transaction(walletId: WalletId, AssetId, transaction: Transaction)
    case asset(AssetId)
    case priceAlert(AssetId)
    case buyAsset(AssetId, amount: Int?)
    case swapAsset(AssetId, AssetId?)
    case support
    case rewards
    case stake(WalletId, AssetId)
    case test
    case unknown

    public init(from userInfo: [AnyHashable: Any]) throws {
        guard
            let typeString = userInfo["type"] as? String,
            let type = PushNotificationTypes(rawValue: typeString),
            let dataDict = userInfo["data"] as? [AnyHashable: Any] else {
            self = .unknown
            return
        }

        let data = try JSONSerialization.data(withJSONObject: dataDict, options: [])
        let decoder = JSONDateDecoder.standard
        switch type {
        case .transaction:
            let transaction = try decoder.decode(PushNotificationTransaction.self, from: data)
            let assetId = try AssetId(id: transaction.assetId)
            let walletId = try WalletId.from(id: transaction.walletId)
            self = .transaction(walletId: walletId, assetId, transaction: transaction.transaction)
        case .asset:
            let asset = try decoder.decode(PushNotificationAsset.self, from: data)
            self = .asset(try AssetId(id: asset.assetId))
        case .priceAlert:
            let asset = try decoder.decode(PushNotificationAsset.self, from: data)
            self = .priceAlert(try AssetId(id: asset.assetId))
        case .buyAsset:
            // TODO: parse amount from push notification data
            let asset = try decoder.decode(PushNotificationAsset.self, from: data)
            self = .buyAsset(try AssetId(id: asset.assetId), amount: nil)
        case .swapAsset:
            let swapAsset = try decoder.decode(PushNotificationSwapAsset.self, from: data)
            let fromAssetId = try AssetId(id: swapAsset.fromAssetId)
            let toAssetId = try AssetId(id: swapAsset.toAssetId)
            self = .swapAsset(fromAssetId, toAssetId)
        case .support:
            self = .support
        case .rewards:
            self = .rewards
        case .stake:
            let value = try decoder.decode(PushNotificationStake.self, from: data)
            self = .stake(value.walletId, value.assetId)
        case .test:
            self = .test
        }
    }
}
