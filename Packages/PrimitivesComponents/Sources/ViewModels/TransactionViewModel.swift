// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import Localization
import Primitives
import Style
import SwiftUI

public struct TransactionViewModel: Sendable {
    public let transaction: TransactionExtended

    private let explorerService: any ExplorerLinkFetchable
    private let assetImageFormatter = AssetImageFormatter()
    private let currency: String
    private let formatter: ValueFormatter = .short

    public init(
        explorerService: any ExplorerLinkFetchable,
        transaction: TransactionExtended,
        currency: String
    ) {
        self.transaction = transaction
        self.explorerService = explorerService
        self.currency = currency
    }

    public var assetImage: AssetImage {
        let asset = AssetIdViewModel(assetId: assetId).assetImage
        if let nftMetadata = transaction.transaction.metadata?.decode(TransactionNFTTransferMetadata.self) {
            return AssetImage(
                type: "",
                imageURL: assetImageFormatter.getNFTUrl(for: nftMetadata.assetId),
                placeholder: asset.placeholder,
                chainPlaceholder: overlayImage
            )
        }
        return AssetImage(
            type: asset.type,
            imageURL: asset.imageURL,
            placeholder: asset.placeholder,
            chainPlaceholder: overlayImage
        )
    }

    public var overlayImage: Image? {
        switch transaction.transaction.type {
        case .transfer, .transferNFT, .smartContractCall:
            switch transaction.transaction.direction {
            case .incoming: Images.Transaction.incoming
            case .outgoing, .selfTransfer: Images.Transaction.outgoing
            }
        case .swap,
                .tokenApproval,
                .stakeDelegate,
                .stakeUndelegate,
                .stakeRewards,
                .stakeRedelegate,
                .stakeWithdraw,
                .assetActivation,
                .perpetualOpenPosition,
                .perpetualClosePosition,
                .stakeFreeze,
                .stakeUnfreeze,
                .perpetualModifyPosition,
                .earnDeposit,
                .earnWithdraw: .none
        }
    }

    public var infoModel: TransactionInfoViewModel {
        let direction: TransactionDirection? = switch transaction.transaction.type {
        case .transfer: transaction.transaction.direction
        case .stakeRewards, .stakeWithdraw: .incoming
        default: nil
        }

        return TransactionInfoViewModel(
            currency: currency,
            asset: transaction.asset,
            assetPrice: transaction.price,
            feeAsset: transaction.feeAsset,
            feeAssetPrice: transaction.feePrice,
            value: transaction.transaction.valueBigInt,
            feeValue: transaction.transaction.feeBigInt,
            direction: direction
        )
    }

    public var titleTextValue: TextValue {
        let title: String = {
            switch transaction.transaction.type {
            case .transfer, .transferNFT, .smartContractCall:
                switch transaction.transaction.state {
                case .confirmed:
                    switch transaction.transaction.direction {
                    case .incoming:
                        return Localized.Transaction.Title.received
                    case .outgoing, .selfTransfer:
                        return Localized.Transaction.Title.sent
                    }
                case .failed, .pending, .reverted, .inTransit:
                    return Localized.Transfer.title
                }
            case .swap:
                return Localized.Wallet.swap
            case .tokenApproval:
                return Localized.Transfer.Approve.title
            case .stakeDelegate:
                return Localized.Transfer.Stake.title
            case .stakeUndelegate:
                return Localized.Transfer.Unstake.title
            case .stakeRedelegate:
                return Localized.Transfer.Redelegate.title
            case .stakeRewards:
                return Localized.Transfer.Rewards.title
            case .stakeWithdraw:
                return Localized.Transfer.Withdraw.title
            case .assetActivation:
                return Localized.Transfer.ActivateAsset.title
            case .stakeFreeze:
                return Localized.Transfer.Freeze.title
            case .stakeUnfreeze:
                return Localized.Transfer.Unfreeze.title
            case .perpetualOpenPosition:
                if let metadata = transaction.transaction.metadata?.decode(TransactionPerpetualMetadata.self) {
                    return Localized.Perpetual.openDirection(PerpetualDirectionViewModel(direction: metadata.direction).title)
                }
                return .empty
            case .perpetualClosePosition:
                if let metadata = transaction.transaction.metadata?.decode(TransactionPerpetualMetadata.self) {
                    return Localized.Perpetual.closeDirection(PerpetualDirectionViewModel(direction: metadata.direction).title)
                }
                return .empty
            case .perpetualModifyPosition:
                return .empty
            case .earnDeposit:
                return Localized.Transfer.Stake.title
            case .earnWithdraw:
                return Localized.Transfer.Withdraw.title
            }
        }()
        return TextValue(
            text: title,
            style: TextStyle(font: Font.system(.body, weight: .medium), color: .primary)
        )
    }

