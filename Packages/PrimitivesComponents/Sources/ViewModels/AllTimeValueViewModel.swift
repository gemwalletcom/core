// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Components
import Formatters
import Localization
import Style

public struct AllTimeValueViewModel: Sendable {
    private let priceFormatter: CurrencyFormatter
    private let percentFormatter: CurrencyFormatter

    public init(priceFormatter: CurrencyFormatter, percentFormatter: CurrencyFormatter) {
        self.priceFormatter = priceFormatter
        self.percentFormatter = percentFormatter
    }

    public func allTimeHigh(chartValue: ChartValuePercentage) -> ListItemModel {
        model(title: Localized.Asset.allTimeHigh, chartValue: chartValue)
    }

    public func allTimeLow(chartValue: ChartValuePercentage) -> ListItemModel {
        model(title: Localized.Asset.allTimeLow, chartValue: chartValue)
    }

    public func model(title: String, chartValue: ChartValuePercentage) -> ListItemModel {
        let percentage = Double(chartValue.percentage)
        return ListItemModel(
            title: title,
            titleExtra: TransactionDateFormatter(date: chartValue.date).section,
            subtitle: priceFormatter.string(Double(chartValue.value)),
            subtitleExtra: percentFormatter.string(percentage),
            subtitleStyleExtra: TextStyle(font: .callout, color: PriceViewModel.priceChangeTextColor(value: percentage))
        )
    }
}
