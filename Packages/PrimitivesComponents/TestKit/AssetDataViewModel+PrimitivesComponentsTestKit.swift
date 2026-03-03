// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Primitives
import PrimitivesTestKit
@testable import PrimitivesComponents

public extension AssetDataViewModel {
    static func mock(
        assetData: AssetData = .mock(),
        formatter: ValueFormatter = .medium,
        currencyCode: String = "USD"
    ) -> AssetDataViewModel {
        AssetDataViewModel(
            assetData: assetData,
            formatter: formatter,
            currencyCode: currencyCode
        )
    }
}
