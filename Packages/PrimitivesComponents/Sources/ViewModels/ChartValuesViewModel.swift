// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import Primitives
import Style
import SwiftUI

internal import Charts

public struct ChartValuesViewModel: Sendable {
    public let period: ChartPeriod
    public let price: Price?
    public let values: ChartValues
    public let lineColor: Color
    public let formatter: CurrencyFormatter
    public let type: ChartValueType
    public let headerValue: Double?

    public static let defaultPeriod = ChartPeriod.day

    public init(
        period: ChartPeriod,
        price: Price?,
        values: ChartValues,
        lineColor: Color = Colors.blue,
        formatter: CurrencyFormatter,
        type: ChartValueType = .price,
        headerValue: Double? = nil
    ) {
        self.period = period
        self.price = price
        self.values = values
        self.lineColor = lineColor
        self.formatter = formatter
        self.type = type
        self.headerValue = headerValue
    }

    var charts: [ChartDateValue] { values.charts }
    var lowerBoundValueText: String { formatter.string(values.lowerBoundValue) }
    var upperBoundValueText: String { formatter.string(values.upperBoundValue) }

    var chartHeaderViewModel: ChartHeaderViewModel? {
        guard let price else { return nil }
        let priceChangePercentage = switch type {
        case .priceChange:
            price.priceChangePercentage24h
        case .price:
            period == Self.defaultPeriod
                ? price.priceChangePercentage24h
                : PriceChangeCalculator.calculate(.percentage(from: values.baseValue, to: price.price))
        }
        return ChartHeaderViewModel(period: period, date: nil, price: price.price, priceChangePercentage: priceChangePercentage, headerValue: headerValue, formatter: formatter, type: type)
    }

    public static func priceChange(
        charts: [ChartDateValue],
        period: ChartPeriod,
        formatter: CurrencyFormatter,
        showHeaderValue: Bool = false
    ) -> ChartValuesViewModel? {
        guard let values = try? ChartValues.from(charts: charts), values.hasVariation else {
            return nil
        }
        let price = Price(
            price: values.lastValue - values.firstValue,
            priceChangePercentage24h: PriceChangeCalculator.calculate(.percentage(from: values.firstValue, to: values.lastValue)),
            updatedAt: .now
        )
        return ChartValuesViewModel(
            period: period,
            price: price,
            values: values,
            formatter: formatter,
            type: .priceChange,
            headerValue: showHeaderValue ? values.lastValue : nil
        )
    }

    func headerViewModel(for element: ChartDateValue) -> ChartHeaderViewModel {
        let base = type == .priceChange ? values.firstValue : values.baseValue
        let priceChangePercentage = PriceChangeCalculator.calculate(.percentage(from: base, to: element.value))
        let displayPrice = type == .priceChange ? element.value - base : element.value
        let elementHeaderValue = headerValue != nil ? element.value : nil
        return ChartHeaderViewModel(period: period, date: element.date, price: displayPrice, priceChangePercentage: priceChangePercentage, headerValue: elementHeaderValue, formatter: formatter, type: type)
    }
}
