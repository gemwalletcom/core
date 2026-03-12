// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives
import WalletConnector
import Transfer
import TransactionStateService
import ExplorerService
import Signer
import Style

struct WalletConnectorNavigationStack: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    private let type: WalletConnectorSheetType
    private let presenter: WalletConnectorPresenter

    init(
        type: WalletConnectorSheetType,
        presenter: WalletConnectorPresenter
    ) {
        self.type = type
        self.presenter = presenter
    }

    var body: some View {
        NavigationStack {
            Group {
                switch type {
                case .transferData(let data):
                    ConfirmTransferScene(
                        model: viewModelFactory.confirmTransferScene(
                            wallet: data.payload.wallet,
                            data: data.payload.tranferData,
                            confirmTransferDelegate: data.delegate,
                            simulation: data.payload.simulation,
                            onComplete: { presenter.complete(type: type) }
                        )
                    )
                case .signMessage(let data):
                    SignMessageScene(
                        model: viewModelFactory.signMessageScene(
                            payload: data.payload,
                            confirmTransferDelegate: data.delegate
                        ),
                        onComplete: { presenter.complete(type: type) }
                    )
                case .connectionProposal(let data):
                    ConnectionProposalScene(
                        model: ConnectionProposalViewModel(
                            confirmTransferDelegate: data.delegate,
                            pairingProposal: data.payload
                        ),
                        onComplete: { presenter.complete(type: type) }
                    )
                }
            }
            .interactiveDismissDisabled(true)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("", systemImage: SystemImage.xmark) {
                        presenter.cancelSheet(type: type)
                    }
                }
            }
        }
    }
}
