// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension Wallet {
    static func mock(
        id: String = WalletId.mock().id,
        externalId: String? = nil,
        name: String = "",
        type: WalletType = .multicoin,
        accounts: [Account] = [],
        index: Int = 0,
        order: Int = 0,
        imageUrl: String? = nil,
        source: WalletSource = .create
    ) -> Wallet {
        Wallet(
            id: id,
            externalId: externalId,
            name: name,
            index: index.asInt32,
            type: type,
            accounts: accounts,
            order: order.asInt32,
            isPinned: false,
            imageUrl: imageUrl,
            source: source
        )
    }
}
