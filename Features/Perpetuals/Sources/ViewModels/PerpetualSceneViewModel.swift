// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import InfoSheet
import Localization
import PerpetualService
import Primitives
import PrimitivesComponents
import Store
import SwiftUI
import Formatters
import ExplorerService
import Preferences
import BigInt
import GemstonePrimitives
import TransactionsService

@Observable
@MainActor
public final class PerpetualSceneViewModel {
    private let perpetualService: PerpetualServiceable
    private let observerService: any PerpetualObservable<HyperliquidSubscription>
    private let transactionsService: TransactionsService
    private let onTransferData: TransferDataAction
    private let onPerpetualRecipientData: ((PerpetualRecipientData) -> Void)?
    private let perpetualOrderFactory = PerpetualOrderFactory()

    public let wallet: Wallet
    public let asset: Asset

    public let explorerService: any ExplorerLinkFetchable = ExplorerService.standard

    public let positionsQuery: ObservableQuery<PerpetualPositionsRequest>
    public let perpetualQuery: ObservableQuery<PerpetualRequest>
    public let perpetualTotalValueQuery: ObservableQuery<TotalValueRequest>
    public let transactionsQuery: ObservableQuery<TransactionsRequest>

    public var positions: [PerpetualPositionData] { positionsQuery.value }
    public var perpetualData: PerpetualData { perpetualQuery.value }
    public var perpetualTotalValue: TotalFiatValue { perpetualTotalValueQuery.value }
    public var transactions: [TransactionExtended] { transactionsQuery.value }

    public var state: StateViewType<[ChartCandleStick]> = .loading
    public var currentPeriod: ChartPeriod = .day

    public var isPresentingInfoSheet: InfoSheetType?
    public var isPresentingModifyAlert: Bool?
    public var isPresentingAutoclose: Bool = false

    let preference = Preferences.standard

    private var observeTask: Task<Void, Never>?

    public init(
        wallet: Wallet,
        asset: Asset,
        perpetualService: PerpetualServiceable,
        transactionsService: TransactionsService,
        observerService: any PerpetualObservable<HyperliquidSubscription>,
        onTransferData: TransferDataAction = nil,
        onPerpetualRecipientData: ((PerpetualRecipientData) -> Void)? = nil
    ) {
        self.wallet = wallet
        self.asset = asset
        self.perpetualService = perpetualService
        self.transactionsService = transactionsService
        self.observerService = observerService
        self.onTransferData = onTransferData
        self.onPerpetualRecipientData = onPerpetualRecipientData

        self.positionsQuery = ObservableQuery(PerpetualPositionsRequest(walletId: wallet.walletId, filter: .assetId(asset.id)), initialValue: [])
        self.perpetualQuery = ObservableQuery(PerpetualRequest(assetId: asset.id), initialValue: .empty)
        self.perpetualTotalValueQuery = ObservableQuery(TotalValueRequest(walletId: wallet.walletId, balanceType: .perpetual), initialValue: .zero)
        self.transactionsQuery = ObservableQuery(
            TransactionsRequest.perpetualScene(
                walletId: wallet.walletId,
                assetId: asset.id,
                limit: 50
            ),
            initialValue: []
        )
    }

    public var navigationTitle: String {
        let name = perpetualViewModel.name
        return name.isEmpty ? asset.symbol : name
    }
    public var currency: String { preference.currency }
    public var hasOpenPosition: Bool { !positionViewModels.isEmpty }

    public var positionSectionTitle: String { Localized.Perpetual.position }
    public var infoSectionTitle: String { Localized.Common.info }
    public var transactionsSectionTitle: String { Localized.Activity.title }
    public var closePositionTitle: String { Localized.Perpetual.closePosition }
    public var modifyPositionTitle: String { Localized.Perpetual.modify }
    public var increasePositionTitle: String { Localized.Perpetual.increasePosition }
    public var reducePositionTitle: String { Localized.Perpetual.reducePosition }
    public var longButtonTitle: String { Localized.Perpetual.long }
    public var shortButtonTitle: String { Localized.Perpetual.short }

