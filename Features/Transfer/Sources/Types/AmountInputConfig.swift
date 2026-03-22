// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Primitives
import Style
import SwiftUI

struct AmountInputConfig: CurrencyInputConfigurable {
    let sceneType: AmountType
    let inputType: AmountInputType
    let asset: Asset
    let currencyFormatter: CurrencyFormatter
    let numberSanitizer: NumberSanitizer
    let secondaryText: String
    let onTapActionButton: (() -> Void)?

    var placeholder: String { .zero }
    var keyboardType: UIKeyboardType {
        switch sceneType {
        case .transfer, .deposit, .withdraw, .perpetual, .freeze, .unfreeze, .earn: .decimalPad
        case .stake(let stakeType):
            switch stakeType {
            case .stake, .unstake: asset.chain == .tron ? .numberPad : .decimalPad
            case .redelegate, .withdraw: .decimalPad
            }
        }
    }

    var currencyPosition: CurrencyTextField.CurrencyPosition {
        switch inputType {
        case .asset: .trailing
        case .fiat: .leading
        }
    }

    var currencySymbol: String {
        switch inputType {
        case .asset: asset.symbol
        case .fiat: currencyFormatter.symbol
        }
    }

    var actionStyle: CurrencyInputActionStyle? {
        switch sceneType {
        case .transfer: CurrencyInputActionStyle(
            position: .secondary,
            image: Images.Actions.swap.renderingMode(.template)
        )
        case .deposit, .withdraw, .perpetual, .stake, .freeze, .unfreeze, .earn: nil
        }
    }

    var sanitizer: ((String) -> String)? {
        { numberSanitizer.sanitize($0) }
    }
}
