import SwiftUI
import Primitives
import Components
import Style
import Store
import PerpetualService
import Perpetuals
import TransactionsService

public struct PerpetualNavigationView: View {
    @State private var model: PerpetualSceneViewModel
    @Binding var isPresentingTransferData: TransferData?
    @Binding var isPresentingPerpetualRecipientData: PerpetualRecipientData?

    public init(
        asset: Asset,
        wallet: Wallet,
        perpetualService: any PerpetualServiceable,
        transactionsService: TransactionsService,
        observerService: any PerpetualObservable<HyperliquidSubscription>,
        isPresentingTransferData: Binding<TransferData?>,
        isPresentingPerpetualRecipientData: Binding<PerpetualRecipientData?>
    ) {
        _isPresentingTransferData = isPresentingTransferData
        _isPresentingPerpetualRecipientData = isPresentingPerpetualRecipientData
        _model = State(initialValue: PerpetualSceneViewModel(
            wallet: wallet,
            asset: asset,
            perpetualService: perpetualService,
            transactionsService: transactionsService,
            observerService: observerService,
            onTransferData: { isPresentingTransferData.wrappedValue = $0 },
            onPerpetualRecipientData: { isPresentingPerpetualRecipientData.wrappedValue = $0 }
        ))
    }

    public var body: some View {
        PerpetualScene(model: model)
            .sheet(isPresented: $model.isPresentingAutoclose) {
                if let position = model.positions.first {
                    AutocloseNavigationStack(
                        position: position,
                        wallet: model.wallet,
                        onComplete: model.onAutocloseComplete
                    )
                }
            }
            .bindQuery(model.positionsQuery, model.perpetualQuery, model.transactionsQuery, model.perpetualTotalValueQuery)
            // we should ideally observer is isCompleted, but don't have access from here
            .onChange(of: isPresentingTransferData) { _, newValue in
                if newValue == .none {
                    model.fetch()
                }
            }
            .onChange(of: isPresentingPerpetualRecipientData) { oldValue, newValue in
                if newValue == .none {
                    model.fetch()
                }
            }
    }
}
