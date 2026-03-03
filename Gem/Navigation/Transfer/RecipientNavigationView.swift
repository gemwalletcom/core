// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Primitives
import Transfer
import QRScanner

struct RecipientNavigationView: View {
    @Environment(\.viewModelFactory) private var viewModelFactory
    @State private var model: RecipientSceneViewModel

    init(model: RecipientSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        RecipientScene(
            model: model
        )
        .sheet(item: $model.isPresentingScanner) { value in
            ScanQRCodeNavigationStack() {
                model.onHandleScan($0, for: value)
            }
        }
        .navigationDestination(for: RecipientData.self) { data in
            AmountNavigationView(
                model: viewModelFactory.amountScene(
                    input: AmountInput(type: .transfer(recipient: data), asset: model.asset),
                    wallet: model.wallet,
                    onTransferAction: model.onTransferAction
                )
            )
        }
    }
}
