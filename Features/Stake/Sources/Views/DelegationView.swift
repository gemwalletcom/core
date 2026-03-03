// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Style
import Components

public struct DelegationView: View {

    private let delegation: DelegationViewModel

    public init(delegation: DelegationViewModel) {
        self.delegation = delegation
    }

    public var body: some View {
        ListItemView(
           title: delegation.validatorText,
           titleStyle: delegation.titleStyle,
           titleExtra: delegation.stateModel.title,
           titleStyleExtra: delegation.stateModel.textStyle,
           subtitle: delegation.balanceText,
           subtitleStyle: delegation.subtitleStyle,
           subtitleExtra: delegation.fiatValueText,
           subtitleStyleExtra: delegation.subtitleExtraStyle,
           imageStyle: .asset(assetImage: delegation.validatorImage)
        )
    }
}
