// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import BigInt
import Components
@testable import Transfer
@testable import Primitives
import PrimitivesTestKit
import PrimitivesComponents

struct ConfirmHeaderViewModelTests {

    @Test
    func amountShowsClearHeader() {
        let model = ConfirmHeaderViewModel(
            headerType: .amount(
                .numeric(
                    NumericViewModel(
                        data: AssetValuePrice(asset: .mockEthereumUSDT(), value: BigInt(1), price: nil),
                        style: AmountDisplayStyle(currencyCode: "USD")
                    )
                )
            )
        )

        guard case .header(let item) = model.itemModel else { return }
        guard case .amount = item.headerType else { return }
        #expect(item.showClearHeader == true)
    }

    @Test
    func swapHidesClearHeader() {
        let model = ConfirmHeaderViewModel(
            headerType: .swap(
                from: SwapAmountField(assetImage: AssetImage(), amount: "1 ETH", fiatAmount: "$1"),
                to: SwapAmountField(assetImage: AssetImage(), amount: "2 USDC", fiatAmount: "$2")
            )
        )

        guard case .header(let item) = model.itemModel else { return }
        guard case .swap = item.headerType else { return }
        #expect(item.showClearHeader == false)
    }

    @Test
    func nftShowsClearHeader() {
        let model = ConfirmHeaderViewModel(
            headerType: .nft(name: nil, image: AssetImage())
        )

        guard case .header(let item) = model.itemModel else { return }
        guard case .nft = item.headerType else { return }
        #expect(item.showClearHeader == true)
    }

    @Test
    func assetValueShowsClearHeader() {
        let model = ConfirmHeaderViewModel(
            headerType: .assetValue(AssetValueHeaderData(asset: .mockEthereumUSDT(), value: .exact(BigInt(1000000))))
        )

        guard case .header(let item) = model.itemModel else { return }
        guard case .assetValue(let data) = item.headerType else { return }
        #expect(data.asset == .mockEthereumUSDT())
        #expect(data.value == .exact(BigInt(1000000)))
        #expect(item.showClearHeader == true)
    }

    @Test
    func assetShowsClearHeader() {
        let model = ConfirmHeaderViewModel(headerType: .asset(image: AssetImage()))

        guard case .header(let item) = model.itemModel else { return }
        guard case .asset = item.headerType else { return }
        #expect(item.showClearHeader == true)
    }
}
