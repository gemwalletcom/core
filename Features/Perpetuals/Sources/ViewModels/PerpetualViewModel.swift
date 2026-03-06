// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesComponents
import Formatters
import Components
import Localization
import Style
import SwiftUI

public struct PerpetualViewModel {
    public let perpetual: Perpetual
    private let currencyFormatter: CurrencyFormatter
    private let percentFormatter = CurrencyFormatter.percent
    private let fundingRateFormatter: NumberFormatter = {
        let formatter = NumberFormatter()
        formatter.numberStyle = .decimal
        formatter.minimumFractionDigits = 2
        formatter.maximumFractionDigits = 5
        return formatter
    }()
    
    public init(perpetual: Perpetual, currencyStyle: CurrencyFormatterType = .abbreviated) {
        self.perpetual = perpetual
        self.currencyFormatter = CurrencyFormatter(type: currencyStyle, currencyCode: Currency.usd.rawValue)
    }
    
    public var name: String {
        perpetual.name
    }
    
    public var assetImage: AssetImage {
        AssetIdViewModel(assetId: perpetual.assetId).assetImage
    }
    
    public var volumeField: ListItemField {
        ListItemField(title: Localized.Markets.dailyVolume, value: currencyFormatter.string(perpetual.volume24h))
    }

    public var openInterestField: ListItemField {
        ListItemField(title: Localized.Info.OpenInterest.title, value: currencyFormatter.string(perpetual.openInterest))
    }

    public var fundingRateField: ListItemField {
        let text: String
        if let formattedNumber = fundingRateFormatter.string(from: NSNumber(value: perpetual.funding)) {
            text = "\(formattedNumber)%"
        } else {
            text = percentFormatter.string(perpetual.funding)
        }
        return ListItemField(title: Localized.Info.FundingRate.title, value: text)
    }
    
    public var priceText: String {
        currencyFormatter.string(perpetual.price)
    }
    
    public var priceChangeText: String {
        percentFormatter.string(perpetual.pricePercentChange24h)
    }
    
    public var priceChangeTextColor: Color {
        PriceChangeColor.color(for: perpetual.pricePercentChange24h)
    }
}
