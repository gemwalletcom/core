// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import Primitives

internal import GemstonePrimitives

public struct TransactionInfoViewModel: Sendable {
    private let asset: Asset
    private let assetPrice: Price?
    private let value: BigInt
    private let direction: TransactionDirection?

    private let feeAsset: Asset
    private let feeAssetPrice: Price?
    private let feeValue: BigInt?
    
    private let currency: String

    public init(
        currency: String,
        asset: Asset,
        assetPrice: Price?,
        feeAsset: Asset,
        feeAssetPrice: Price?,
        value: BigInt,
        feeValue: BigInt?,
        direction: TransactionDirection?
    ) {
        self.currency = currency
        self.asset = asset
        self.assetPrice = assetPrice
        self.feeAsset = feeAsset
        self.feeAssetPrice = feeAssetPrice
        self.value = value
        self.feeValue = feeValue
        self.direction = direction
    }
    
    public var isZero: Bool {
        value.isZero
    }

    public func amountDisplay(formatter: ValueFormatter = .full) -> AmountDisplay {
        .numeric(
            asset: asset,
            price: assetPrice,
            value: value,
            direction: direction,
            currency: currency,
            formatter: formatter
        )
    }

    public var feeDisplay: AmountDisplay? {
        feeValue.map {
            .numeric(
                asset: feeAsset,
                price: feeAssetPrice,
                value: $0,
                currency: currency,
                formatter: .medium
            )
        }
    }

    public func headerType(input: TransactionHeaderInputType) -> TransactionHeaderType {
        switch input {
        case let .amount(showFiat): .amount(
            amountDisplay().fiatVisibility(showFiat)
        )
        case let .nft(name, id): .nft(
            name: name,
            image: AssetImage(
                type: "NFT",
                imageURL: AssetImageFormatter().getNFTUrl(for: id),
                placeholder: .none,
                chainPlaceholder: .none
            )
        )
        case let .swap(swapInput): .swap(
            from: swapAmountField(input: swapInput.from),
            to: swapAmountField(input: swapInput.to)
        )
        case .symbol: .amount(.symbol(asset: asset))
        case .assetImage: .asset(image: AssetViewModel(asset: asset).assetImage)
        }
    }
}

extension TransactionInfoViewModel {
    private func swapAmountField(input: AssetValuePrice) -> SwapAmountField  {
        let display = AmountDisplay.numeric(
            data: input,
            style: .init(formatter: .medium, currencyCode: currency)
        )

        return SwapAmountField(
            assetImage: AssetIdViewModel(assetId: input.asset.id).assetImage,
            amount: display.amount.text,
            fiatAmount: display.fiat?.text
        )
    }
}
