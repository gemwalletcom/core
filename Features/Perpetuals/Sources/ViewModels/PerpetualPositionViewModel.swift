// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Formatters
import SwiftUI
import Style
import Components
import PrimitivesComponents
import Localization

public struct PerpetualPositionViewModel {
    public let data: PerpetualPositionData
    private let currencyFormatter: CurrencyFormatter
    private let percentFormatter: CurrencyFormatter
    private let autocloseFormatter: AutocloseFormatter

    public init(
        _ data: PerpetualPositionData,
        currencyStyle: CurrencyFormatterType = .currency
    ) {
        self.data = data
        self.currencyFormatter = CurrencyFormatter(type: currencyStyle, currencyCode: Currency.usd.rawValue)
        self.percentFormatter = CurrencyFormatter(type: .percent, currencyCode: Currency.usd.rawValue)
        self.autocloseFormatter = AutocloseFormatter(currencyFormatter: currencyFormatter)
    }
    
    public var assetImage: AssetImage {
        AssetIdViewModel(assetId: data.perpetual.assetId).assetImage
    }
    
    public var nameText: String {
        data.asset.name
    }
    
    public var symbolText: String {
        data.asset.symbol
    }
    
    public var leverageText: String {
        "\(Int(data.position.leverage))x"
    }
    
    public var directionText: String {
        PerpetualDirectionViewModel(direction: data.position.direction).title
    }
    
    public var positionTypeText: String {
        "\(directionText.uppercased()) \(leverageText)"
    }
    
    public var positionTypeColor: Color {
        PerpetualDirectionViewModel(direction: data.position.direction).color
    }
    
    public var pnlViewModel: PnLViewModel {
        PnLViewModel(
            pnl: data.position.pnl,
            marginAmount: data.position.marginAmount,
            currencyFormatter: currencyFormatter,
            percentFormatter: percentFormatter
        )
    }
    public var pnlField: ListItemField {
        ListItemField(
            title: TextValue(text: pnlViewModel.title, style: .body),
            value: TextValue(text: pnlViewModel.text ?? "", style: pnlViewModel.textStyle)
        )
    }
    public var pnlColor: Color { pnlViewModel.color }
    public var pnlPercent: Double { pnlViewModel.percent }
    public var pnlWithPercentText: String { pnlViewModel.text ?? "" }

    public var marginAmountText: String {
        currencyFormatter.string(data.position.marginAmount)
    }

    var autocloseTitle: String { Localized.Perpetual.autoClose }
    var autocloseText: (subtitle: String, subtitleExtra: String?) {
        autocloseFormatter.format(
            takeProfit: data.position.takeProfit?.price,
            stopLoss: data.position.stopLoss?.price
        )
    }

    public var marginField: ListItemField {
        let marginAmount = currencyFormatter.string(data.position.marginAmount)
        return ListItemField(title: Localized.Perpetual.margin, value: "\(marginAmount) (\(data.position.marginType.displayText))")
    }

    public var fundingPaymentsField: ListItemField {
        ListItemField(
            title: TextValue(text: Localized.Info.FundingPayments.title, style: .body),
            value: TextValue(text: fundingPaymentsModel.text ?? "-", style: fundingPaymentsModel.textStyle)
        )
    }
    public var fundingPaymentsColor: Color { fundingPaymentsModel.color }

    public var sizeField: ListItemField {
        ListItemField(title: Localized.Perpetual.size, value: currencyFormatter.string(data.position.sizeValue))
    }

    public var entryPriceField: ListItemField {
        ListItemField(title: Localized.Perpetual.entryPrice, value: currencyFormatter.string(data.position.entryPrice))
    }

    public var liquidationPriceField: ListItemField? {
        guard let price = data.position.liquidationPrice, price > 0 else { return .none }
        return ListItemField(
            title: TextValue(text: Localized.Info.LiquidationPrice.title, style: .body),
            value: TextValue(text: currencyFormatter.string(price), style: liquidationPriceTextStyle)
        )
    }
}

// MARK: - Private

extension PerpetualPositionViewModel {
    private var fundingPaymentsModel: PriceChangeViewModel {
        PriceChangeViewModel(value: data.position.funding.map { Double($0) }, currencyFormatter: currencyFormatter)
    }

    private var liquidationPriceTextStyle: TextStyle {
        TextStyle(font: .callout, color: Colors.secondaryText)
    }
}

extension PerpetualPositionViewModel: Identifiable {
    public var id: String { data.position.id }
}

extension PerpetualMarginType {
    var displayText: String {
        switch self {
        case .cross:
            return "cross"
        case .isolated:
            return "isolated"
        }
    }
}
