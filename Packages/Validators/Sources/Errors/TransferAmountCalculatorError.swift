// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import BigInt
import Primitives
import Localization

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
        case .minimumAccountBalanceTooLow(let asset, _):
            Localized.Transfer.minimumAccountBalance(Self.title(asset: asset))
        }
    }

    static private func title(asset: Asset) -> String {
        asset.name == asset.symbol ? asset.name : String(format: "%@ (%@)", asset.name, asset.symbol)
    }
}
