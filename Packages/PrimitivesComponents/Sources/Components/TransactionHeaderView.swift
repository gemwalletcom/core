// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Primitives
import Style

public enum TransactionHeaderType {
    case amount(AmountDisplay)
    case swap(from: SwapAmountField, to: SwapAmountField)
    case nft(name: String?, image: AssetImage)
    case asset(image: AssetImage)
    case assetValue(AssetValueHeaderData)
}

public struct TransactionHeaderView: View {
    public let type: TransactionHeaderType
    
    public init(type: TransactionHeaderType) {
        self.type = type
    }
    
    public var body: some View {
        VStack(alignment: .center) {
            switch type {
            case .amount(let display):
                WalletHeaderView(
                    model: TransactionAmountHeaderViewModel(display: display),
                    isPrivacyEnabled: .constant(false),
                    balanceActionType: .none,
                    onHeaderAction: nil,
                    onInfoAction: nil
                )
            case .swap(let from, let to):
                SwapAmountView(from: from, to: to)
            case .nft(let name, let image):
                VStack(spacing: .medium) {
                    NftImageView(assetImage: image)
                        .frame(width: .image.large, height: .image.large)
                        .cornerRadius(.image.large/4)
                    if let name {
                        Text(name)
                    }
                }
            case .asset(let image):
                AssetImageView(assetImage: image, size: .image.large)
                    .padding(.bottom, .space12)
            case .assetValue(let data):
                WalletHeaderView(
                    model: AssetValueHeaderViewModel(data: data),
                    isPrivacyEnabled: .constant(false),
                    balanceActionType: .none,
                    onHeaderAction: nil,
                    onInfoAction: nil
                )
            }
        }
        .frame(maxWidth: .infinity)
    }
}
