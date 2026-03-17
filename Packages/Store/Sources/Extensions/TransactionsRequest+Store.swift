// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension TransactionsRequest {
    static func assetScene(walletId: WalletId, assetId: AssetId, limit: Int = 25) -> TransactionsRequest {
        TransactionsRequest(
            walletId: walletId,
            type: .asset(assetId: assetId),
            limit: limit
        )
    }

    static func perpetualScene(walletId: WalletId, assetId: AssetId, limit: Int = 50) -> TransactionsRequest {
        TransactionsRequest(
            walletId: walletId,
            type: .asset(assetId: assetId),
            filters: [.types([
                TransactionType.perpetualOpenPosition.rawValue,
                TransactionType.perpetualClosePosition.rawValue
            ])],
            limit: limit
        )
    }
}
