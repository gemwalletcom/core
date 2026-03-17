// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Store
import Primitives

struct TransactionsRequestTests {

    @Test
    func assetScene() {
        let walletId = WalletId.multicoin(address: "wallet")
        let assetId = AssetId(chain: .ethereum)

        #expect(
            TransactionsRequest.assetScene(walletId: walletId, assetId: assetId) ==
            TransactionsRequest(
                walletId: walletId,
                type: .asset(assetId: assetId),
                limit: 25
            )
        )
    }

    @Test
    func perpetualScene() {
        let walletId = WalletId.multicoin(address: "wallet")
        let assetId = AssetId(chain: .hyperCore, tokenId: AssetId.subTokenId(["perpetual", "SOL"]))

        #expect(
            TransactionsRequest.perpetualScene(
                walletId: walletId,
                assetId: assetId,
                limit: 50
            ) ==
            TransactionsRequest(
                walletId: walletId,
                type: .asset(assetId: assetId),
                filters: [.types([
                    TransactionType.perpetualOpenPosition.rawValue,
                    TransactionType.perpetualClosePosition.rawValue
                ])],
                limit: 50
            )
        )
    }
}
