// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

extension FiatQuote {
    public static func mock(
        id: String = UUID().uuidString,
        fiatAmount: Double = 0,
        cryptoAmount: Double = 0,
        type: FiatQuoteType = .buy,
        fiatCurrency: String = Currency.usd.rawValue
    ) -> FiatQuote {
        FiatQuote(
            id: id,
            provider: FiatProvider(id: "moonpay", name: "", imageUrl: ""),
            type: type,
            fiatAmount: fiatAmount,
            fiatCurrency: fiatCurrency,
            cryptoAmount: cryptoAmount
        )
    }
}
