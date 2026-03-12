// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Primitives
import Style
import Formatters
import Components

public struct ChartHeaderViewModel {
    public let period: ChartPeriod
    public let date: Date?
    public let price: Double
    public let priceChangePercentage: Double
    public let headerValue: Double?
    public let type: ChartValueType

    private let formatter: CurrencyFormatter

    public init(
        period: ChartPeriod,
        date: Date?,
        price: Double,
        priceChangePercentage: Double,
        headerValue: Double? = nil,
        formatter: CurrencyFormatter,
        type: ChartValueType = .price
    ) {
        self.period = period
        self.date = date
        self.price = price
        self.priceChangePercentage = priceChangePercentage
        self.headerValue = headerValue
        self.type = type
        self.formatter = formatter
    }

    private var valueChange: PriceChangeViewModel? {
        type == .priceChange ? PriceChangeViewModel(value: price, currencyFormatter: formatter) : nil
    }

    public var dateText: String? {
        guard let date else { return nil }
        switch period {
        case .hour:
            return date.formatted(.dateTime.hour().minute())
        case .day:
            return date.formatted(.dateTime.weekday(.abbreviated).hour().minute())
        case .week, .month:
            return date.formatted(.dateTime.month(.abbreviated).day().hour().minute())
        case .year, .all:
            return date.formatted(.dateTime.year().month(.abbreviated).day())
        }
    }

    public var headerValueText: String? {
        headerValue.map { formatter.string($0) }
    }

    public var priceText: String {
        valueChange?.text ?? formatter.string(price)
    }

    public var priceColor: Color {
        valueChange?.color ?? Colors.black
    }

    public var priceChangeText: String? {
        guard price != 0 else { return nil }
        switch type {
        case .priceChange:
            guard headerValue != nil, priceChangePercentage != 0 else { return nil }
            return "(\(CurrencyFormatter.percentSignLess.string(priceChangePercentage)))"
        case .price:
            return CurrencyFormatter.percent.string(priceChangePercentage)
        }
    }

    public var priceChangeTextColor: Color {
        PriceChangeColor.color(for: priceChangePercentage)
    }

    public var priceFont: Font {
        headerValue != nil ? .app.headline : .title2
    }

    public var priceChangeFont: Font {
        headerValue != nil ? .app.headline : .callout
    }
}