    public var perpetual: Perpetual { perpetualData.perpetual }
    public var perpetualViewModel: PerpetualViewModel { PerpetualViewModel(perpetual: perpetual) }
    public var positionViewModels: [PerpetualPositionViewModel] { positions.map { PerpetualPositionViewModel($0) } }

    var chartLineModels: [ChartLineViewModel] {
        guard let positionData = positions.first else { return [] }
        let position = positionData.position
        let prices: [(ChartLineType, Double?)] = [
            (.entry, position.entryPrice),
            (.takeProfit, position.takeProfit?.price),
            (.stopLoss, position.stopLoss?.price),
            (.liquidation, position.liquidationPrice)
        ]
        return prices.compactMap { type, price in
            price.map {
                ChartLineViewModel(
                    line: ChartLine(type: type, price: $0),
                    formatter: CurrencyFormatter(type: .currency, currencyCode: .empty)
                )
            }
        }
    }

    private var currentChartSubscription: ChartSubscription { ChartSubscription(coin: perpetual.name, period: currentPeriod) }
}

// MARK: - Actions

public extension PerpetualSceneViewModel {
    func fetch() {
        Task { await observerService.update(for: wallet) }
        Task { try await perpetualService.updateMarket(symbol: perpetual.name) }
        Task { await fetchTransactions() }
        Task { await updateCandlesticks() }
    }

    func onAppear() async {
        fetch()
        await subscribeCandles(currentChartSubscription)
        observeTask = Task {
            await observeCandles()
        }
    }

    func onDisappear() async {
        observeTask?.cancel()
        observeTask = nil
        await unsubscribeCandles(currentChartSubscription)
    }

    func onScenePhaseChange(_ oldPhase: ScenePhase, _ newPhase: ScenePhase) {
        switch newPhase {
        case .active:
            Task { try? await perpetualService.updateMarket(symbol: perpetual.name) }
            Task { await fetchTransactions() }
            Task { await updateCandlesticks() }
        case .inactive, .background: break
        @unknown default: break
        }
    }

    func onPeriodChange(_ oldPeriod: ChartPeriod, _ newPeriod: ChartPeriod) {
        Task {
            await unsubscribeCandles(ChartSubscription(coin: perpetual.name, period: oldPeriod))
            await updateCandlesticks()
            await subscribeCandles(ChartSubscription(coin: perpetual.name, period: newPeriod))
        }
    }

    func onSelectFundingRateInfo() {
        isPresentingInfoSheet = .fundingRate
    }

    func onSelectFundingPaymentsInfo() {
        isPresentingInfoSheet = .fundingPayments
    }

    func onSelectLiquidationPriceInfo() {
        isPresentingInfoSheet = .liquidationPrice
    }

    func onSelectOpenInterestInfo() {
        isPresentingInfoSheet = .openInterest
    }

    func onSelectAutoclose() {
        isPresentingAutoclose = true
    }

    func onSelectAutocloseInfo() {
        isPresentingInfoSheet = .autoclose
    }

    func onModifyPosition() {
        isPresentingModifyAlert = true
    }

    func onClosePosition() {
        guard
            let position = positions.first?.position,
            let assetIndex = UInt32(perpetual.identifier)
        else { return }

        let data = perpetualOrderFactory.makeCloseOrder(
            assetIndex: Int32(assetIndex),
            perpetual: perpetual,
            position: position,
            asset: asset,
            baseAsset: .hypercoreUSDC()
        )

        let transferData = TransferData(
            type: .perpetual(asset, .close(data)),
            recipientData: .hyperliquid(),
            value: .zero,
            canChangeValue: false
        )

        onTransferData?(transferData)
    }

