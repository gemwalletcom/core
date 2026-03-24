// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing
import GemAPI
import Primitives
import PrimitivesTestKit
import Formatters
import BigInt
import BalanceServiceTestKit

@testable import FiatConnect

@MainActor
final class FiatSceneViewModelTests {
    private static func mock(
        service: any GemAPIFiatService = GemAPIService(),
        currencyFormatter: CurrencyFormatter = .init(locale: Locale.US, currencyCode: Currency.usd.rawValue),
        assetAddress: AssetAddress = .mock(),
        wallet: Wallet = .mock()
    ) -> FiatSceneViewModel {
        FiatSceneViewModel(
            fiatService: service,
            currencyFormatter: currencyFormatter,
            assetAddress: assetAddress,
            wallet: wallet,
            assetsEnabler: .mock()
        )
    }

    @Test
    func testDefaultAmountText() {
        let model = FiatSceneViewModelTests.mock()
        #expect(model.inputValidationModel.text == "50")

        model.type = .sell
        model.onChangeType(oldType: .buy, newType: .sell)

        #expect(model.inputValidationModel.text == "100")
    }

    @Test
    func testSelectBuyAmount() {
        let model = FiatSceneViewModelTests.mock()
        model.onSelect(amount: 150)

        #expect(model.inputValidationModel.text == "150")

        model.onSelect(amount: 1)

        #expect(model.inputValidationModel.text == "1")
    }

    @Test
    func testSelectSellAmount() {
        let model = FiatSceneViewModelTests.mock()
        model.type = .sell
        model.onChangeType(oldType: .buy, newType: .sell)

        model.onSelect(amount: 50)

        #expect(model.inputValidationModel.text == "50")

        model.onSelect(amount: 100)

        #expect(model.inputValidationModel.text == "100")
    }

    @Test
    func testCurrencySymbol() {
        let model = FiatSceneViewModelTests.mock()
        #expect(model.currencyInputConfig.currencySymbol == "$")

        model.type = .sell
        model.onChangeType(oldType: .buy, newType: .sell)

        #expect(model.currencyInputConfig.currencySymbol == "$")
    }

    @Test
    func testButtonsTitle() {
        let model = FiatSceneViewModelTests.mock()

        #expect(model.buttonTitle(amount: 10) == "$10")

        model.type = .sell
        model.onChangeType(oldType: .buy, newType: .sell)

        #expect(model.buttonTitle(amount: 100) == "$100")
    }

    @Test
    func testRateValue() {
        let model = FiatSceneViewModelTests.mock()
        let quote = FiatQuote.mock(fiatAmount: 1200, cryptoAmount: 2.0, type: model.type)
        model.buyViewModel.selectedQuote = quote

        #expect(model.rateValue == "1 \(model.asset.symbol) ≈ $600.00")
    }

    @Test
    func testFiatValidation() {
        let model = FiatSceneViewModelTests.mock()

        model.inputValidationModel.text = "24"
        #expect(model.inputValidationModel.update() == false)

        model.inputValidationModel.text = "25"
        #expect(model.inputValidationModel.update() == true)

        model.inputValidationModel.text = "10000"
        #expect(model.inputValidationModel.update() == true)

        model.inputValidationModel.text = "10001"
        #expect(model.inputValidationModel.update() == false)
    }

    @Test
    func testSellFiatValidation() {
        let model = FiatSceneViewModelTests.mock()
        model.type = .sell

        model.inputValidationModel.text = "24"
        #expect(model.inputValidationModel.update() == false)

        model.inputValidationModel.text = "25"
        #expect(model.inputValidationModel.update() == true)

        model.inputValidationModel.text = "10000"
        #expect(model.inputValidationModel.update() == true)

        model.inputValidationModel.text = "10001"
        #expect(model.inputValidationModel.update() == false)
    }

    @Test
    func actionButtonStateInvalidInput() {
        let model = FiatSceneViewModelTests.mock()
        model.buyViewModel.quotesState = .data(FiatQuotes(amount: 100, quotes: []))

        model.inputValidationModel.text = "4"
        model.inputValidationModel.update()

        #expect(model.actionButtonState.value == nil)
    }

