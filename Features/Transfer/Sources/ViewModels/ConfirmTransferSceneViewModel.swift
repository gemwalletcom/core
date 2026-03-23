// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import BigInt
import Blockchain
import Components
import Localization
import Primitives
import PrimitivesComponents
import WalletConnector
import InfoSheet
import Validators
import SwiftUI
import Swap

@Observable
@MainActor
public final class ConfirmTransferSceneViewModel {
    var feeModel: NetworkFeeSceneViewModel
    var state: StateViewType<TransactionInputViewModel> = .loading {
        didSet {
            onStateChange(state: state)
        }
    }

    var confirmingState: StateViewType<Bool> = .noData {
        didSet {
            if case .error(let error) = confirmingState {
                isPresentingAlertMessage = AlertMessage(
                    title: Localized.Errors.transferError,
                    message: error.localizedDescription
                )
            } else {
                isPresentingAlertMessage = nil
            }
        }
    }

    var isPresentingSheet: ConfirmTransferSheetType?
    var isPresentingAlertMessage: AlertMessage?

    private let confirmService: ConfirmService
    private let simulationService: ConfirmSimulationService

    private let wallet: Wallet
    private let onComplete: VoidAction
    private let confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate?
    private let simulation: SimulationResult?
    private var simulationState: ConfirmSimulationState

    private let transferData: TransferData
    private var metadata: TransferDataMetadata?

    public init(
        wallet: Wallet,
        data: TransferData,
        confirmService: ConfirmService,
        simulationService: ConfirmSimulationService,
        confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate? = .none,
        simulation: SimulationResult? = nil,
        onComplete: VoidAction
    ) {
        self.wallet = wallet
        self.transferData = data
        self.confirmService = confirmService
        self.simulationService = simulationService
        self.confirmTransferDelegate = confirmTransferDelegate
        self.simulation = simulation
        self.onComplete = onComplete

        self.feeModel = NetworkFeeSceneViewModel(
            chain: data.chain,
            priority: confirmService.defaultPriority(for: data.type)
        )

        self.metadata = try? confirmService.getMetadata(wallet: wallet, data: data)
        self.simulationState = simulationService.makeState(data: data, simulation: simulation)
    }

    var title: String { dataModel.title }

    var websiteURL: URL? { dataModel.websiteURL }
    var websiteTitle: String { Localized.Settings.website }

    var senderAddress: String { (try? wallet.account(for: dataModel.chain).address) ?? "" }
    var senderAddressExplorerUrl: URL { senderLink.url }
    var senderExplorerText: String { Localized.Transaction.viewOn(senderLink.name) }

    var progressMessage: String { Localized.Common.loading }

    var simulationWarnings: [SimulationWarning] {
        simulationState.warnings
    }

    var primaryPayloadFields: [SimulationPayloadField] {
        simulationState.primaryFields
    }

    var secondaryPayloadFields: [SimulationPayloadField] {
        simulationState.secondaryFields
    }

    var hasPayloadDetails: Bool {
        simulationState.hasDetails
    }

    var isButtonDisabled: Bool {
        simulationWarnings.contains(where: { $0.severity == .critical })
    }

    var confirmButtonModel: ConfirmButtonViewModel {
        ConfirmButtonViewModel(
            state: state,
            icon: confirmButtonIcon,
            isDisabled: isButtonDisabled,
            onAction: { [weak self] in
                guard let self else { return }
                if case .data(let data) = state, data.isReady {
                    onSelectConfirmTransfer()
                } else {
                    self.fetch()
                }
            }
        )
    }

    var detailsViewModel: ConfirmDetailsViewModel {
        ConfirmDetailsViewModel(type: transferData.type, metadata: metadata)
    }


    private var headerType: TransactionHeaderType {
        if let headerData = simulationState.headerData {
            return .assetValue(headerData)
        }

        if case .tokenApprove(let asset, _) = transferData.type {
            return .asset(image: AssetViewModel(asset: asset).assetImage)
        }

        if case .generic = transferData.type,
           let header = simulation?.header {
            return .asset(image: AssetIdViewModel(assetId: header.assetId).assetImage)
        }

        if let inputModel = state.value {
            return inputModel.headerType
        }

        return TransactionInputViewModel(
            data: transferData,
            transactionData: nil,
            metaData: metadata,
            transferAmount: nil
        ).headerType
    }
}

// MARK: - ListSectionProvideable

extension ConfirmTransferSceneViewModel: ListSectionProvideable {
    public var sections: [ListSection<ConfirmTransferItem>] {
        var result: [ListSection<ConfirmTransferItem>] = []
        result.append(ListSection(type: .header, [.header]))
        let detailItems: [ConfirmTransferItem] = {
            if case .generic = transferData.type {
                return [.app, .sender, .network]
            }
            return [.app, .sender, .network, .recipient, .memo, .details]
        }()
        result.append(ListSection(type: .details, detailItems))

        if !simulationWarnings.isEmpty {
            result.append(ListSection(type: .warnings, [.warnings]))
        }

        if !primaryPayloadFields.isEmpty {
            result.append(ListSection(type: .payload, [.payload]))
        }

        result.append(ListSection(type: .fee, [.networkFee]))
        result.append(ListSection(type: .error, [.error]))
        return result
    }