    func onOpenLongPosition() {
        guard let transferData = createTransferData(
            direction: .long,
            leverage: perpetual.maxLeverage
        ) else {
            return
        }
        onPositionAction(.open(transferData))
    }

    func onOpenShortPosition() {
        guard let transferData = createTransferData(
            direction: .short,
            leverage: perpetual.maxLeverage
        ) else {
            return
        }
        onPositionAction(.open(transferData))
    }

    func onIncreasePosition() {
        isPresentingModifyAlert = false

        guard let position = positions.first?.position,
              let transferData = createTransferData(direction: position.direction, leverage: position.leverage)
        else { return }

        onPositionAction(.increase(transferData))
    }

    func onReducePosition() {
        isPresentingModifyAlert = false

        guard let position = positions.first?.position else {
            return
        }

        let direction: PerpetualDirection = {
            switch position.direction {
            case .long: .short
            case .short: .long
            }
        }()

        guard let transferData = createTransferData(direction: direction, leverage: position.leverage) else {
            return
        }

        onPositionAction(
            .reduce(
                transferData,
                available: BigInt(position.marginAmount * pow(10.0, Double(position.baseAsset.decimals))),
                positionDirection: position.direction
            )
        )
    }

    func onAutocloseComplete() {
        isPresentingAutoclose = false
    }
}

// MARK: - Private

private extension PerpetualSceneViewModel {
    func updateCandlesticks() async {
        state = .loading
        do {
            let candlesticks = try await perpetualService.candlesticks(
                symbol: perpetual.name,
                period: currentPeriod
            )
            state = .data(candlesticks)
        } catch {
            state = .error(error)
        }
    }

    func subscribeCandles(_ subscription: ChartSubscription) async {
        do {
            try await observerService.subscribe(.candle(subscription))
        } catch {
            debugLog("Chart subscription failed: \(error)")
        }
    }

    func unsubscribeCandles(_ subscription: ChartSubscription) async {
        do {
            try await observerService.unsubscribe(.candle(subscription))
        } catch {
            debugLog("Chart unsubscribe failed: \(error)")
        }
    }

    func observeCandles() async {
        for await update in await observerService.chartService.makeStream() {
            if Task.isCancelled { break }
            handleChartUpdate(update)
        }
    }

    func handleChartUpdate(_ update: ChartCandleUpdate) {
        guard update.coin == currentChartSubscription.coin,
              update.interval == currentChartSubscription.interval,
              case .data(var candlesticks) = state,
              let lastCandle = candlesticks.last
        else {
            return
        }

        let candle = update.candle
        if lastCandle.date == candle.date {
            candlesticks[candlesticks.count - 1] = candle
        } else if candle.date > lastCandle.date {
            candlesticks.removeFirst()
            candlesticks.append(candle)
        }

        state = .data(candlesticks)
    }

    func createTransferData(direction: PerpetualDirection, leverage: UInt8) -> PerpetualTransferData? {
        guard let assetIndex = Int(perpetual.identifier) else {
            return nil
        }

        return PerpetualTransferData(
            provider: perpetual.provider,
            direction: direction,
            asset: asset,
            baseAsset: .hypercoreUSDC(),
            assetIndex: assetIndex,
            price: perpetual.price,
            leverage: leverage
        )
    }

    func onPositionAction(_ positionAction: PerpetualPositionAction) {
        let recipientData = PerpetualRecipientData(
            recipient: .hyperliquid(),
            positionAction: positionAction
        )
        onPerpetualRecipientData?(recipientData)
    }

    func fetchTransactions() async {
        do {
            try await transactionsService.updateForAsset(wallet: wallet, assetId: asset.id)
        } catch {
            debugLog("perpetual scene: fetchTransactions error \(error)")
        }
    }
}

public extension RecipientData {
    static func hyperliquid() -> RecipientData {
        RecipientData(
            recipient: Recipient(name: "Hyperliquid", address: "", memo: .none),
            amount: .none
        )
    }
}
