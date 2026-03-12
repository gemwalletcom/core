// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension AddressName {
    static func mock(
        chain: Chain = .arbitrum,
        address: String = "0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7",
        name: String = "Hyperliquid",
        type: AddressType? = nil,
        status: VerificationStatus = .verified
    ) -> AddressName {
        AddressName(
            chain: chain,
            address: address,
            name: name,
            type: type,
            status: status
        )
    }
}
