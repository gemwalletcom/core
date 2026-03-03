// Copyright (c). Gem Wallet. All rights reserved.

import EarnService

public struct AmountService: Sendable {
    let earnDataProvider: any EarnDataProvidable

    public init(earnDataProvider: any EarnDataProvidable) {
        self.earnDataProvider = earnDataProvider
    }
}
