// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI
import Style

public struct ChartHeaderView: View {
    let model: ChartHeaderViewModel

    public init(model: ChartHeaderViewModel) {
        self.model = model
    }

    public var body: some View {
        VStack(spacing: Spacing.tiny) {
            if let headerValueText = model.headerValueText {
                Text(headerValueText)
                    .font(.app.largeTitle)
                    .foregroundStyle(Colors.black)
                    .numericTransition(for: headerValueText)
                    .minimumScaleFactor(0.5)
                    .lineLimit(1)
                    .padding(.bottom, Spacing.space10)
            }

            HStack(alignment: .center, spacing: Spacing.tiny) {
                Text(model.priceText)
                    .font(model.priceFont)
                    .foregroundStyle(model.priceColor)
                    .numericTransition(for: model.priceText)

                if let priceChange = model.priceChangeText {
                    Text(priceChange)
                        .font(model.priceChangeFont)
                        .foregroundStyle(model.priceChangeTextColor)
                        .numericTransition(for: priceChange)
                }
            }

            HStack {
                if let date = model.dateText {
                    Text(date)
                        .font(.footnote)
                        .foregroundStyle(Colors.gray)
                }
            }.frame(height: 16)
        }
    }
}
