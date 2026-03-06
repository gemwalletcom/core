// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Style
import Components

struct CandleTooltipView: View {
    let model: CandleTooltipViewModel

    var body: some View {
        Grid(alignment: .leading, horizontalSpacing: Spacing.small, verticalSpacing: Spacing.extraSmall) {
            GridItemView(field: model.openField)
            GridItemView(field: model.highField)
            GridItemView(field: model.lowField)
            GridItemView(field: model.closeField)

            Divider()
                .gridCellColumns(2)
                .padding(.vertical, Spacing.tiny)

            GridItemView(field: model.changeField)
            GridItemView(field: model.volumeField)
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