    public func itemModel(for item: ConfirmTransferItem) -> any ItemModelProvidable<ConfirmTransferItemModel> {
        switch item {
        case .header:
            ConfirmHeaderViewModel(headerType: headerType)
        case .warnings:
            ConfirmTransferItemModel.warnings(simulationWarnings)
        case .app:
            ConfirmAppViewModel(type: transferData.type)
        case .sender:
            ConfirmSenderViewModel(wallet: wallet)
        case .network:
            ConfirmNetworkViewModel(type: transferData.type)
        case .recipient:
            ConfirmRecipientViewModel(
                model: dataModel,
                addressName: try? confirmService.getAddressName(chain: dataModel.chain, address: dataModel.recipient.address),
                addressLink: confirmService.getExplorerLink(chain: dataModel.chain, address: dataModel.recipient.address)
            )
        case .memo:
            ConfirmMemoViewModel(type: transferData.type, recipientData: transferData.recipientData)
        case .details:
            detailsViewModel
        case .payload:
            ConfirmTransferItemModel.payload(primaryPayloadFields)
        case .networkFee:
            ConfirmNetworkFeeViewModel(
                state: state,
                title: feeModel.title,
                value: feeModel.value,
                fiatValue: feeModel.fiatValue,
                infoAction: onSelectNetworkFeeInfo
            )
        case .error:
            ConfirmErrorViewModel(
                state: state,
                onSelectListError: onSelectListError
            )
        }
    }
}

// MARK: - Business Logic

extension ConfirmTransferSceneViewModel {
    func onSelectListError(error: Error) {
        switch error {
        case let error as TransferAmountCalculatorError:
            switch error {
            case let .insufficientBalance(asset):
                isPresentingSheet = .info(.insufficientBalance(asset, image: AssetViewModel(asset: asset).assetImage))
            case let .insufficientNetworkFee(asset, required):
                isPresentingSheet = .info(.insufficientNetworkFee(asset, image: AssetViewModel(asset: asset).assetImage, required: required, action: onSelectBuy))
            case let .minimumAccountBalanceTooLow(asset, required):
                isPresentingSheet = .info(.accountMinimalBalance(asset, required: required))
            }
        case let error as ScanTransactionError:
            switch error {
            case .malicious:
                isPresentingSheet = .info(.maliciousTransaction)
            case let .memoRequired(symbol):
                isPresentingSheet = .info(.memoRequired(symbol: symbol))
            }
        default:
            if let chainError = ChainCoreError.fromError(error) {
                switch chainError {
                case .dustThreshold:
                    let asset = dataModel.asset
                    isPresentingSheet = .info(.dustThreshold(asset.chain, image: AssetViewModel(asset: asset).assetImage))
                case .feeRateMissed, .cantEstimateFee, .incorrectAmount:
                    break
                }
            }
        }
    }

    func onSelectNetworkFeeInfo() {
        isPresentingSheet = .info(.networkFee(dataModel.chain))
    }

    func contextMenuItems(for field: SimulationPayloadField) -> [ContextMenuItemType] {
        var items = payloadFieldViewModel(for: field).contextMenuItems
        if field.fieldType == .address {
            let link = confirmService.getExplorerLink(chain: transferData.chain, address: field.value)
            items.append(.url(title: Localized.Transaction.viewOn(link.name), onOpen: { [weak self] in
                if let url = URL(string: link.link) {
                    self?.isPresentingSheet = .url(url)
                }
            }))
        }
        return items
    }

    func payloadFieldViewModel(for field: SimulationPayloadField) -> SimulationPayloadFieldViewModel {
        SimulationPayloadFieldViewModel(
            field: field,
            chain: transferData.chain,
            addressName: simulationState.addressName(chain: transferData.chain, for: field)
        )
    }

    func onSelectPayloadDetails() {
        isPresentingSheet = .payloadDetails
    }

    func onSelectOpenWebsiteURL() {
        if let websiteURL {
            isPresentingSheet = .url(websiteURL)
        }
    }

    func onSelectOpenSenderAddressURL() {
        isPresentingSheet = .url(senderAddressExplorerUrl)
    }

    func onSelectFeePicker() {
        isPresentingSheet = .networkFeeSelector
    }

    func onSelectSwapDetails() {
        isPresentingSheet = .swapDetails
    }

    func onSelectPerpetualDetails(_ model: PerpetualDetailsViewModel) {
        isPresentingSheet = .perpetualDetails(model)
    }

    func onChangeFeePriority(_: FeePriority) async {
        await fetchData()
    }

    func fetch() {
        Task {
            await fetchData()
        }
    }
}

