// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Components
import Localization
import Primitives

public struct PerpetualDetailsView: View {
    private var model: PerpetualDetailsViewModel

    public init(model: PerpetualDetailsViewModel) {
        self.model = model
    }

    public var body: some View {
        List {
            Section {
                ListItemView(field: model.positionField)

                if let pnlField = model.pnlField {
                    ListItemView(field: pnlField)
                }
            }

            Section {
                ListItemView(field: model.marginField)
                ListItemView(field: model.sizeField)
            }

            if model.showAutoclose {
                Section {
                    ListItemView(
                        title: model.autocloseTitle,
                        subtitle: model.autocloseText.subtitle,
                        subtitleExtra: model.autocloseText.subtitleExtra
                    )
                }
            }

            Section {
                ListItemView(field: model.marketPriceField)

                if let entryPriceField = model.entryPriceField {
                    ListItemView(field: entryPriceField)
                }

                ListItemView(field: model.slippageField)
            }
        }
        .toolbarDismissItem(type: .close, placement: .topBarLeading)
        .navigationTitle(Localized.Common.details)
        .navigationBarTitleDisplayMode(.inline)
        .listSectionSpacing(.compact)
        .contentMargins([.top], .extraSmall, for: .scrollContent)
    }
}