    @Test
    func actionButtonStateLoading() {
        let model = FiatSceneViewModelTests.mock()
        model.buyViewModel.quotesState = .loading

        model.inputValidationModel.text = "100"
        model.inputValidationModel.update()

        #expect(model.actionButtonState.value == nil)
    }

    @Test
    func actionButtonStateValidWithQuote() {
        let model = FiatSceneViewModelTests.mock()
        let quote = FiatQuote.mock(fiatAmount: 100, cryptoAmount: 1, type: .buy)

        model.buyViewModel.quotesState = .data(FiatQuotes(amount: 100, quotes: [quote]))
        model.buyViewModel.selectedQuote = quote
        model.inputValidationModel.text = "100"
        model.inputValidationModel.update()

        #expect(model.actionButtonState.value != nil)
    }

    @Test
    func actionButtonStateValidNoQuote() {
        let model = FiatSceneViewModelTests.mock()
        model.buyViewModel.quotesState = .noData

        model.inputValidationModel.text = "100"
        model.inputValidationModel.update()

        #expect(model.actionButtonState.value == nil)
    }

    @Test
    func urlStateBlocksButton() {
        let model = FiatSceneViewModelTests.mock()
        let quote = FiatQuote.mock(fiatAmount: 100, cryptoAmount: 1, type: .buy)

        model.buyViewModel.quotesState = .data(FiatQuotes(amount: 100, quotes: [quote]))
        model.buyViewModel.selectedQuote = quote
        model.inputValidationModel.text = "100"
        model.inputValidationModel.update()

        #expect(model.actionButtonState.value != nil)

        model.urlState = .loading

        #expect(model.actionButtonState.value == nil)
    }

    @Test
    func urlStateInitialValue() {
        let model = FiatSceneViewModelTests.mock()

        #expect(model.urlState.isNoData == true)
        #expect(model.urlState.isLoading == false)
    }

    @Test
    func fetchTriggerOnChangeTypeIsImmediate() {
        let model = FiatSceneViewModelTests.mock()

        model.onChangeType(oldType: .buy, newType: .sell)

        #expect(model.fetchTrigger.type == .sell)
        #expect(model.fetchTrigger.isImmediate == true)
    }

    @Test
    func fetchTriggerOnSelectAmountIsImmediate() {
        let model = FiatSceneViewModelTests.mock()

        model.onSelect(amount: 250)

        #expect(model.fetchTrigger.amount == "250")
        #expect(model.fetchTrigger.isImmediate == true)
    }

    @Test
    func fetchTriggerOnChangeAmountTextIsDebounced() {
        let model = FiatSceneViewModelTests.mock()

        model.onChangeAmountText("", text: "123")

        #expect(model.fetchTrigger.amount == "123")
        #expect(model.fetchTrigger.isImmediate == false)
    }

    @Test
    func fetchTriggerOnSelectRandomAmountIsImmediate() {
        let model = FiatSceneViewModelTests.mock()

        model.onSelectRandomAmount()

        #expect(model.fetchTrigger.isImmediate == true)
    }

    // MARK: - ShouldSkipFetch Tests

    @Test
    func secondFetchSkippedWhenSameAmountLoading() {
        let model = FiatSceneViewModelTests.mock()

        model.buyViewModel.loadingAmount = 100.0

        #expect(model.buyViewModel.shouldSkipFetch(for: 100.0) == true)
    }

    @Test
    func secondFetchSkippedWhenDataExistsForSameAmount() {
        let model = FiatSceneViewModelTests.mock()
        
        model.buyViewModel.quotesState = .data(FiatQuotes(amount: 100.0, quotes: []))

        #expect(model.buyViewModel.shouldSkipFetch(for: 100.0) == true)
    }

    @Test
    func fetchAllowedForDifferentAmount() {
        let model = FiatSceneViewModelTests.mock()

        model.buyViewModel.loadingAmount = 100.0
        model.buyViewModel.quotesState = .data(FiatQuotes(amount: 100.0, quotes: []))

        #expect(model.buyViewModel.shouldSkipFetch(for: 200.0) == false)
    }
}
