// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import InfoSheet
import Localization
import Perpetuals
import Preferences
import Primitives
import PrimitivesComponents
import Store
import Style
import Validators

@MainActor
@Observable
public final class AmountSceneViewModel {
    private let wallet: Wallet
    private let onTransferAction: TransferDataAction

    private let formatter = ValueFormatter(style: .full)
    private let valueConverter = ValueConverter()
    let currencyFormatter: CurrencyFormatter

    let provider: AmountDataProvider

    public let assetQuery: ObservableQuery<AssetRequest>
    var assetData: AssetData { assetQuery.value }
    var transferState: StateViewType<TransferData> = .noData
    var amountInputModel: InputValidationViewModel
    var isPresentingSheet: AmountSheetType?

    private var amountInputType: AmountInputType = .asset {
        didSet { amountInputModel.update(validators: inputValidators) }
    }

    public init(
        input: AmountInput,
        wallet: Wallet,
        service: AmountService,
        preferences: Preferences = .standard,
        onTransferAction: TransferDataAction
    ) {
        self.wallet = wallet
        self.onTransferAction = onTransferAction
        self.currencyFormatter = CurrencyFormatter(type: .currency, currencyCode: preferences.currency)
        self.provider = .make(from: input, wallet: wallet, service: service)
        self.assetQuery = ObservableQuery(AssetRequest(walletId: wallet.walletId, assetId: input.asset.id), initialValue: .with(asset: input.asset))
        self.amountInputModel = InputValidationViewModel(mode: .onDemand, validators: [])
        amountInputModel.update(validators: inputValidators)

        if let amount = provider.recipientData().amount {
            amountInputModel.update(text: amount)
        }
    }

    var asset: Asset { provider.asset }
    var title: String { provider.title }
    var canChangeValue: Bool { provider.canChangeValue }
    var isInputDisabled: Bool { !canChangeValue }
    var isBalanceViewEnabled: Bool { !isInputDisabled }

    var assetImage: AssetImage { AssetViewModel(asset: asset).assetImage }
    var assetName: String { asset.name }

    var balanceText: String {
        ValueFormatter(style: .medium).string(
            provider.availableValue(from: assetData),
            decimals: asset.decimals.asInt,
            currency: asset.symbol
        )
    }

    var actionButtonState: ButtonState {
        if transferState.isLoading { return .loading() }
        return amountInputModel.text.isNotEmpty && amountInputModel.isValid ? .normal : .disabled
    }

    var infoText: String? {
        guard provider.shouldReserveFee(from: assetData), amountInputModel.text == maxBalance else { return nil }
        return Localized.Transfer.reservedFees(formatter.string(provider.reserveForFee, asset: asset))
    }

    var maxTitle: String { Localized.Transfer.max }
    var continueTitle: String { Localized.Common.continue }
    var isNextEnabled: Bool { actionButtonState == .normal }

    var inputConfig: any CurrencyInputConfigurable {
        AmountInputConfig(
            sceneType: provider.amountType,
            inputType: amountInputType,
            asset: asset,
            currencyFormatter: currencyFormatter,
            numberSanitizer: NumberSanitizer(),
            secondaryText: secondaryText,
            onTapActionButton: onSelectInputButton
        )
    }
}

extension AmountSceneViewModel {
    var shouldFocusOnAppear: Bool { canChangeValue }

    func onAppear() {
        if !canChangeValue {
            setMax()
        }
    }

    func onChangeAssetBalance(_: AssetData, _: AssetData) {
        amountInputModel.update(validators: inputValidators)
    }

    func onSelectNextButton() {
        Task {
            await fetch()
        }
    }

    func onSelectMaxButton() {
        setMax()
    }

    func onSelectInputButton() {
        amountInputType = amountInputType == .asset ? .fiat : .asset
        cleanInput()
    }

    func onSelectReservedFeesInfo() {
        isPresentingSheet = .infoAction(.stakingReservedFees(image: assetImage))
    }

    func onSelectLeverage() {
        guard case let .perpetual(perpetual) = provider,
              let selection = perpetual.leverageSelection else { return }
        isPresentingSheet = .leverageSelector(selection: selection)
    }

