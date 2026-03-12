// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Components
import Formatters
import Localization
import PrimitivesComponents
import Style
import InfoSheet

struct AssetMarketViewModel {
    private let market: AssetMarket
    private let assetSymbol: String
    private let currencyFormatter: CurrencyFormatter
    private let currency: String

    init(
        market: AssetMarket,
        assetSymbol: String,
        currency: String
    ) {
        self.market = market
        self.assetSymbol = assetSymbol
        self.currency = currency
        self.currencyFormatter = CurrencyFormatter(type: .abbreviated, currencyCode: currency)
    }

    // MARK: - Market

    var marketCap: MarketValueViewModel {
        if let rank = market.marketCapRank, Int(rank).isBetween(1, and: 1000) {
            return MarketValueViewModel(
                title: Localized.Asset.marketCap,
                subtitle: formatCurrency(market.marketCap),
                titleTag: " #\(rank) ",
                titleTagStyle: TextStyle(font: .system(.body), color: Colors.grayLight, background: Colors.grayVeryLight)
            )
        }
        return MarketValueViewModel(title: Localized.Asset.marketCap, subtitle: formatCurrency(market.marketCap))
    }

    var tradingVolume: MarketValueViewModel {
        MarketValueViewModel(title: Localized.Asset.tradingVolume, subtitle: formatCurrency(market.totalVolume))
    }

    var fdv: MarketValueViewModel {
        MarketValueViewModel(
            title: Localized.Info.FullyDilutedValuation.title,
            subtitle: formatCurrency(market.marketCapFdv),
            infoSheetType: .fullyDilutedValuation
        )
    }

    // MARK: - Supply

    var circulatingSupply: MarketValueViewModel {
        MarketValueViewModel(
            title: Localized.Asset.circulatingSupply,
            subtitle: formatSupply(market.circulatingSupply),
            infoSheetType: .circulatingSupply
        )
    }

    var totalSupply: MarketValueViewModel {
        MarketValueViewModel(
            title: Localized.Asset.totalSupply,
            subtitle: formatSupply(market.totalSupply),
            infoSheetType: .totalSupply
        )
    }

    var maxSupply: MarketValueViewModel {
        MarketValueViewModel(
            title: Localized.Info.MaxSupply.title,
            subtitle: market.maxSupply == 0 ? "∞ \(assetSymbol)" : formatSupply(market.maxSupply),
            infoSheetType: .maxSupply
        )
    }

    // MARK: - All Time

    var allTimeHigh: MarketValueViewModel {
        marketValueViewModel(Localized.Asset.allTimeHigh, chartValue: market.allTimeHighValue)
    }

    var allTimeLow: MarketValueViewModel {
        marketValueViewModel(Localized.Asset.allTimeLow, chartValue: market.allTimeLowValue)
    }

    // MARK: - Private

    private var allTime: AllTimeValueViewModel {
        AllTimeValueViewModel(
            priceFormatter: CurrencyFormatter(currencyCode: currency),
            percentFormatter: CurrencyFormatter(type: .percent, currencyCode: currency)
        )
    }

    private func marketValueViewModel(_ title: String, chartValue: ChartValuePercentage?) -> MarketValueViewModel {
        guard let chartValue else {
            return MarketValueViewModel(title: title, subtitle: nil)
        }
        let item = allTime.model(title: title, chartValue: chartValue)
        return MarketValueViewModel(
            title: title,
            titleExtra: item.titleExtra,
            subtitle: item.subtitle,
            subtitleExtra: item.subtitleExtra,
            subtitleExtraStyle: item.subtitleStyleExtra
        )
    }

    private func formatCurrency(_ value: Double?) -> String? {
        value.map { currencyFormatter.string($0) }
    }

    private func formatSupply(_ value: Double?) -> String? {
        value.map { currencyFormatter.string(double: $0, symbol: assetSymbol) }
    }
}
