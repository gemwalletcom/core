// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct AssetValueHeaderData: Sendable, Equatable {
    public let asset: Asset
    public let value: ApprovalValue

    public init(asset: Asset, value: ApprovalValue) {
        self.asset = asset
        self.value = value
    }
}