    func onSelectAutoclose() {
        guard case let .perpetual(perpetual) = provider else { return }
        let amount = currencyFormatter.double(from: amountInputModel.text) ?? .zero
        isPresentingSheet = .autoclose(perpetual.makeAutocloseData(size: amount))
    }

    func onAutocloseComplete(takeProfit: InputValidationViewModel, stopLoss: InputValidationViewModel) {
        if case let .perpetual(perpetual) = provider {
            perpetual.takeProfit = takeProfit.text.isEmpty ? nil : takeProfit.text
            perpetual.stopLoss = stopLoss.text.isEmpty ? nil : stopLoss.text
        }
        isPresentingSheet = nil
    }

    func onChangeResource(_: Resource, _: Resource) {
        cleanInput()
    }

    func onChangeLeverage(_: LeverageOption, _: LeverageOption) {
        amountInputModel.update(validators: inputValidators)
    }

    func onValidatorSelected(_ validator: DelegationValidator) {
        if case let .stake(stake) = provider {
            stake.validatorSelection.selected = validator
        }
    }

    func infoAction(for error: Error) -> (() -> Void)? {
        guard let transferError = error as? TransferError,
              case .minimumAmount(let asset, let required) = transferError
        else {
            return nil
        }
        return { [weak self] in
            guard let self else { return }
            self.isPresentingSheet = .infoAction(.stakeMinimumAmount(asset, required: required, action: self.onSelectBuy))
        }
    }
}

private extension AmountSceneViewModel {
    func setMax() {
        amountInputType = .asset
        amountInputModel.update(text: maxBalance)
    }

    var maxBalance: String {
        formatter.string(provider.maxValue(from: assetData), decimals: asset.decimals.asInt)
    }

    func cleanInput() {
        amountInputModel.text = .empty
        amountInputModel.update(validators: inputValidators)
    }

    func fetch() async {
        do {
            transferState = .loading
            let value = try formatter.inputNumber(from: amountTransferValue, decimals: asset.decimals.asInt)
            let transfer = try await provider.makeTransferData(value: value)
            transferState = .noData
            onTransferAction?(transfer)
        } catch {
            transferState = .error(error)
            amountInputModel.update(error: error)
        }
    }

    func onSelectBuy() {
        let senderAddress = (try? wallet.account(for: asset.chain).address) ?? ""
        let assetAddress = AssetAddress(asset: asset, address: senderAddress)
        isPresentingSheet = .fiatConnect(assetAddress: assetAddress, walletId: wallet.walletId)
    }

    var inputValidators: [any TextValidator] {
        let source: AmountValidator.Source = switch amountInputType {
        case .asset: .asset
        case .fiat: .fiat(price: assetData.price?.mapToAssetPrice(assetId: asset.id), converter: valueConverter)
        }
        return [
            .amount(
                source: source,
                decimals: asset.decimals.asInt,
                validators: [
                    PositiveValueValidator<BigInt>().silent,
                    MinimumValueValidator<BigInt>(minimumValue: provider.minimumValue, asset: asset),
                    BalanceValueValidator<BigInt>(available: provider.availableValue(from: assetData), asset: asset)
                ]
            )
        ]
    }

    var amountTransferValue: String {
        switch amountInputType {
        case .asset: amountInputModel.text
        case .fiat: amountValue
        }
    }

    var amountValue: String {
        guard let price = assetData.price else { return .zero }
        return (try? valueConverter.convertToAmount(
            fiatValue: amountInputModel.text,
            price: price.mapToAssetPrice(assetId: asset.id),
            decimals: asset.decimals.asInt
        )).or(.zero)
    }

    var fiatValue: Decimal {
        guard let price = assetData.price else { return .zero }
        return (try? valueConverter.convertToFiat(
            amount: amountInputModel.text,
            price: price.mapToAssetPrice(assetId: asset.id)
        )).or(.zero)
    }

    var secondaryText: String {
        switch amountInputType {
        case .asset: currencyFormatter.string(fiatValue.doubleValue)
        case .fiat: [amountValue, asset.symbol].joined(separator: " ")
        }
    }
}
