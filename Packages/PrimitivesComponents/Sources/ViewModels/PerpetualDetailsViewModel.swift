// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives
import Formatters
import Components
import Style
import SwiftUI

public enum PerpetualDetailsType: Sendable {
    case open(PerpetualConfirmData)
    case close(PerpetualConfirmData)
    case increase(PerpetualConfirmData)
    case reduce(PerpetualReduceData)

    public init(_ perpetualType: PerpetualType) {
        switch perpetualType {
        case .open(let data): self = .open(data)
        case .close(let data): self = .close(data)
        case .increase(let data): self = .increase(data)
        case .reduce(let data): self = .reduce(data)
        case .modify: fatalError("not suppoerted")
        }
    }

    var data: PerpetualConfirmData {
        switch self {
        case .open(let data), .close(let data), .increase(let data): data
        case .reduce(let data): data.data
        }
    }
}

public struct PerpetualDetailsViewModel: Sendable, Identifiable {
    public var id: String { type.data.baseAsset.id.identifier }
    private let type: PerpetualDetailsType
    private let currencyFormatter = CurrencyFormatter(type: .currency, currencyCode: Currency.usd.rawValue)
    private let percentFormatter = CurrencyFormatter(type: .percent, currencyCode: Currency.usd.rawValue)
    private let percentSignLessFormatter = CurrencyFormatter.percentSignLess
    private let autocloseFormatter = AutocloseFormatter()

    public init(type: PerpetualDetailsType) {
        self.type = type
    }

    var data: PerpetualConfirmData {
        type.data
    }

    public var listItemModel: ListItemModel {
        ListItemModel(
            title: Localized.Common.details,
            subtitle: listItemSubtitle,
            subtitleStyle: listItemSubtitleStyle
        )
    }

    var positionField: ListItemField {
        ListItemField(
            title: TextValue(text: Localized.Perpetual.position, style: .body),
            value: TextValue(text: positionText, style: TextStyle(font: .callout, color: directionViewModel.color))
        )
    }
    var positionText: String { "\(directionViewModel.title) \(leverageText)" }

    var directionViewModel: PerpetualDirectionViewModel {
        let direction = switch type {
        case .open(let data), .close(let data), .increase(let data): data.direction
        case .reduce(let data): data.positionDirection
        }
        return PerpetualDirectionViewModel(direction: direction)
    }

    var leverageTitle: String { Localized.Perpetual.leverage}
    var leverageText: String { "\(data.leverage)x" }

    var slippageField: ListItemField {
        ListItemField(title: Localized.Swap.slippage, value: percentSignLessFormatter.string(data.slippage))
    }

    var marketPriceField: ListItemField {
        ListItemField(title: Localized.PriceAlerts.SetAlert.currentPrice, value: currencyFormatter.string(data.marketPrice))
    }

    var entryPriceField: ListItemField? {
        guard let price = data.entryPrice else { return nil }
        return ListItemField(title: Localized.Perpetual.entryPrice, value: currencyFormatter.string(price))
    }

    var pnlViewModel: PnLViewModel {
        PnLViewModel(
            pnl: data.pnl,
            marginAmount: data.marginAmount,
            currencyFormatter: currencyFormatter,
            percentFormatter: percentFormatter
        )
    }
    var pnlField: ListItemField? {
        guard let text = pnlViewModel.text else { return nil }
        return ListItemField(
            title: TextValue(text: pnlViewModel.title, style: .body),
            value: TextValue(text: text, style: pnlViewModel.textStyle)
        )
    }
    var pnlText: String? { pnlViewModel.text }
    var pnlTextStyle: TextStyle { pnlViewModel.textStyle }

    var marginField: ListItemField {
        ListItemField(title: Localized.Perpetual.margin, value: currencyFormatter.string(data.marginAmount))
    }

    var sizeField: ListItemField {
        ListItemField(title: Localized.Perpetual.size, value: currencyFormatter.string(data.fiatValue))
    }

    var autocloseTitle: String { Localized.Perpetual.autoClose }
    var autocloseText: (subtitle: String, subtitleExtra: String?) {
        autocloseFormatter.format(
            takeProfit: data.takeProfit.flatMap { currencyFormatter.double(from: $0) },
            stopLoss: data.stopLoss.flatMap { currencyFormatter.double(from: $0) }
        )
    }
    var showAutoclose: Bool { data.takeProfit != nil || data.stopLoss != nil }
}

// MARK: - Private

extension PerpetualDetailsViewModel {
    private var listItemSubtitle: String? {
        switch type {
        case .open: String(format: "%@ %@", directionViewModel.title, leverageText)
        case .close: pnlText
        case .increase: directionViewModel.increaseTitle
        case .reduce: directionViewModel.reduceTitle
        }
    }

    private var listItemSubtitleStyle: TextStyle {
        switch type {
        case .open: TextStyle(font: .callout, color: directionViewModel.color)
        case .close: pnlTextStyle
        case .increase, .reduce: .calloutSecondary
        }
    }
}
