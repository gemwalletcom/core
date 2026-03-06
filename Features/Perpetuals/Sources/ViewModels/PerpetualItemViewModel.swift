// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Components
import Primitives
import Style
import SwiftUI
import PrimitivesComponents
import Formatters

struct PerpetualItemViewModel: ListAssetItemViewable {
    
    let model: PerpetualViewModel
    
    init(
        model: PerpetualViewModel
    ) {
        self.model = model
    }

    var name: String { model.name }
    var symbol: String? { .none }
    var action: ((ListAssetItemAction) -> Void)?
    
    var assetImage: AssetImage {
        model.assetImage
    }
    
    var subtitleView: ListAssetItemSubtitleView {
        .price(
            price: TextValue(
                text: model.priceText,
                style: TextStyle(font: .footnote, color: Colors.gray)
            ),
            priceChangePercentage24h: TextValue(
                text: model.priceChangeText,
                style: TextStyle(font: .footnote, color: model.priceChangeTextColor)
            )
        )
    }
    
    var rightView: ListAssetItemRightView {
        .balance(
            balance: TextValue(
                text: model.volumeField.value.text,
                style: TextStyle(font: .body, color: .primary, fontWeight: .semibold)
            ),
            totalFiat: TextValue(
                text: "",
                style: TextStyle(font: .footnote, color: .secondary)
            )
        )
    }
}
