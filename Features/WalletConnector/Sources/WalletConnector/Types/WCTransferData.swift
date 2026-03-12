// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct WCTransferData: Identifiable, Sendable {
    public let tranferData: TransferData
    public let wallet: Wallet
    public let simulation: SimulationResult

    public init(tranferData: TransferData, wallet: Wallet, simulation: SimulationResult) {
        self.tranferData = tranferData
        self.wallet = wallet
        self.simulation = simulation
    }

    public var id: String { wallet.id }
}
