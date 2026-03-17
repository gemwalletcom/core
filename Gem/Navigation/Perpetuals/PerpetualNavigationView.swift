import SwiftUI
import Primitives
import Components
import Style
import Store
import PerpetualService
import Perpetuals
import WalletTab
import TransactionsService

public struct PerpetualNavigationView: View {
    @State private var model: PerpetualSceneViewModel
    @Binding var isPresentingSheet: WalletSheetType?

    public init(
        asset: Asset,
        wallet: Wallet,
        perpetualService: any PerpetualServiceable,
        transactionsService: TransactionsService,
        observerService: any PerpetualObservable<HyperliquidSubscription>,
        isPresentingSheet: Binding<WalletSheetType?>
    ) {
        _isPresentingSheet = isPresentingSheet
        _model = State(initialValue: PerpetualSceneViewModel(
            wallet: wallet,
            asset: asset,
            perpetualService: perpetualService,
            transactionsService: transactionsService,
            observerService: observerService,
            onTransferData: { isPresentingSheet.wrappedValue = .transferData($0) },
            onPerpetualRecipientData: { isPresentingSheet.wrappedValue = .perpetualRecipientData($0) }
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
            .onChange(of: isPresentingSheet) { oldValue, newValue in
                guard newValue == nil else { return }
                switch oldValue {
                case .transferData, .perpetualRecipientData:
                    model.fetch()
                default:
                    break
                }
            }
    }
}
