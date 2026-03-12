// Copyright (c). Gem Wallet. All rights reserved.

import Style
import Components
import Primitives

public struct SymbolViewModel: Sendable, AmountDisplayable {
    private let symbol: String
    private let image: AssetImage

    public init(asset: Asset) {
        self.symbol = asset.symbol
        self.image = AssetViewModel(asset: asset).assetImage
    }

    public init(assetId: AssetId) {
        self.symbol = assetId.tokenId?.truncate(first: 8, last: 6) ?? assetId.chain.asset.symbol
        self.image = AssetIdViewModel(assetId: assetId).assetImage
    }

    public var amount: TextValue {
        TextValue(
            text: symbol,
            style: TextStyle(
                font: .body,
                color: Colors.black,
                fontWeight: .semibold
            ),
            lineLimit: 1
        )
    }

    public var fiat: TextValue? { nil }

    public var assetImage: AssetImage? { image }
}
