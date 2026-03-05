// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

extension AssetMarket {
    public static func mock(
        marketCap: Double? = 1_000_000,
        marketCapFdv: Double? = 2_000_000,
        marketCapRank: Int32? = 1,
        totalVolume: Double? = 500_000,
        circulatingSupply: Double? = 100_000,
        totalSupply: Double? = 200_000,
        maxSupply: Double? = 300_000,
        allTimeHighValue: ChartValuePercentage? = ChartValuePercentage(date: .now, value: 100, percentage: -10),
        allTimeLowValue: ChartValuePercentage? = ChartValuePercentage(date: .now, value: 1, percentage: 100)
    ) -> AssetMarket {
        AssetMarket(
            marketCap: marketCap,
            marketCapFdv: marketCapFdv,
            marketCapRank: marketCapRank,
            totalVolume: totalVolume,
            circulatingSupply: circulatingSupply,
            totalSupply: totalSupply,
            maxSupply: maxSupply,
            allTimeHighValue: allTimeHighValue,
            allTimeLowValue: allTimeLowValue
        )
    }
}
