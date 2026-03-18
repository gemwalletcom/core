// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import PrimitivesComponents
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
                AssetPreviewView(model: model.appPreview, subtitleLayout: .vertical)
                    .frame(maxWidth: .infinity)
                    .padding(.bottom, .small)
            }
            .cleanListRow()

            Section {
                NavigationLink(value: Scenes.SelectWallet()) {
                    ListItemView(
                        title: model.walletTitle,
                        subtitle: model.walletName
                    )
                }
                ListItemView(
                    title: model.connectionTitle,
                    subtitle: model.connectionText
                )
                ListItemImageView(
                    title: Localized.Transaction.status,
                    subtitle: model.statusText,
                    subtitleStyle: model.statusTextStyle,
                    assetImage: model.statusAssetImage
                )
            }

            Section(model.permissionsTitle) {
                ForEach(model.permissions, id: \.title) { permission in
                    ListItemView(model: permission)
                }
            }
        }
        .safeAreaButton {
            StateButton(
                text: model.buttonTitle,
                action: onAccept
            )
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
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
