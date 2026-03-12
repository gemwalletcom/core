// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Style
import Localization
import PrimitivesComponents
import Primitives

public struct SignMessageScene: View {
    @State private var model: SignMessageSceneViewModel
    private let onComplete: () -> Void

    public init(
        model: SignMessageSceneViewModel,
        onComplete: @escaping () -> Void
    ) {
        _model = State(wrappedValue: model)
        self.onComplete = onComplete
    }

    public var body: some View {
        List {
            Section {
                ListItemImageView(
                    title: Localized.WalletConnect.app,
                    subtitle: model.appText,
                    assetImage: model.appAssetImage
                )
                .contextMenu(
                    .url(title: Localized.WalletConnect.website, onOpen: model.onViewWebsite)
                )
                ListItemImageView(
                    title: Localized.Common.wallet,
                    subtitle: model.walletText,
                    assetImage: model.walletAssetImage
                )
                ListItemImageView(
                    title: Localized.Transfer.network,
                    subtitle: model.networkText,
                    assetImage: model.networkAssetImage
                )
            }

            if model.hasWarnings {
                Section {
                    SimulationWarningsContent(warnings: model.simulationWarnings)
                }
            }

            if model.hasPayload {
                Section {
                    SimulationPayloadFieldsContent(
                        fields: model.primaryPayloadFields,
                        fieldViewModel: model.payloadFieldViewModel(for:),
                        contextMenuItems: model.contextMenuItems(for:)
                    )

                    NavigationCustomLink(with: ListItemView(title: Localized.Common.details)) {
                        model.onViewPayloadDetails()
                    }
                }
            } else if case .text(let string) = model.messageDisplayType {
                Section(Localized.SignMessage.message) {
                    Text(string)
                }
            }
        }
        .listSectionSpacing(.compact)
        .taskOnce { model.fetch() }
        .safeAreaButton {
            StateButton(
                text: model.buttonTitle,
                type: model.buttonType,
                action: sign
            )
        }
        .navigationTitle(model.title)
        .safariSheet(url: $model.isPresentingUrl)
        .sheet(isPresented: $model.isPresentingPayloadDetails) {
            if model.hasPayload {
                NavigationStack {
                    SimulationPayloadDetailsScene(
                        primaryFields: model.primaryPayloadFields,
                        secondaryFields: model.secondaryPayloadFields,
                        fieldViewModel: model.payloadFieldViewModel(for:),
                        contextMenuItems: model.contextMenuItems(for:),
                        actionTitle: Localized.SignMessage.viewFullMessage,
                        actionDestination: AnyView(TextMessageScene(model: model.textMessageViewModel))
                    )
                    .presentationDetents([.large])
                    .presentationBackground(Colors.grayBackground)
                }
            }
        }
    }

    func sign() {
        Task {
            do {
                try await model.signMessage()
                onComplete()
            } catch {
                debugLog("sign message error \(error)")
            }
        }
    }
}