    public var titleTagType: TitleTagType {
        switch transaction.transaction.state {
        case .confirmed: .none
        case .pending, .inTransit: .progressView()
        case .failed, .reverted: .none // TODO: Image
        }
    }

    public var titleTagTextValue: TextValue? {
        let title: String? = switch transaction.transaction.state {
        case .confirmed: .none
        case .pending, .inTransit, .failed, .reverted: TransactionStateViewModel(state: transaction.transaction.state).title
        }
        let model = TransactionStateViewModel(state: transaction.transaction.state)
        return title.map {
            TextValue(
                text: $0,
                style: TextStyle(
                    font: Font.system(.footnote, weight: .medium),
                    color: model.color,
                    background: model.background
                )
            )
        }
    }

    public var titleExtraTextValue: TextValue? {
        let title: String? = {
            let chain = assetId.chain
            switch transaction.transaction.type {
            case .transfer, .transferNFT, .tokenApproval, .smartContractCall:
                switch transaction.transaction.direction {
                case .incoming:
                    return String(
                        format: "%@ %@",
                        Localized.Transfer.from,
                        getDisplayName(address: transaction.transaction.from, chain: chain)
                    )
                case .outgoing, .selfTransfer:
                    return String(
                        format: "%@ %@",
                        Localized.Transfer.to,
                        getDisplayName(address: transaction.transaction.to, chain: chain)
                    )
                }
            case .stakeDelegate,
                    .stakeRedelegate:
                return String(
                    format: "%@ %@",
                    Localized.Transfer.to,
                    getDisplayName(address: transaction.transaction.to, chain: chain)
                )
            case .stakeUndelegate:
                return String(
                    format: "%@ %@",
                    Localized.Transfer.from,
                    getDisplayName(address: transaction.transaction.to, chain: chain)
                )
            case .stakeFreeze:
                guard let title = getResourceTitle() else { return .none }
                return String(format: "%@ %@", Localized.Transfer.to, title)
            case .stakeUnfreeze:
                guard let title = getResourceTitle() else { return .none }
                return String(format: "%@ %@", Localized.Transfer.from, title)
            case .swap,
                    .stakeRewards,
                    .stakeWithdraw,
                    .assetActivation,
                    .perpetualModifyPosition,
                    .perpetualOpenPosition,
                    .perpetualClosePosition,
                    .earnDeposit,
                    .earnWithdraw:
                guard let metadata = transaction.transaction.metadata?.decode(TransactionPerpetualMetadata.self) else {
                    return .none
                }
                let price = AmountDisplay.currency(value: metadata.price, currencyCode: Currency.usd.rawValue, showSign: false).text
                return String(format: "%@: %@", Localized.Asset.price, price)
            }
        }()

        return title.map {
            TextValue(
                text: $0,
                style: .footnote
            )
        }
    }

