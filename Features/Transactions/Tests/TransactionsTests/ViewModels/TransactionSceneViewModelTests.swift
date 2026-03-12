import Testing
import Foundation
import Localization
import Primitives
import PrimitivesTestKit
import Preferences
import PreferencesTestKit
import PrimitivesComponents
import Style
import Components

@testable import Transactions
@testable import Store

@MainActor
struct TransactionSceneViewModelTests {

    @Test
    func itemModelReturnsNonEmpty() {
        let model = TransactionSceneViewModel.mock()

        verifyNonEmpty(model.item(for: TransactionItem.header))
        verifyNonEmpty(model.item(for: TransactionItem.date))
        verifyNonEmpty(model.item(for: TransactionItem.status))
        verifyNonEmpty(model.item(for: TransactionItem.network))
        verifyNonEmpty(model.item(for: TransactionItem.fee))
        verifyNonEmpty(model.item(for: TransactionItem.explorerLink))
    }

    @Test
    func headerItemModel() {
        let model = TransactionSceneViewModel.mock(
            type: TransactionType.transfer,
            direction: TransactionDirection.outgoing
        )
        let itemModel = model.item(for: TransactionItem.header)

        verifyNonEmpty(itemModel)
    }

    @Test
    func swapButtonItemModel() {
        let swapModel = TransactionSceneViewModel.mock(type: TransactionType.swap, state: TransactionState.confirmed)
        let swapItem = swapModel.item(for: TransactionItem.swapButton)

        if case .empty = swapItem {
        } else if case .swapAgain = swapItem {
        } else {
            Issue.record("Unexpected swap button model type")
        }

        let transferModel = TransactionSceneViewModel.mock(type: TransactionType.transfer)
        let transferItem = transferModel.item(for: TransactionItem.swapButton)

        if case .empty = transferItem {
        } else {
            Issue.record("Expected empty for non-swap transaction")
        }
    }

    @Test
    func dateItemModel() {
        let testDate = Date(timeIntervalSince1970: 1609459200)
        let model = TransactionSceneViewModel.mock(createdAt: testDate)

        if case .listItem(let item) = model.item(for: TransactionItem.date) {
            #expect(item.title == Localized.Transaction.date)
            #expect(item.subtitle != nil)
        } else {
            Issue.record("Expected listItem for date")
        }
    }

    @Test
    func statusItemModel() {
        let confirmedModel = TransactionSceneViewModel.mock(state: TransactionState.confirmed)
        if case .listItem(let item) = confirmedModel.item(for: TransactionItem.status) {
            #expect(item.title == Localized.Transaction.status)
            #expect(item.subtitleStyle.color == Colors.green)
        } else {
            Issue.record("Expected listItem for confirmed status")
        }

        let pendingModel = TransactionSceneViewModel.mock(state: TransactionState.pending)
        if case .listItem(let item) = pendingModel.item(for: TransactionItem.status) {
            if case .progressView = item.titleTagType {
            } else {
                Issue.record("Expected progress indicator for pending status")
            }
            #expect(item.subtitleStyle.color == Colors.orange)
        } else {
            Issue.record("Expected listItem for pending status")
        }
    }

    @Test
    func participantItemModel() {
        let transaction = TransactionExtended.mock(
            transaction: Transaction.mock(
                type: .transfer,
                direction: .incoming,
                from: "0xSenderAddress",
                to: "0xRecipientAddress"
            )
        )
        let modelWithAddresses = TransactionSceneViewModel(
            transaction: transaction,
            walletId: .mock(),
            preferences: Preferences.standard
        )

        if case .participant(let item) = modelWithAddresses.item(for: TransactionItem.participant) {
            #expect(item.title == Localized.Transaction.sender)
            #expect(item.account.address == "0xSenderAddress")
        } else {
            Issue.record("Expected participant item for incoming transfer")
        }

        let swapModel = TransactionSceneViewModel.mock(type: TransactionType.swap)
        if case .empty = swapModel.item(for: TransactionItem.participant) {
        } else {
            Issue.record("Expected empty for swap participant")
        }
    }