// MARK: - Private

extension ConfirmTransferSceneViewModel {
    private func fetchData() async {
        state = .loading
        feeModel.reset()
        async let nextSimulationState = simulationService.updateState(
            data: transferData,
            simulation: simulation
        )

        do {
            let metadata = try confirmService.getMetadata(wallet: wallet, data: transferData)
            try TransferAmountCalculator().validateNetworkFee(metadata.feeAvailable, feeAssetId: metadata.feeAssetId)

            let transferTransactionData = try await confirmService.loadTransferTransactionData(
                wallet: wallet, data: transferData,
                priority: feeModel.priority,
                available: metadata.available
            )
            let transferAmount = calculateTransferAmount(
                assetBalance: metadata.assetBalance,
                assetFeeBalance: metadata.assetFeeBalance,
                fee: transferTransactionData.transactionData.fee.fee
            )

            self.simulationState = await nextSimulationState
            self.metadata = metadata
            self.feeModel.update(rates: transferTransactionData.rates)
            self.updateState(
                with: transactionInputViewModel(
                    transferAmount: transferAmount,
                    input: transferTransactionData.transactionData,
                    metaData: metadata
                )
            )
        } catch {
            self.simulationState = await nextSimulationState
            state.setError(error)
            debugLog("preload transaction error: \(error)")
        }
    }

    private func onStateChange(state: StateViewType<TransactionInputViewModel>) {
        switch state {
        case .data(let data):
            if case .failure(let error) = data.transferAmount {
                onSelectListError(error: error)
            }
        case .error(let error as TransferAmountCalculatorError):
            onSelectListError(error: error)
        case .error(let error as ScanTransactionError):
            onSelectListError(error: error)
        case .error, .loading, .noData:
            break
        }
    }

    private func onSelectBuy() {
        isPresentingSheet = .fiatConnect(
            assetAddress: feeAssetAddress,
            walletId: wallet.walletId
        )
    }
    private func onSelectConfirmTransfer() {
        guard let value = state.value,
              let transactionData = value.transactionData,
              case .success(let amount) = value.transferAmount
        else { return }
        confirmTransfer(transactionData: transactionData, amount: amount)
    }

    private func confirmTransfer(
        transactionData: TransactionData,
        amount: TransferAmount
    ) {
        Task {
            await processConfirmation(
                transactionData: transactionData,
                amount: amount
            )
            if case .data(_) = confirmingState {
                onComplete?()
            }
        }
    }

    private func processConfirmation(transactionData: TransactionData, amount: TransferAmount) async {
        confirmingState = .loading
        do {
            let input = TransferConfirmationInput(
                data: state.value!.data,
                wallet: wallet,
                transactionData: transactionData,
                amount: amount,
                delegate: confirmTransferDelegate
            )
            try await confirmService.executeTransfer(input: input)
            if let data = input.data.type.recentActivityData {
                confirmService.updateRecent(data: data, walletId: wallet.walletId)
            }
            confirmingState = .data(true)
        } catch {
            confirmingState = .error(error)
            debugLog("confirm transaction error: \(error)")
        }
    }

    private func updateState(with model: TransactionInputViewModel) {
        feeModel.update(
            value: model.networkFeeText,
            fiatValue: model.networkFeeFiatText
        )
        state = .data(model)
    }

    private func calculateTransferAmount(
        assetBalance: Balance,
        assetFeeBalance: Balance,
        fee: BigInt
    ) -> TransferAmountValidation {
        TransferAmountCalculator().validate(input: TransferAmountInput(
            asset: dataModel.asset,
            assetBalance: assetBalance,
            value: dataModel.data.value,
            availableValue: availableValue,
            assetFee: dataModel.asset.feeAsset,
            assetFeeBalance: assetFeeBalance,
            fee: fee,
            transferData: transferData
        ))
    }

    private func transactionInputViewModel(
        transferAmount: TransferAmountValidation,
        input: TransactionData? = nil,
        metaData: TransferDataMetadata? = nil
    ) -> TransactionInputViewModel {
        TransactionInputViewModel(
            data: transferData,
            transactionData: input,
            metaData: metaData,
            transferAmount: transferAmount
        )
    }

    private var availableValue: BigInt { dataModel.availableValue(metadata: metadata) }
    private var senderLink: BlockExplorerLink { confirmService.getExplorerLink(chain: dataModel.chain, address: senderAddress) }
    private var feeAssetAddress: AssetAddress { AssetAddress(asset: dataModel.asset.feeAsset, address: senderAddress)}
    private var confirmButtonIcon: Image? {
        guard !state.isError, state.value?.transferAmount?.isSuccess ?? false,
              let auth = try? confirmService.getPasswordAuthentication(),
              let systemName = KeystoreAuthenticationViewModel(authentication: auth).authenticationImage
        else { return nil }
        return Image(systemName: systemName)
    }

    private var dataModel: TransferDataViewModel { TransferDataViewModel(data: transferData) }
}
