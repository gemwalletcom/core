// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import InfoSheet

public enum WalletSheetType: Identifiable, Equatable, Sendable {
    case wallets
    case selectAsset(SelectAssetType)
    case infoSheet(InfoSheetType)
    case transferData(TransferData)
    case perpetualRecipientData(PerpetualRecipientData)
    case setPriceAlert(Asset)
    case addAsset
    case portfolio

    public var id: String {
        switch self {
        case .wallets: "wallets"
        case .selectAsset(let type): "selectAsset-\(type.id)"
        case .infoSheet(let type): "infoSheet-\(type.id)"
        case .transferData(let data): "transferData-\(data.id)"
        case .perpetualRecipientData: "perpetualRecipientData"
        case .setPriceAlert(let asset): "setPriceAlert-\(asset.id.identifier)"
        case .addAsset: "addAsset"
        case .portfolio: "portfolio"
        }
    }
}
