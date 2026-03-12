// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Formatters
import Primitives
@testable import PrimitivesComponents

public extension ChartHeaderViewModel {
    static func mock(
        period: ChartPeriod = .day,
        date: Date? = nil,
        price: Double = 100,
        priceChangePercentage: Double = 5,
        headerValue: Double? = nil,
        type: ChartValueType = .price
    ) -> ChartHeaderViewModel {
        ChartHeaderViewModel(
            period: period,
            date: date,
            price: price,
            priceChangePercentage: priceChangePercentage,
            headerValue: headerValue,
            formatter: CurrencyFormatter(type: .currency, currencyCode: "USD"),
            type: type
        )
    }
}
