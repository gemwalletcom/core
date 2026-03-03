// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesComponents
import Localization
import Components

struct TransactionParticipantViewModel {
    private let transactionViewModel: TransactionViewModel

    init(transactionViewModel: TransactionViewModel) {
        self.transactionViewModel = transactionViewModel
    }
}

// MARK: - ItemModelProvidable

extension TransactionParticipantViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        switch transactionViewModel.transaction.transaction.type {
        case .stakeFreeze, .stakeUnfreeze: resourceItemModel
        case .transfer, .transferNFT, .tokenApproval, .smartContractCall, .stakeDelegate: participantItemModel
        case .swap, .stakeUndelegate, .stakeRedelegate, .stakeRewards, .stakeWithdraw, .assetActivation, .perpetualOpenPosition, .perpetualClosePosition, .perpetualModifyPosition, .earnDeposit, .earnWithdraw: .empty
        }
    }
}

// MARK: - Private

extension TransactionParticipantViewModel {
    private var participantItemModel: TransactionItemModel {
        guard transactionViewModel.participant.isNotEmpty,
              let participantTitle
        else {
            return .empty
        }

        let address = transactionViewModel.participant
        let chain = transactionViewModel.transaction.transaction.assetId.chain
        let addressName = transactionViewModel.getAddressName(address: address)
        let account = SimpleAccount(
            name: addressName?.name,
            chain: chain,
            address: address,
            assetImage: nil,
            addressType: addressName?.type
        )

        return .participant(
            TransactionParticipantItemModel(
                title: participantTitle,
                account: account,
                addressLink: transactionViewModel.addressLink(account: account)
            )
        )
    }

    private var resourceItemModel: TransactionItemModel {
        guard let resourceType = transactionViewModel.transaction.transaction.metadata?.decode(TransactionResourceTypeMetadata.self)?.resourceType else {
            return .empty
        }
        let resourceTitle = ResourceViewModel(resource: resourceType).title
        return .listItem(ListItemModel(title: Localized.Stake.resource, subtitle: resourceTitle))
    }

    private var participantTitle: String? {
        switch transactionViewModel.transaction.transaction.type {
        case .transfer, .transferNFT:
            switch transactionViewModel.transaction.transaction.direction {
            case .incoming: Localized.Transaction.sender
            case .outgoing, .selfTransfer: Localized.Transaction.recipient
            }
        case .tokenApproval:
            Localized.Asset.contract
        case .smartContractCall:
            switch transactionViewModel.transaction.transaction.metadata?.decode(TransactionWalletConnectMetadata.self)?.outputAction {
            case .send: Localized.Transaction.recipient
            case .sign, .none: Localized.Asset.contract
            }
        case .stakeDelegate:
            Localized.Stake.validator
        case .stakeFreeze, .stakeUnfreeze:
            Localized.Stake.resource
        case .swap, .stakeUndelegate, .stakeRedelegate, .stakeRewards, .stakeWithdraw,
                .assetActivation, .perpetualOpenPosition, .perpetualClosePosition, .perpetualModifyPosition, .earnDeposit, .earnWithdraw: nil
        }
    }
}
