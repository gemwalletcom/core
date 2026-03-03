// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Primitives
import Components

struct ValidatorSelectionView: View {

    private let value: ListItemValue<DelegationValidator>
    private let selection: String?
    private let action: ((DelegationValidator) -> Void)?

    init(
        value: ListItemValue<DelegationValidator>,
        selection: String?,
        action: ((DelegationValidator) -> Void)?
    ) {
        self.value = value
        self.selection = selection
        self.action = action
    }

    var body: some View {
        HStack {
            ValidatorImageView(model: ValidatorViewModel(validator: value.value))
            ListItemSelectionView(
                title: value.title,
                titleExtra: .none,
                titleTag: .none,
                titleTagType: .none,
                subtitle: value.subtitle,
                subtitleExtra: .none,
                value: value.value.id,
                selection: selection
            ) { _ in
                action?(value.value)
            }
        }
    }
}
