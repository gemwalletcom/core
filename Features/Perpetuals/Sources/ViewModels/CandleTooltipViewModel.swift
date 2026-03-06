// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives
import Formatters
import Style
import Components
import Localization

public struct CandleTooltipViewModel {
    private static let titleStyle = TextStyle(font: .caption2, color: Colors.secondaryText, fontWeight: .medium)
    private static let subtitleStyle = TextStyle(font: .caption2.monospacedDigit(), color: Colors.black, fontWeight: .semibold)
    private static let volumeFormatter = CurrencyFormatter(type: .abbreviated, currencyCode: Currency.usd.rawValue)

    private let candle: ChartCandleStick
    private let formatter: CurrencyFormatter

    public init(candle: ChartCandleStick, formatter: CurrencyFormatter) {
        self.candle = candle
        self.formatter = formatter
    }

    var openField: ListItemField {
        ListItemField(
            title: TextValue(text: Localized.Charts.Price.open, style: Self.titleStyle, lineLimit: 1),
            value: TextValue(text: formatter.string(double: candle.open), style: Self.subtitleStyle, lineLimit: 1)
        )
    }

    var closeField: ListItemField {
        ListItemField(
            title: TextValue(text: Localized.Charts.Price.close, style: Self.titleStyle, lineLimit: 1),
            value: TextValue(text: formatter.string(double: candle.close), style: Self.subtitleStyle, lineLimit: 1)
        )
    }

    var highField: ListItemField {
        ListItemField(
            title: TextValue(text: Localized.Charts.Price.high, style: Self.titleStyle, lineLimit: 1),
            value: TextValue(text: formatter.string(double: candle.high), style: Self.subtitleStyle, lineLimit: 1)
        )
    }

    var lowField: ListItemField {
        ListItemField(
            title: TextValue(text: Localized.Charts.Price.low, style: Self.titleStyle, lineLimit: 1),
            value: TextValue(text: formatter.string(double: candle.low), style: Self.subtitleStyle, lineLimit: 1)
        )
    }

    var changeField: ListItemField {
        let change = PriceChangeCalculator.calculate(.percentage(from: candle.open, to: candle.close))
        return ListItemField(
            title: TextValue(text: Localized.Charts.Price.change, style: Self.titleStyle, lineLimit: 1),
            value: TextValue(text: CurrencyFormatter.percent.string(change), style: TextStyle(font: .caption2.monospacedDigit(), color: PriceChangeColor.color(for: change), fontWeight: .semibold), lineLimit: 1)
        )
    }

    var volumeField: ListItemField {
        ListItemField(
            title: TextValue(text: Localized.Perpetual.volume, style: Self.titleStyle, lineLimit: 1),
            value: TextValue(text: Self.volumeFormatter.string(candle.volume * candle.close), style: Self.subtitleStyle, lineLimit: 1)
        )
    }
}
