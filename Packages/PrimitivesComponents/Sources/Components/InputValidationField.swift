// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Style
import Primitives

public struct InputValidationField<TrailingView: View>: View {
    @Binding private var model: InputValidationViewModel

    let style: FloatFieldStyle
    let placeholder: String
    let allowClean: Bool
    let onClean: VoidAction

    private let trailingView: TrailingView

    public init(
        model: Binding<InputValidationViewModel>,
        style: FloatFieldStyle = .standard,
        placeholder: String,
        allowClean: Bool = true,
        onClean: VoidAction = nil,
        @ViewBuilder trailingView: () -> TrailingView = { EmptyView() }
    ) {
        self.trailingView = trailingView()
        _model = model
        self.style = style
        self.placeholder = placeholder
        self.allowClean = allowClean
        self.onClean = onClean
    }

    public var body: some View {
        Group {
            FloatTextField(
                placeholder,
                text: $model.text,
                style: style,
                allowClean: allowClean,
                onClean: onClean,
                trailingView: {
                    trailingView
                }
            )

            if let message = model.error?.localizedDescription {
                Text(.init(message))
                    .textStyle(TextStyle(font: .footnote, color: Colors.red))
                    .transition(.opacity)
            }
        }
    }
}