    @Test
    func memoItemModel() {
        let modelWithMemo = TransactionSceneViewModel.mock(assetId: .mock(.cosmos), memo: "Test memo")
        if case .listItem(let item) = modelWithMemo.item(for: TransactionItem.memo) {
            #expect(item.title == Localized.Transfer.memo)
            #expect(item.subtitle == "Test memo")
        } else {
            Issue.record("Expected listItem for memo")
        }

        let modelNoMemo = TransactionSceneViewModel.mock(assetId: .mock(.cosmos), memo: nil)
        if case .empty = modelNoMemo.item(for: TransactionItem.memo) {
        } else {
            Issue.record("Expected empty for nil memo")
        }

        let modelEmptyMemo = TransactionSceneViewModel.mock(assetId: .mock(.cosmos), memo: "")
        if case .empty = modelEmptyMemo.item(for: TransactionItem.memo) {
        } else {
            Issue.record("Expected empty for empty memo")
        }
    }

    @Test
    func networkItemModel() {
        let model = TransactionSceneViewModel.mock()

        if case .network(let title, let subtitle, _) = model.item(for: TransactionItem.network) {
            #expect(title == Localized.Transfer.network)
            #expect(subtitle == "Bitcoin")
        } else {
            Issue.record("Expected network item for network")
        }
    }

    @Test
    func providerItemModel() {
        let model = TransactionSceneViewModel.mock()
        if case .empty = model.item(for: TransactionItem.provider) {
        } else {
            Issue.record("Expected empty for provider")
        }
    }

    @Test
    func feeItemModel() {
        let model = TransactionSceneViewModel.mock()

        if case .fee(let item) = model.item(for: TransactionItem.fee) {
            #expect(item.title == Localized.Transfer.networkFee)
            #expect(item.infoAction != nil)
        } else {
            Issue.record("Expected listItem for fee")
        }
    }

    @Test
    func explorerLinkItemModel() {
        let model = TransactionSceneViewModel.mock()

        if case .explorer(let url, let text) = model.item(for: TransactionItem.explorerLink) {
            #expect(url.absoluteString == "https://blockchair.com/bitcoin/transaction/1")
            #expect(text == "View on Blockchair")
        } else {
            Issue.record("Expected explorer item for explorer link")
        }
    }

    @Test
    func sectionsStructure() {
        let model = TransactionSceneViewModel.mock()
        let sections = model.sections

        #expect(sections.count == 4)
        #expect(sections[0].id == "header")
        #expect(sections[1].id == "swapAction")
        #expect(sections[2].id == "details")
        #expect(sections[3].id == "explorer")

        #expect(sections[0].values == [TransactionItem.header])
        #expect(sections[1].values == [TransactionItem.swapButton])
        #expect(sections[2].values == [
            TransactionItem.date,
            TransactionItem.status,
            TransactionItem.participant,
            TransactionItem.memo,
            TransactionItem.network,
            TransactionItem.pnl,
            TransactionItem.price,
            TransactionItem.provider,
            TransactionItem.fee
        ])
        #expect(sections[3].values == [TransactionItem.explorerLink])
    }

    private func verifyNonEmpty(_ model: TransactionItemModel) {
        if case .empty = model {
            Issue.record("Expected non-empty model")
        }
    }
}

extension TransactionSceneViewModel {
    static func mock(
        type: TransactionType = .transfer,
        state: TransactionState = .confirmed,
        direction: TransactionDirection = .outgoing,
        assetId: AssetId = .mock(),
        toAddress: String = "participant_address",
        memo: String? = nil,
        createdAt: Date = Date()
    ) -> TransactionSceneViewModel {
        TransactionSceneViewModel(
            transaction: TransactionExtended.mock(
                transaction: Transaction.mock(
                    type: type,
                    state: state,
                    direction: direction,
                    assetId: assetId,
                    to: toAddress,
                    memo: memo
                )
            ),
            walletId: .mock(),
            preferences: Preferences.standard
        )
    }
}
