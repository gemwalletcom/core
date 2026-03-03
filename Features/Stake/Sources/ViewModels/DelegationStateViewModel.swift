// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Localization
import Primitives
import Style

public struct DelegationStateViewModel {

    public let state: DelegationState

    public init(state: DelegationState) {
        self.state = state
    }

    public var title: String {
        switch state {
        case .active: Localized.Stake.active
        case .pending: Localized.Stake.pending
        case .inactive: Localized.Stake.inactive
        case .activating: Localized.Stake.activating
        case .deactivating: Localized.Stake.deactivating
        case .awaitingWithdrawal: Localized.Stake.awaitingWithdrawal
        }
    }

    public var color: Color {
        switch state {
        case .active: Colors.green
        case .pending,
            .activating,
            .deactivating: Colors.orange
        case .inactive,
            .awaitingWithdrawal: Colors.red
        }
    }

    public var textStyle: TextStyle {
        TextStyle(font: .callout, color: color)
    }
}
