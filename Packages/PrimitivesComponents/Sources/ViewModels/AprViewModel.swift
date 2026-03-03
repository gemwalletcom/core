// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Localization
import Style

public struct AprViewModel: Sendable {
    private let apr: Double

    public init(apr: Double) {
        self.apr = apr
    }

    public var title: TextValue {
        TextValue(text: Localized.Stake.apr(""), style: .body)
    }

    public var subtitle: TextValue {
        let text = apr > .zero ? CurrencyFormatter.percentSignLess.string(apr) : .empty
        return TextValue(text: text, style: TextStyle(font: .callout, color: Colors.green))
    }

    public var text: String { Localized.Stake.apr(subtitle.text) }

    public var showApr: Bool { !apr.isZero }
}
