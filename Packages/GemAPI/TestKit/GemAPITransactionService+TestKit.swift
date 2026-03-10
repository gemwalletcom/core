// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public final class GemAPITransactionServiceMock: GemAPITransactionService, @unchecked Sendable {
    public init() {}

    public func getDeviceTransactions(walletId: String, fromTimestamp: Int) async throws -> TransactionsResponse {
        TransactionsResponse(transactions: [], addressNames: [])
    }

    public func getDeviceTransactionsForAsset(walletId: String, asset: AssetId, fromTimestamp: Int) async throws -> TransactionsResponse {
        TransactionsResponse(transactions: [], addressNames: [])
    }
}
