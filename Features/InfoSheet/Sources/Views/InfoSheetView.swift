// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Style
import Components

struct InfoSheetView: View {
    private let model: InfoSheetModel

    init(model: InfoSheetModel) {
        self.model = model
    }

    var body: some View {
        VStack(spacing: .medium) {
            Group {
                switch model.image {
                case .image(let image):
                    image
                        .resizable()
                        .aspectRatio(contentMode: .fit)
                case .assetImage(let asset):
                    AssetImageView(
                        assetImage: asset,
                        size: .image.semiExtraLarge
                    )
                case nil: EmptyView()
                }
            }
            .frame(size: .image.large)

            VStack(spacing: .small) {
                Text(model.title)
                    .textStyle(model.titleStyle)
                Text(.init(model.description))
                    .textStyle(model.descriptionStyle)
            }
            .multilineTextAlignment(.center)
            .minimumScaleFactor(0.85)
        }
        .frame(maxWidth: .infinity, alignment: .top)
    }
}
