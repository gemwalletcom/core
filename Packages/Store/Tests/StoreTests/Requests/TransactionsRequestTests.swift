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
}
