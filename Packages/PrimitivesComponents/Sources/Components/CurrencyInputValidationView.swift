// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Style
import Components
import Validators

public struct CurrencyInputValidationView: View {
    @Binding private var model: InputValidationViewModel

    private let config: CurrencyInputConfigurable
    private let infoAction: (any Error) -> (() -> Void)?

    public init(
        model: Binding<InputValidationViewModel>,
        config: CurrencyInputConfigurable,
        infoAction: @escaping (any Error) -> (() -> Void)? = { _ in nil }
    ) {
        _model  = model
        self.config = config
        self.infoAction = infoAction
    }

    public var body: some View {
        VStack(spacing: .small) {
            CurrencyInputView(
                text: $model.text,
                config: config
            )

            if let error = model.error, !(error is SilentValidationError) {
                HStack {
                    if let action = infoAction(error) {
                        InfoButton(action: action)
                    }
                    Text(.init(error.localizedDescription))
                        .multilineTextAlignment(.center)
                        .textStyle(TextStyle(font: .footnote, color: Colors.red))
                        .transition(.opacity)
                }
            }
        }
    }
}
