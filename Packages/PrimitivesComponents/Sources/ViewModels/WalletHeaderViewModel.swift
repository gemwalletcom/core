// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives
import Style
import Components
import Formatters

public struct WalletHeaderViewModel {
    private let walletType: WalletType
    private let totalValue: TotalFiatValue
    private let bannerEventsViewModel: HeaderBannerEventViewModel
    private let totalValueViewModel: TotalValueViewModel

    public init(
        walletType: WalletType,
        totalValue: TotalFiatValue,
        currencyCode: String,
        bannerEventsViewModel: HeaderBannerEventViewModel
    ) {
        self.walletType = walletType
        self.totalValue = totalValue
        self.bannerEventsViewModel = bannerEventsViewModel
        let formatter = CurrencyFormatter(type: .currency, currencyCode: currencyCode)
        self.totalValueViewModel = TotalValueViewModel(totalValue: totalValue, currencyFormatter: formatter)
    }
}

// MARK: - HeaderViewModel

extension WalletHeaderViewModel: HeaderViewModel {
    public var isWatchWallet: Bool { walletType == .view }
    public var title: String { totalValueViewModel.title }
    public var assetImage: AssetImage? { .none }
    public var subtitle: String? {
        guard let amount = totalValueViewModel.pnlAmountText else { return nil }
        guard let percentage = totalValueViewModel.pnlPercentageText else { return amount }
        return "\(amount) (\(percentage))"
    }
    public var subtitleColor: Color { totalValueViewModel.pnlColor }

    public var subtitleImage: Image? {
        Image(systemName: SystemImage.chartLineUptrendXyaxis)
    }

    public var buttons: [HeaderButton] {
        [
            HeaderButton(type: .send, isEnabled: bannerEventsViewModel.isButtonsEnabled),
            HeaderButton(type: .receive, isEnabled: bannerEventsViewModel.isButtonsEnabled),
            HeaderButton(type: .buy, isEnabled: bannerEventsViewModel.isButtonsEnabled)
        ]
    }
}
