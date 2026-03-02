// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Primitives
import BigInt
import Testing
import Validators

@testable import PrimitivesComponents

struct TransferAmountCalculatorTests {
    let coinAsset = Asset(.ethereum)
    let tokenAsset = Asset(
        id: AssetId(chain: .ethereum, tokenId: "0x1"),
        name: "",
        symbol: "",
        decimals: 0,
        type: .erc20
    )
    let service = TransferAmountCalculator()

    @Test
    func testTransferCoin() {
        #expect(throws: TransferAmountCalculatorError.insufficientBalance(coinAsset)) {
            try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: .zero,
                value: BigInt(10),
                availableValue: .zero,
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(10)),
                fee: BigInt(1),
                canChangeValue: true,
                ignoreValueCheck: false
            ))
        }

        #expect(throws: TransferAmountCalculatorError.insufficientBalance(coinAsset)) {
            try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: .zero,
                value: .zero,
                availableValue: BigInt(0),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: .zero),
                fee: BigInt(1),
                canChangeValue: true,
                ignoreValueCheck: false
            ))
        }

        #expect(throws: TransferAmountCalculatorError.insufficientBalance(coinAsset)) {
            try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: BigInt(10)),
                value: BigInt(20),
                availableValue: BigInt(10),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(100)),
                fee: BigInt(0),
                canChangeValue: true,
                ignoreValueCheck: false
            ))
        }

        #expect(throws: TransferAmountCalculatorError.insufficientBalance(coinAsset)) {
            try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: .zero,
                value: BigInt(10),
                availableValue: .zero,
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: .zero,
                fee: .zero,
                canChangeValue: true,
                ignoreValueCheck: false
            ))
        }

        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: .zero,
                value: .zero,
                availableValue: .zero,
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: .zero),
                fee: .zero,
                canChangeValue: true,
                ignoreValueCheck: false
            ))
            #expect(result == TransferAmount(value: .zero, networkFee: .zero, useMaxAmount: true))
        }

        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: BigInt(100)),
                value: BigInt(50),
                availableValue: BigInt(100),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(100)),
                fee: .zero,
                canChangeValue: true,
                ignoreValueCheck: false
            ))
            #expect(result == TransferAmount(value: 50, networkFee: .zero, useMaxAmount: false))
        }

        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: 12),
                value: BigInt(10),
                availableValue: BigInt(12),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(12)),
                fee: BigInt(1),
                canChangeValue: true,
                ignoreValueCheck: false
            ))
            #expect(result == TransferAmount(value: 10, networkFee: 1, useMaxAmount: false))
        }

        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: 12),
                value: BigInt(11),
                availableValue: BigInt(12),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(12)),
                fee: BigInt(1),
                canChangeValue: true,
                ignoreValueCheck: false
            ))
            #expect(result == TransferAmount(value: 11, networkFee: 1, useMaxAmount: false))
        }

        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: 12),
                value: BigInt(12),
                availableValue: BigInt(12),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(12)),
                fee: BigInt(3),
                canChangeValue: true,
                ignoreValueCheck: false
            ))
            #expect(result == TransferAmount(value: 9, networkFee: 3, useMaxAmount: true))
        }
    }

    @Test
    func testClaimRewards() {
        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: 12),
                value: BigInt(1000),
                availableValue: BigInt(1000),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(12)),
                fee: BigInt(1),
                canChangeValue: true,
                ignoreValueCheck: false
            ))
            #expect(result == TransferAmount(value: 1000, networkFee: 1, useMaxAmount: true))
        }

        #expect(throws: TransferAmountCalculatorError.insufficientBalance(coinAsset)) {
            try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: 12),
                value: BigInt(1000),
                availableValue: BigInt(12),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(12)),
                fee: BigInt(1),
                canChangeValue: true,
                ignoreValueCheck: false
            ))
        }
    }

    @Test
    func testCanChangeValue() {
        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: 12),
                value: BigInt(12),
                availableValue: BigInt(12),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(12)),
                fee: BigInt(3),
                canChangeValue: true,
                ignoreValueCheck: false
            ))
            #expect(result == TransferAmount(value: 9, networkFee: 3, useMaxAmount: true))
        }

        #expect(throws: TransferAmountCalculatorError.insufficientBalance(coinAsset)) {
            try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: 12),
                value: BigInt(12),
                availableValue: BigInt(12),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(12)),
                fee: BigInt(3),
                canChangeValue: false,
                ignoreValueCheck: false
            ))
        }
    }

    @Test
    func testIgnoreValueCheck() {
        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: 12),
                value: BigInt(2222),
                availableValue: BigInt(12),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(12)),
                fee: BigInt(3),
                canChangeValue: true,
                ignoreValueCheck: true
            ))
            #expect(result == TransferAmount(value: 2222, networkFee: 3, useMaxAmount: false))
        }

        #expect(throws: TransferAmountCalculatorError.insufficientNetworkFee(coinAsset, required: BigInt(13))) {
            try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: 12),
                value: BigInt(2222),
                availableValue: BigInt(12),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(12)),
                fee: BigInt(13),
                canChangeValue: true,
                ignoreValueCheck: true
            ))
        }
    }

    @Test
    func testInsufficientBalanceError() {
        #expect(throws: TransferAmountCalculatorError.insufficientBalance(coinAsset)) {
            try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: BigInt(50)),
                value: BigInt(100),
                availableValue: BigInt(50),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(20)),
                fee: BigInt(3),
                transferData: TransferData.mock(type: .transfer(coinAsset), value: BigInt(100))
            ))
        }
    }

    @Test
    func testMinimumAccountBalance() {
        let asset1 = Asset(.solana)

        #expect(throws: TransferAmountCalculatorError.minimumAccountBalanceTooLow(asset1, required: BigInt(890880))) {
            try service.calculate(input: TransferAmountInput(
                asset: asset1,
                assetBalance: Balance(available: BigInt(1000890880)),
                value: BigInt(1000590880),
                availableValue: BigInt(1000890880),
                assetFee: asset1.feeAsset,
                assetFeeBalance: Balance(available: .zero),
                fee: .zero,
                canChangeValue: true,
                ignoreValueCheck: false
            ))
        }

        #expect(throws: TransferAmountCalculatorError.minimumAccountBalanceTooLow(asset1, required: BigInt(890880))) {
            try service.calculate(input: TransferAmountInput(
                asset: asset1,
                assetBalance: Balance(available: BigInt(1_000_000)),
                value: BigInt(900_000),
                availableValue: BigInt(1_000_000),
                assetFee: asset1.feeAsset,
                assetFeeBalance: Balance(available: BigInt(1_000_000)),
                fee: BigInt(200_000),
                canChangeValue: true,
                ignoreValueCheck: false
            ))
        }

        let asset2 = Asset(.bitcoin)

        #expect(throws: Never.self) {
            try service.calculate(input: TransferAmountInput(
                asset: asset2,
                assetBalance: Balance(available: BigInt(1000890880)),
                value: BigInt(1000590880),
                availableValue: BigInt(1000890880),
                assetFee: asset2.feeAsset,
                assetFeeBalance: Balance(available: .zero),
                fee: .zero,
                canChangeValue: true,
                ignoreValueCheck: false
            ))
        }
    }

    @Test
    func testMinimumAccountBalanceForToken() {
        let assetCoin = Asset.mockEthereum()
        let assetToken = Asset.mockEthereumUSDT()

        #expect(throws: Never.self) {
            try service.calculate(input: TransferAmountInput(
                asset: assetToken,
                assetBalance: Balance(available: BigInt(1000890880)),
                value: BigInt(1000590880),
                availableValue: BigInt(1000890880),
                assetFee: assetCoin,
                assetFeeBalance: Balance(available: .zero),
                fee: .zero,
                canChangeValue: true,
                ignoreValueCheck: false
            ))
        }
    }

    @Test
    func testValidateNetworkFee() {
        #expect(throws: TransferAmountCalculatorError.insufficientNetworkFee(.mockEthereum(), required: nil)) {
            try service.validateNetworkFee(
                .zero,
                feeAssetId: .mockEthereum()
            )
        }

        #expect(throws: TransferAmountCalculatorError.insufficientNetworkFee(coinAsset, required: BigInt(10))) {
            try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: BigInt(100)),
                value: BigInt(50),
                availableValue: BigInt(100),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(5)),
                fee: BigInt(10),
                transferData: TransferData.mock(type: .transfer(coinAsset), value: BigInt(50))
            ))
        }

        #expect(throws: Never.self) {
            try service.validateNetworkFee(
                BigInt(123_456),
                feeAssetId: .mockEthereum()
            )
        }
    }

    @Test
    func testWithdraw() throws {
        let asset = Asset(.solana)
        let input = TransferAmountInput(
            asset: asset,
            assetBalance: Balance(available: BigInt(1_060_000)),
            value: BigInt(100_000),
            availableValue: BigInt(1_000_000),
            assetFee: asset.feeAsset,
            assetFeeBalance: Balance(available: BigInt(1_000_000)),
            fee: BigInt(5_000),
            canChangeValue: false,
            ignoreValueCheck: false
        )

        #expect(throws: Never.self) {
            try service.calculate(input: input)
        }
    }

    @Test
    func testTransferFlexible() {
        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: BigInt(100)),
                value: BigInt(50),
                availableValue: BigInt(100),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(100)),
                fee: BigInt(5),
                transferData: TransferData.mock(type: .transfer(coinAsset), value: BigInt(50))
            ))
            #expect(result == TransferAmount(value: 50, networkFee: 5, useMaxAmount: false))
        }
    }

    @Test
    func testTransferFixed() {
        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: BigInt(100)),
                value: BigInt(50),
                availableValue: BigInt(100),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(100)),
                fee: BigInt(5),
                canChangeValue: false,
                ignoreValueCheck: false
            ))
            #expect(result == TransferAmount(value: 50, networkFee: 5, useMaxAmount: false))
        }
    }

    @Test
    func testSwapCalculation() {
        let swapData = SwapData.mock()
        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: BigInt(200)),
                value: BigInt(100),
                availableValue: BigInt(200),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(50)),
                fee: BigInt(10),
                transferData: TransferData.mock(type: .swap(coinAsset, tokenAsset, swapData), value: BigInt(100))
            ))
            #expect(result == TransferAmount(value: 100, networkFee: 10, useMaxAmount: false))
        }
    }

    @Test
    func testNftTransfer() {
        let nftAsset = NFTAsset.mock()
        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: BigInt(50)),
                value: BigInt(1),
                availableValue: BigInt(50),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(20)),
                fee: BigInt(3),
                transferData: TransferData.mock(type: .transferNft(nftAsset), value: BigInt(1))
            ))
            #expect(result == TransferAmount(value: 1, networkFee: 3, useMaxAmount: false))
        }
    }

    @Test
    func testStakeFlexible() {
        let stakeType = StakeType.stake(.mock())
        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: BigInt(1000)),
                value: BigInt(500),
                availableValue: BigInt(1000),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(100)),
                fee: BigInt(5),
                transferData: TransferData.mock(type: .stake(coinAsset, stakeType), value: BigInt(500))
            ))
            #expect(result == TransferAmount(value: 500, networkFee: 5, useMaxAmount: false))
        }
    }

    @Test
    func testUnstakeFixed() {
        let delegation = Delegation.mock(state: .active)
        let stakeType = StakeType.unstake(delegation)
        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: BigInt(800)),
                value: BigInt(300),
                availableValue: BigInt(800),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(50)),
                fee: BigInt(2),
                transferData: TransferData.mock(type: .stake(coinAsset, stakeType), value: BigInt(300), canChangeValue: false)
            ))
            #expect(result == TransferAmount(value: 300, networkFee: 2, useMaxAmount: false))
        }
    }

    @Test
    func testMaxAmountTransfer() {
        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: BigInt(100)),
                value: BigInt(100),
                availableValue: BigInt(100),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(100)),
                fee: BigInt(5),
                transferData: TransferData.mock(type: .transfer(coinAsset), value: BigInt(100))
            ))
            #expect(result == TransferAmount(value: 95, networkFee: 5, useMaxAmount: true))
        }
    }

    @Test
    func testDeposit() {
        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: BigInt(200)),
                value: BigInt(100),
                availableValue: BigInt(200),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(50)),
                fee: BigInt(5),
                transferData: TransferData.mock(type: .deposit(coinAsset), value: BigInt(100))
            ))
            #expect(result == TransferAmount(value: 100, networkFee: 5, useMaxAmount: false))
        }
    }

    @Test
    func testPerpetualOpen() {
        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: BigInt(500)),
                value: BigInt(200),
                availableValue: BigInt(500),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(100)),
                fee: BigInt(8),
                transferData: TransferData.mock(
                    type: .perpetual(coinAsset, .open(.mock(direction: .long, assetIndex: 0, price: "100", size: "1"))),
                    value: BigInt(200)
                )
            ))
            #expect(result == TransferAmount(value: 200, networkFee: 8, useMaxAmount: false))
        }
    }

    @Test
    func testPerpetualClose() {
        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: coinAsset,
                assetBalance: Balance(available: BigInt(50)),
                value: BigInt(999_999),
                availableValue: BigInt(0),
                assetFee: coinAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(20)),
                fee: BigInt(4),
                transferData: TransferData.mock(
                    type: .perpetual(coinAsset, .close(.mock(direction: .long, assetIndex: 0, price: "100", size: "1"))),
                    value: BigInt(999_999)
                )
            ))
            #expect(result == TransferAmount(value: 999_999, networkFee: 4, useMaxAmount: false))
        }
    }
    
    @Test
    func testHypercoreIgnoreValueCheck() {
        let hypercoreAsset = Asset(.hyperCore)
        #expect(throws: Never.self) {
            let result = try service.calculate(input: TransferAmountInput(
                asset: hypercoreAsset,
                assetBalance: Balance(available: BigInt(0)),
                value: BigInt(1000),
                availableValue: BigInt(0),
                assetFee: hypercoreAsset.feeAsset,
                assetFeeBalance: Balance(available: BigInt(0)),
                fee: BigInt(0),
                canChangeValue: true,
                ignoreValueCheck: true
            ))
            #expect(result == TransferAmount(value: 1000, networkFee: 0, useMaxAmount: false))
        }
    }
}
