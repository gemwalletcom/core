// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Style
import Primitives
import Localization

public struct ConnectionProposalScene: View {
    @State private var model: ConnectionProposalViewModel
    private let onComplete: () -> Void

    public init(
        model: ConnectionProposalViewModel,
        onComplete: @escaping () -> Void
    ) {
        _model = State(initialValue: model)
        self.onComplete = onComplete
    }

    public var body: some View {
        List {
            Section { } header: {
                VStack(alignment: .center) {
                    AsyncImageView(url: model.imageUrl, size: .image.semiLarge)
                }
                .padding(.top, .small)
            }
            .cleanListRow()

            Section {
                NavigationLink(value: Scenes.SelectWallet()) {
                    ListItemView(
                        title: model.walletTitle,
                        subtitle: model.walletName
                    )
                }
                ListItemView(title: model.appTitle, subtitle: model.appText)
                ListItemImageView(
                    title: Localized.WalletConnect.Connection.title,
                    subtitle: model.statusText,
                    subtitleStyle: model.statusTextStyle,
                    assetImage: model.statusAssetImage
                )
            }
        }
        .safeAreaButton {
            StateButton(
                text: model.buttonTitle,
                action: onAccept
            )
        }
        .navigationTitle(model.title)
        .navigationDestination(for: Scenes.SelectWallet.self) { _ in
            SelectWalletScene(model: $model.walletSelectorModel)
        }
    }
}

// MARK: - Actions

extension ConnectionProposalScene {
    private func onAccept() {
        do {
            try model.accept()
            onComplete()
        } catch {
            debugLog("accept proposal error \(error)")
        }
    }
}
