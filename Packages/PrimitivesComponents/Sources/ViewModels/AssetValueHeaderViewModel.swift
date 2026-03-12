// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Localization
import Primitives
import Style
import SwiftUI

struct AssetValueHeaderViewModel: HeaderViewModel {
    private static let formatter = ValueFormatter(style: .full)

    let data: AssetValueHeaderData

    let isWatchWallet: Bool = false
    let buttons: [HeaderButton] = []

    var assetImage: AssetImage? {
        AssetViewModel(asset: data.asset).assetImage
    }

    var title: String {
        switch data.value {
        case .unlimited:
            return Localized.Simulation.Header.unlimitedAsset(data.asset.symbol)
        case .exact(let value):
            return Self.formatter.string(
                value,
                decimals: data.asset.decimals.asInt,
                currency: data.asset.symbol
            )
        }
    }

    var subtitle: String? { nil }

    var subtitleColor: Color { Colors.gray }
}