    public var subtitleTextValue: TextValue? {
        switch transaction.transaction.type {
        case .transfer,
            .smartContractCall,
            .stakeRewards,
            .stakeWithdraw,
            .stakeDelegate,
            .stakeUndelegate,
            .stakeRedelegate,
            .assetActivation,
            .stakeFreeze,
            .stakeUnfreeze,
            .earnDeposit,
            .earnWithdraw:
            return infoModel.amountDisplay(formatter: formatter).amount
        case .perpetualClosePosition:
            guard let metadata = transaction.transaction.metadata?.decode(TransactionPerpetualMetadata.self), metadata.pnl != 0 else {
                return .none
            }
            return AmountDisplay.currency(value: metadata.pnl, currencyCode: Currency.usd.rawValue)
        case .perpetualOpenPosition:
            return AmountDisplay.numeric(
                asset: .hypercoreUSDC(),
                price: Price(price: 1, priceChangePercentage24h: .zero, updatedAt: .now),
                value: transaction.transaction.valueBigInt,
                currency: Currency.usd.rawValue,
                formatter: formatter,
                textStyle: TextStyle(font: .body, color: Colors.black, fontWeight: .medium)
            ).fiat
        case .tokenApproval:
            return AmountDisplay.symbol(asset: transaction.asset).amount
        case .swap:
            guard let metadata = transaction.transaction.metadata?.decode(TransactionSwapMetadata.self), let asset = transaction.assets.first(where: { $0.id == metadata.toAsset }) else {
                return .none
            }
            return AmountDisplay.numeric(
                data: AssetValuePrice(asset: asset, value: BigInt.fromString(metadata.toValue), price: nil),
                style: AmountDisplayStyle(sign: .incoming, formatter: formatter, currencyCode: currency)
            ).amount
        case .transferNFT:
            return nil
        case .perpetualModifyPosition:
            return nil
        }
    }

    public var subtitleExtraTextValue: TextValue? {
        switch transaction.transaction.type {
        case .transfer,
                .transferNFT,
                .tokenApproval,
                .stakeDelegate,
                .stakeUndelegate,
                .stakeRedelegate,
                .stakeRewards,
                .stakeWithdraw,
                .assetActivation,
                .smartContractCall,
                .perpetualOpenPosition,
                .perpetualClosePosition,
                .perpetualModifyPosition,
                .stakeFreeze,
                .stakeUnfreeze,
                .earnDeposit,
                .earnWithdraw:
            return .none
        case .swap:
            guard let metadata = transaction.transaction.metadata?.decode(TransactionSwapMetadata.self), let asset = transaction.assets.first(where: { $0.id == metadata.fromAsset }) else {
                return .none
            }
            return AmountDisplay.numeric(
                data: AssetValuePrice(asset: asset, value: BigInt(stringLiteral: metadata.fromValue), price: nil),
                style: AmountDisplayStyle(
                    sign: .outgoing,
                    formatter: formatter,
                    currencyCode: currency,
                    textStyle: .footnote
                )
            ).amount
        }
    }

    public var participant: String {
        switch transaction.transaction.direction {
        case .incoming: transaction.transaction.from
        case .outgoing, .selfTransfer: transaction.transaction.to
        }
    }

    public var viewOnAddressExplorerText: String { Localized.Transaction.viewOn(addressLink.name) }
    public var viewOnTransactionExplorerText: String { Localized.Transaction.viewOn(transactionLink.name) }

    public var addressExplorerUrl: URL { addressLink.url }
    public var transactionExplorerUrl: URL { transactionLink.url }

    private var transactionLink: BlockExplorerLink {
        explorerService.transactionLink(
            chain: assetId.chain,
            provider: transaction.transaction.swapProvider,
            hash: transaction.transaction.id.hash,
            recipient: transaction.transaction.to
        )
    }

    private var addressLink: BlockExplorerLink { explorerService.addressUrl(chain: assetId.chain, address: participant) }
    private var assetId: AssetId { transaction.transaction.assetId }
    
    public func getAddressName(address: String) -> AddressName? {
        if address == transaction.transaction.from {
            return transaction.fromAddress
        }

        if address == transaction.transaction.to {
            return transaction.toAddress
        }

        return .none
    }
    
    public func addressLink(account: SimpleAccount) -> BlockExplorerLink {
        explorerService.addressUrl(chain: account.chain, address: account.address)
    }
    
    // MARK: - Private methods
    
    private func getDisplayName(address: String, chain: Chain) -> String {
        if let name = getAddressName(address: address)?.name {
            return name
        }
        return AddressFormatter(address: address, chain: chain).value()
    }

    private func getResourceTitle() -> String? {
        guard let resourceType = transaction.transaction.metadata?.decode(TransactionResourceTypeMetadata.self)?.resourceType else { return nil }
        return ResourceViewModel(resource: resourceType).title
    }
}
