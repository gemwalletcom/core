// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import BigInt
import Validators

public typealias TransferAmountValidation = Result<TransferAmount, TransferAmountCalculatorError>

public struct TransferAmountCalculator {
    public init() {}

    public func validate(input: TransferAmountInput) -> TransferAmountValidation {
        do {
            return .success(try calculate(input: input))
        } catch {
            return .failure(error)
        }
    }

    public func validateNetworkFee(_ feeBalance: BigInt, feeAssetId: AssetId) throws(TransferAmountCalculatorError) {
        if [Chain.hyperCore, Chain.tron].contains(feeAssetId.chain) {
            return
        }
        if feeBalance.isZero && feeAssetId.type == .native {
            throw TransferAmountCalculatorError.insufficientNetworkFee(feeAssetId.chain.asset, required: nil)
        }
    }

    func calculate(input: TransferAmountInput) throws(TransferAmountCalculatorError) -> TransferAmount {
        if input.assetBalance.available == 0 && !input.ignoreValueCheck {
            guard input.fee.isZero else {
                throw TransferAmountCalculatorError.insufficientBalance(input.asset)
            }
        }

        if input.ignoreValueCheck {
            // some chains like hypercore does not require fee for transactions, incorporate this into the flow
            let chains = Set<Chain>([Chain.hyperCore])
            
            if input.assetFeeBalance.available < input.fee && !chains.contains(input.assetFee.chain) {
                throw TransferAmountCalculatorError.insufficientNetworkFee(input.assetFee, required: input.fee)
            }
            return TransferAmount(value: input.value, networkFee: input.fee, useMaxAmount: false)
        }

        if input.availableValue < input.value  {
            throw TransferAmountCalculatorError.insufficientBalance(input.asset)
        }

        if input.assetFeeBalance.available < input.fee {
            throw TransferAmountCalculatorError.insufficientNetworkFee(input.assetFee, required: input.fee)
        }

        if !input.canChangeValue && input.asset == input.assetFee {
            if  input.availableValue < input.value + input.fee {
                throw TransferAmountCalculatorError.insufficientBalance(input.asset)
            }
        }

        // max value transfer
        if input.assetBalance.available == input.value {
            if input.asset == input.asset.feeAsset && input.canChangeValue  {
                return TransferAmount(
                    value: input.assetBalance.available - input.fee,
                    networkFee: input.fee,
                    useMaxAmount: true
                )
            }
            return TransferAmount(value: input.assetBalance.available, networkFee: input.fee, useMaxAmount: true)
        }
        if input.asset.type == .native && input.asset.chain.minimumAccountBalance > 0 &&
            (input.availableValue - input.value - input.fee).isBetween(-BigInt.MAX_256, and: input.asset.chain.minimumAccountBalance)
        {
            throw TransferAmountCalculatorError.minimumAccountBalanceTooLow(input.asset, required: input.asset.chain.minimumAccountBalance)
        }

        let useMaxAmount = input.availableValue == input.value

        return TransferAmount(value: input.value, networkFee: input.fee, useMaxAmount: useMaxAmount)
    }
}
