// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import BigInt

public struct TransferData: Identifiable, Sendable, Hashable {
    public let type: TransferDataType
    public let recipientData: RecipientData
    public let value: BigInt
    public let canChangeValue: Bool

    public init(
        type: TransferDataType,
        recipientData: RecipientData,
        value: BigInt,
        canChangeValue: Bool = true
    ) {
        self.type = type
        self.recipientData = recipientData
        self.value = value
        self.canChangeValue = canChangeValue
    }

    public var id: String {
        [type.transactionType.rawValue, recipientData.recipient.address, String(value)].joined(separator: "-")
    }
    
    public var chain: Chain {
        type.chain
    }
}
