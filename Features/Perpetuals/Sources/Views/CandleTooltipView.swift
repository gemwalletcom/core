// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Style
import Components

struct CandleTooltipView: View {
    let model: CandleTooltipViewModel

    var body: some View {
        Grid(alignment: .leading, horizontalSpacing: Spacing.small, verticalSpacing: Spacing.extraSmall) {
            GridItemView(title: model.openTitle, value: model.openValue)
            GridItemView(title: model.highTitle, value: model.highValue)
            GridItemView(title: model.lowTitle, value: model.lowValue)
            GridItemView(title: model.closeTitle, value: model.closeValue)

            Divider()
                .gridCellColumns(2)
                .padding(.vertical, Spacing.tiny)

            GridItemView(title: model.changeTitle, value: model.changeValue)
            GridItemView(title: model.volumeTitle, value: model.volumeValue)
        }
        .padding(Spacing.small)
        .background(.thickMaterial)
        .clipShape(RoundedRectangle(cornerRadius: Spacing.small))
        .overlay(
            RoundedRectangle(cornerRadius: Spacing.small)
                .stroke(Colors.black.opacity(.opacity8), lineWidth: .space1)
        )
        .shadow(color: .black.opacity(.opacity12), radius: Spacing.small, y: Spacing.tiny)
        .fixedSize()
    }
}
