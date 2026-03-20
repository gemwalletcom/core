// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing
import Primitives
import PrimitivesTestKit
import Formatters
import BigInt
import Validators

@testable import FiatConnect

@MainActor
final class FiatOperationViewModelTests {

    private struct MockFiatOperation: FiatOperation {
        var quotes: [FiatQuote] = []
        var defaultAmount: Int = 50
        var emptyAmountTitle: String = "Mock Title"

        func fetch(amount: Double) async throws -> [FiatQuote] {
            quotes
        }

        func validators(availableBalance: BigInt, selectedQuote: FiatQuote?) -> [any TextValidator] {
            []
        }
    }

    private static func mock(
        operation: FiatOperation = MockFiatOperation(),
        asset: Asset = .mock(),
        currencyFormatter: CurrencyFormatter = .init(locale: .US, currencyCode: Currency.usd.rawValue)
    ) -> FiatOperationViewModel {
        FiatOperationViewModel(
            operation: operation,
            asset: asset,
            currencyFormatter: currencyFormatter
        )
    }

    @Test
    func shouldSkipFetchWhenAlreadyLoading() {
        let model = FiatOperationViewModelTests.mock()
        model.loadingAmount = 100.0

        #expect(model.shouldSkipFetch(for: 100.0) == true)
        #expect(model.shouldSkipFetch(for: 50.0) == false)
    }

    @Test
    func shouldSkipFetchWhenDataExists() {
        let model = FiatOperationViewModelTests.mock()
        let quotes = FiatQuotes(amount: 100.0, quotes: [.mock()])
        model.quotesState = .data(quotes)

        #expect(model.shouldSkipFetch(for: 100.0) == true)
        #expect(model.shouldSkipFetch(for: 50.0) == false)
    }

    @Test
    func shouldNotSkipFetchForErrorState() {
        let model = FiatOperationViewModelTests.mock()
        model.quotesState = .error(NSError(domain: "test", code: 1))

        #expect(model.shouldSkipFetch(for: 100.0) == false)
    }

    @Test
    func shouldNotSkipFetchForLoadingState() {
        let model = FiatOperationViewModelTests.mock()
        model.quotesState = .loading

        #expect(model.shouldSkipFetch(for: 100.0) == false)
    }

    @Test
    func shouldNotSkipFetchForNoDataState() {
        let model = FiatOperationViewModelTests.mock()
        model.quotesState = .noData

        #expect(model.shouldSkipFetch(for: 100.0) == false)
    }

    @Test
    func onChangeAmountTextClearsQuoteAndSetsLoading() {
        let model = FiatOperationViewModelTests.mock()
        model.amount = "50"
        model.selectedQuote = .mock()
        model.quotesState = .data(FiatQuotes(amount: 50, quotes: [.mock()]))

        model.onChangeAmountText("", text: "100")

        #expect(model.selectedQuote == nil)
        #expect(model.amount == "100")
        #expect(model.quotesState.isLoading == true)
    }

    @Test
    func onChangeAmountTextPreservesLoadingState() {
        let model = FiatOperationViewModelTests.mock()
        model.amount = "50"
        model.selectedQuote = .mock()
        model.quotesState = .loading

        model.onChangeAmountText("", text: "100")

        #expect(model.selectedQuote == nil)
        #expect(model.amount == "100")
        #expect(model.quotesState.isLoading == true)
    }

    @Test
    func onChangeAmountTextPreservesQuoteWhenAmountUnchanged() {
        let model = FiatOperationViewModelTests.mock()
        let quote = FiatQuote.mock()
        model.amount = "50"
        model.selectedQuote = quote
        model.quotesState = .data(FiatQuotes(amount: 50, quotes: [quote]))

        model.onChangeAmountText("", text: "50")

        #expect(model.selectedQuote == quote)
        #expect(model.amount == "50")
        #expect(model.quotesState.isLoading == false)
    }

    @Test
    func fetchSetsNoDataWhenInputInvalid() {
        let model = FiatOperationViewModelTests.mock()
        model.inputValidationModel.text = "invalid"
        model.quotesState = .loading

        model.fetch()

        #expect(model.quotesState.isNoData == true)
    }

    @Test
    func fetchSetsNoDataWhenAmountZero() {
        let model = FiatOperationViewModelTests.mock()
        model.inputValidationModel.text = "0"
        model.quotesState = .loading

        model.fetch()

        #expect(model.quotesState.isNoData == true)
    }

    @Test
    func fetchSetsNoDataWhenValidationFailsWithNoMatchingQuotes() {
        let model = FiatOperationViewModelTests.mock(operation: MockFiatOperationWithValidator())
        model.inputValidationModel.text = "20000"
        model.quotesState = .loading

        model.fetch()

        #expect(model.quotesState.isNoData == true)
    }

    @Test
    func fetchPreservesQuotesWhenValidationInvalidForSameAmount() {
        let model = FiatOperationViewModelTests.mock(operation: MockFiatOperationWithValidator())
        let quote = FiatQuote.mock()
        let quotes = FiatQuotes(amount: 20000.0, quotes: [quote])
        model.quotesState = .data(quotes)
        model.selectedQuote = quote
        model.inputValidationModel.text = "20000"

        model.fetch()

        #expect(model.quotesState.value?.quotes.count == 1)
        #expect(model.selectedQuote == quote)
    }

    @Test
    func fetchSetsNoDataWhenValidationFailsForDifferentAmount() {
        let model = FiatOperationViewModelTests.mock(operation: MockFiatOperationWithValidator())
        let quote = FiatQuote.mock()
        let quotes = FiatQuotes(amount: 100.0, quotes: [quote])
        model.quotesState = .data(quotes)
        model.selectedQuote = quote
        model.inputValidationModel.text = "20000"

        model.fetch()

        #expect(model.quotesState.isNoData == true)
    }

    private struct MockFiatOperationWithValidator: FiatOperation {
        var defaultAmount: Int = 50
        var emptyAmountTitle: String = "Mock Title"

        func fetch(amount: Double) async throws -> [FiatQuote] {
            []
        }

        func validators(availableBalance: BigInt, selectedQuote: FiatQuote?) -> [any TextValidator] {
            let rangeValidator = FiatRangeValidator(
                range: BigInt(25)...BigInt(10000),
                minimumValueText: "$25",
                maximumValueText: "$10,000"
            )
            return [.assetAmount(decimals: 0, validators: [rangeValidator])]
        }
    }
}
