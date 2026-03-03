// Copyright (c). Gem Wallet. All rights reserved.

public typealias AmountInputAction = ((AmountInput) -> Void)?

public struct AmountInput {
    public let type: AmountType
    public let asset: Asset
    
    public init(type: AmountType, asset: Asset) {
        self.type = type
        self.asset = asset
    }
}

extension AmountInput: Identifiable {
    public var id: String {
        asset.id.identifier
    }
}

extension AmountInput: Hashable {}
