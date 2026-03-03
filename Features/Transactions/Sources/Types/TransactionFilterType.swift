// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public enum TransactionFilterType: Int, CaseIterable, Sendable {
    case transfers
    case swaps
    case stake
    case smartContract
    case perpetuals
    case others
}

extension TransactionFilterType: Identifiable {
    public var id: String { rawValue.description }
}

extension TransactionFilterType: Comparable {
    public static func < (lhs: TransactionFilterType, rhs: TransactionFilterType) -> Bool {
        lhs.rawValue < rhs.rawValue
    }
}

extension TransactionType {
    public var filterType: TransactionFilterType {
        switch self {
        case .transfer, .transferNFT: .transfers
        case .smartContractCall: .smartContract
        case .swap, .tokenApproval: .swaps
        case .stakeDelegate, .stakeUndelegate, .stakeRewards, .stakeRedelegate, .stakeWithdraw, .stakeFreeze, .stakeUnfreeze: .stake
        case .assetActivation: .others
        case .perpetualOpenPosition, .perpetualClosePosition, .perpetualModifyPosition: .perpetuals
        case .earnDeposit, .earnWithdraw: .stake
        }
    }
}
