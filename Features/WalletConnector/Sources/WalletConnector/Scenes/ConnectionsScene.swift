// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Store
import Primitives
import Components
import QRScanner
import Style
import PrimitivesComponents
import Localization

public struct ConnectionsScene: View {
    @State private var model: ConnectionsViewModel

    public init(model: ConnectionsViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            Section {
                ButtonListItem(
                    title: model.pasteButtonTitle,
                    image: Images.System.paste,
                    action: model.onPaste
                )
                ButtonListItem(
                    title: model.scanQRCodeButtonTitle,
                    image: Images.System.qrCodeViewfinder,
                    action: model.onScan
                )
            }
            
            ForEach(model.sections) { section in
                Section(section.title.or(.empty)) {
                    ForEach(section.values) { connection in
                        NavigationLink(value: connection) {
                            ConnectionView(model: WalletConnectionViewModel(connection: connection))
                                .swipeActions(edge: .trailing) {
                                    Button(
                                        model.disconnectTitle,
                                        role: .destructive,
                                        action: { model.onSelectDisconnect(connection) }
                                    )
                                    .tint(Colors.red)
                                }
                        }
                    }
                }
            }
        }
        .bindQuery(model.query)
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .overlay {
            if model.sections.isEmpty {
                EmptyContentView(model: model.emptyContentModel)
                    .padding(.horizontal, .medium)
            }
        }
        .navigationDestination(for: WalletConnection.self) { connection in
            ConnectionScene(model: model.connectionSceneModel(connection: connection))
        }
        .sheet(isPresented: $model.isPresentingScanner) {
            ScanQRCodeNavigationStack(action: model.onHandleScan(_:))
        }
        .toolbarInfoButton(url: model.docsUrl)
        .alertSheet($model.isPresentingAlertMessage)
        .toast(
            isPresenting: $model.isPresentingConnectorBar,
            message: ToastMessage(
                title: "\(Localized.WalletConnect.brandName)...",
                image: SystemImage.network
            ),
            duration: .infinity,
            tapToDismiss: false
        )
        .navigationTitle(model.title)
        .taskOnce { model.fetch() }
        .onChange(of: model.walletConnectorPresenter?.isPresentingSheet?.id, model.hideConnectionBar)
    }
}
