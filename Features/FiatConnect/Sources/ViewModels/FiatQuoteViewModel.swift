// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives
import PrimitivesComponents
import Style
import Components
import Formatters

struct FiatQuoteViewModel: Sendable {
    let quote: FiatQuote
    let selectedQuote: FiatQuote?

    private let asset: Asset
    private let formatter: CurrencyFormatter

    init(
        asset: Asset,
        quote: FiatQuote,
        selectedQuote: FiatQuote? = nil,
        formatter: CurrencyFormatter
    ) {
        self.asset = asset
        self.quote = quote
        self.selectedQuote = selectedQuote
        self.formatter = formatter
    }

    var title: String {
        quote.provider.name
    }

    var amountText: String {
        formatter.string(double: quote.cryptoAmount, symbol: asset.symbol)
    }

    var rateText: String {
        let amount = quote.fiatAmount / quote.cryptoAmount
        return formatter.string(amount)
    }
    
    private var isSelected: Bool {
        selectedQuote?.provider == quote.provider
    }

    private var buyComparisonAmountText: String {
        guard let selectedQuote, selectedQuote.cryptoAmount > 0 else {
            return formatter.string(quote.fiatAmount)
        }

        let rate = selectedQuote.fiatAmount / selectedQuote.cryptoAmount
        return formatter.string(rate * quote.cryptoAmount)
    }
}

extension FiatQuoteViewModel: Identifiable {
    var id: String {
        "\(asset.id.identifier)\(quote.provider.id)\(quote.cryptoAmount)"
    }
}

// MARK: - SimpleListItemViewable

extension FiatQuoteViewModel: SimpleListItemViewable {
    var titleStyle: TextStyle {
        TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold)
    }

    var assetImage: AssetImage {
        AssetImage(
            placeholder: quote.provider.image,
            chainPlaceholder: isSelected ? Images.Wallets.selected : nil
        )
    }

    var subtitle: String? { amountText }

    var subtitleExtra: String? {
        switch quote.type {
        case .buy: buyComparisonAmountText
        case .sell: formatter.string(quote.fiatAmount)
        }
    }

    var subtitleStyle: TextStyle {
        TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold)
    }

    var subtitleStyleExtra: TextStyle {
        TextStyle(font: .footnote, color: Colors.gray)
    }
}

// MARK: - Hashable

extension FiatQuoteViewModel: Hashable {
    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }
}
