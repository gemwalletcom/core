// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Formatters
import Primitives
import PrimitivesTestKit
@testable import PrimitivesComponents

struct PerpetualDetailsViewModelTests {

    @Test
    func leverageText() {
        #expect(PerpetualDetailsViewModel.mock(.open(.mock(leverage: 5))).leverageText == "5x")
    }

    @Test
    func slippageField() {
        #expect(PerpetualDetailsViewModel.mock(.open(.mock(slippage: 2.0))).slippageField.value.text == "2.00%")
    }

    @Test
    func entryPriceField() {
        #expect(PerpetualDetailsViewModel.mock(.open(.mock(entryPrice: 48000.0))).entryPriceField?.value.text == "$48,000.00")
        #expect(PerpetualDetailsViewModel.mock(.open(.mock(entryPrice: nil))).entryPriceField == nil)
    }

    @Test
    func marginField() {
        #expect(PerpetualDetailsViewModel.mock(.open(.mock(marginAmount: 1000.0))).marginField.value.text == "$1,000.00")
    }

    @Test
    func sizeField() {
        #expect(PerpetualDetailsViewModel.mock(.open(.mock(fiatValue: 5000.0))).sizeField.value.text == "$5,000.00")
    }

    @Test
    func positionText() {
        #expect(PerpetualDetailsViewModel.mock(.open(.mock(direction: .long, leverage: 40))).positionText == "Long 40x")
        #expect(PerpetualDetailsViewModel.mock(.open(.mock(direction: .short, leverage: 10))).positionText == "Short 10x")
        #expect(PerpetualDetailsViewModel.mock(.increase(.mock(direction: .long, leverage: 5))).positionText == "Long 5x")

        let reduceModel = PerpetualDetailsViewModel.mock(.reduce(positionDirection: .short))
        #expect(reduceModel.positionText == "Short 3x")
    }

    @Test
    func listItemModelSubtitle() {
        let closeModel = PerpetualDetailsViewModel.mock(.close(.mock(pnl: 500, marginAmount: 1000)))
        #expect(closeModel.listItemModel.title == "Details")
        #expect(closeModel.listItemModel.subtitle == "+$500.00 (+50.00%)")

        #expect(PerpetualDetailsViewModel.mock(.open(.mock())).listItemModel.subtitle == "Long 3x")

        let increaseModel = PerpetualDetailsViewModel.mock(.increase(.mock()))
        #expect(increaseModel.listItemModel.subtitle == "Increase Long")

        let increaseShortModel = PerpetualDetailsViewModel.mock(.increase(.mock(direction: .short)))
        #expect(increaseShortModel.listItemModel.subtitle == "Increase Short")

        let reduceLongModel = PerpetualDetailsViewModel.mock(.reduce())
        #expect(reduceLongModel.listItemModel.subtitle == "Reduce Long")

        let reduceShortModel = PerpetualDetailsViewModel.mock(.reduce(positionDirection: .short))
        #expect(reduceShortModel.listItemModel.subtitle == "Reduce Short")
    }
}

extension PerpetualDetailsViewModel {
    static func mock(_ type: PerpetualDetailsType) -> PerpetualDetailsViewModel {
        PerpetualDetailsViewModel(type: type)
    }
}

extension PerpetualDetailsType {
    static func reduce(
        _ data: PerpetualConfirmData = .mock(),
        positionDirection: PerpetualDirection = .long
    ) -> PerpetualDetailsType {
        .reduce(PerpetualReduceData(data: data, positionDirection: positionDirection))
    }
}
