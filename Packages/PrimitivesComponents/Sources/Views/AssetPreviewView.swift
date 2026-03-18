// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Style

public protocol AssetPreviewable {
    var assetImage: AssetImage { get }
    var name: String { get }
    var subtitleSymbol: String? { get }
}

public struct AssetPreviewView<Model: AssetPreviewable>: View {
    public enum SubtitleLayout {
        case horizontal
        case vertical
    }

    private let model: Model
    private let subtitleLayout: SubtitleLayout

    public init(model: Model, subtitleLayout: SubtitleLayout = .horizontal) {
        self.model = model
        self.subtitleLayout = subtitleLayout
    }

    public var body: some View {
        VStack(spacing: .medium) {
            AssetImageView(assetImage: model.assetImage, size: .image.semiLarge)

            layout {
                Text(model.name)
                    .textStyle(.headline)
                    .lineLimit(1)
                if let symbol = model.subtitleSymbol {
                    Text(symbol)
                        .textStyle(TextStyle(font: .subheadline, color: Colors.secondaryText, fontWeight: .medium))
                        .lineLimit(1)
                }
            }
        }
    }
    
    private var layout: AnyLayout {
        switch subtitleLayout {
        case .horizontal: AnyLayout(HStackLayout(alignment: .lastTextBaseline, spacing: .tiny))
        case .vertical: AnyLayout(VStackLayout(spacing: .tiny))
        }
    }
}
