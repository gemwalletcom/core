// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Formatters
import Localization
import PrimitivesComponents
import Style
import InfoSheet

struct AssetMarketViewModel {
    private let market: AssetMarket
    private let assetSymbol: String
    private let currencyFormatter: CurrencyFormatter
    private let priceFormatter: CurrencyFormatter
    private let percentFormatter: CurrencyFormatter

    init(
        market: AssetMarket,
        assetSymbol: String,
        currency: String
    ) {
        self.market = market
        self.assetSymbol = assetSymbol
        self.currencyFormatter = CurrencyFormatter(type: .abbreviated, currencyCode: currency)
        self.priceFormatter = CurrencyFormatter(currencyCode: currency)
        self.percentFormatter = CurrencyFormatter(type: .percent, currencyCode: currency)
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
        allTimeViewModel(title: Localized.Asset.allTimeHigh, chartValue: market.allTimeHighValue)
    }

    var allTimeLow: MarketValueViewModel {
        allTimeViewModel(title: Localized.Asset.allTimeLow, chartValue: market.allTimeLowValue)
    }

    // MARK: - Private

    private func allTimeViewModel(title: String, chartValue: ChartValuePercentage?) -> MarketValueViewModel {
        guard let chartValue else {
            return MarketValueViewModel(title: title, subtitle: nil)
        }
        return MarketValueViewModel(
            title: title,
            titleExtra: formatDate(chartValue.date),
            subtitle: formatPrice(Double(chartValue.value)),
            subtitleExtra: percentFormatter.string(Double(chartValue.percentage)),
            subtitleExtraStyle: TextStyle(font: .callout, color: PriceViewModel.priceChangeTextColor(value: Double(chartValue.percentage)))
        )
    }

    private func formatCurrency(_ value: Double?) -> String? {
        value.map { currencyFormatter.string($0) }
    }

    private func formatPrice(_ value: Double?) -> String? {
        value.map { priceFormatter.string($0) }
    }

    private func formatSupply(_ value: Double?) -> String? {
        value.map { currencyFormatter.string(double: $0, symbol: assetSymbol) }
    }

    private func formatDate(_ date: Date?) -> String? {
        date.map { TransactionDateFormatter(date: $0).section }
    }
}
