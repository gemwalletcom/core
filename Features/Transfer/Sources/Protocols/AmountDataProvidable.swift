// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

protocol AmountDataProvidable {
    var asset: Asset { get }
    var title: String { get }
    var amountType: AmountType { get }
    var minimumValue: BigInt { get }
    var canChangeValue: Bool { get }
    var reserveForFee: BigInt { get }

    func availableValue(from assetData: AssetData) -> BigInt
    func shouldReserveFee(from assetData: AssetData) -> Bool
    func maxValue(from assetData: AssetData) -> BigInt
    func recipientData() -> RecipientData
    func makeTransferData(value: BigInt) async throws -> TransferData
}

extension AmountDataProvidable {
    func maxValue(from assetData: AssetData) -> BigInt {
        shouldReserveFee(from: assetData) ? max(.zero, availableValue(from: assetData) - reserveForFee) : availableValue(from: assetData)
    }
}
