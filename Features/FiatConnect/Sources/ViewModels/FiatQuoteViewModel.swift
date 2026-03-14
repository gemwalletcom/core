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

}

extension FiatQuoteViewModel: Identifiable {
    var id: String {
        "\(asset.id.identifier)\(quote.provider.id)\(quote.cryptoAmount)"
    }
}

// MARK: - SimpleListItemViewable

extension FiatQuoteViewModel: SimpleListItemViewable {
    var assetImage: AssetImage {
        AssetImage(
            placeholder: quote.provider.image,
            chainPlaceholder: isSelected ? Images.Wallets.selected : nil
        )
    }

    var subtitle: String? { amountText }
}

// MARK: - Hashable

extension FiatQuoteViewModel: Hashable {
    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }
}
