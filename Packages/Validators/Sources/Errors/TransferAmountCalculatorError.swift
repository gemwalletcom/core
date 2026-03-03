// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import BigInt
import Primitives
import Localization
import Formatters

public enum TransferAmountCalculatorError: Equatable {
    case insufficientBalance(Asset)
    case insufficientNetworkFee(Asset, required: BigInt?)
    case minimumAccountBalanceTooLow(Asset, required: BigInt)
}

extension TransferAmountCalculatorError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .insufficientBalance(let asset):
            Localized.Transfer.insufficientBalance(Self.title(asset: asset))
        case .insufficientNetworkFee(let asset, _):
            Localized.Transfer.insufficientNetworkFeeBalance(Self.title(asset: asset))
        case .minimumAccountBalanceTooLow(let asset, let required):
            Localized.Transfer.minimumAccountBalance(Self.formattedValue(required, asset: asset))
        }
    }

    static private func title(asset: Asset) -> String {
        asset.name == asset.symbol ? asset.name : String(format: "%@ (%@)", asset.name, asset.symbol)
    }

    static private func formattedValue(_ value: BigInt, asset: Asset) -> String {
        ValueFormatter(style: .full).string(value, asset: asset)
    }
}
