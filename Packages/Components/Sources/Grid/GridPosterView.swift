// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Style

public struct GridPosterView: View {

    private let model: GridPosterViewModel

    public init(model: GridPosterViewModel) {
        self.model = model
    }

    public var body: some View {
        VStack(alignment: .leading) {
            NftImageView(assetImage: model.assetImage)
                .clipShape(RoundedRectangle(cornerRadius: .medium))
                .aspectRatio(1, contentMode: .fit)
                .overlay(alignment: .topTrailing) {
                    if let count = model.count {
                        countBadge(count)
                    }
                }

            if let title = model.title {
                HStack(spacing: Spacing.tiny) {
                    Text(title)
                        .font(.body)
                        .lineLimit(1)
                        .multilineTextAlignment(.leading)

                    if model.isVerified {
                        Images.System.checkmarkSealFill
                            .font(.callout)
                            .foregroundStyle(Colors.blue)
                    }
                }
            }
            Spacer()
        }
    }

    private func countBadge(_ count: Int) -> some View {
        Text(String(count))
            .font(.footnote.weight(.semibold))
            .foregroundStyle(Colors.whiteSolid)
            .padding(.horizontal, .space6)
            .frame(minWidth: .space24, minHeight: .space24)
            .background(Colors.Empty.image)
            .clipShape(RoundedRectangle(cornerRadius: .small))
            .padding(.space8)
    }
}

#Preview {
    GridPosterView(
        model: GridPosterViewModel(
            assetImage: AssetImage(
                imageURL: URL(string: "https://metadata.nftscan.com/eth/0x47a00fc8590c11be4c419d9ae50dec267b6e24ee/0x0000000000000000000000000000000000000000000000000000000000002e5f.png"),
                placeholder: nil,
                chainPlaceholder: nil
            ),
            title: "gemcoder.eth"
        )
    )
    .frame(width: 300, height: 300)
}
