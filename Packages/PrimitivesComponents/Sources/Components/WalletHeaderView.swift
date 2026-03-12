// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Style
import Components
import Localization
import Primitives

public struct WalletHeaderView: View {
    private let model: any HeaderViewModel

    @Binding var isPrivacyEnabled: Bool

    private let balanceActionType: BalanceActionType
    private let onHeaderAction: HeaderButtonAction?
    private let onSubtitleAction: VoidAction
    private let onInfoAction: VoidAction

    public init(
        model: any HeaderViewModel,
        isPrivacyEnabled: Binding<Bool>,
        balanceActionType: BalanceActionType,
        onHeaderAction: HeaderButtonAction?,
        onSubtitleAction: VoidAction = nil,
        onInfoAction: VoidAction
    ) {
        self.model = model
        _isPrivacyEnabled = isPrivacyEnabled
        self.balanceActionType = balanceActionType
        self.onHeaderAction = onHeaderAction
        self.onSubtitleAction = onSubtitleAction
        self.onInfoAction = onInfoAction
    }

    public var body: some View {
        VStack(spacing: .zero) {
            if let assetImage = model.assetImage {
                AssetImageView(
                    assetImage: assetImage,
                    size: .image.semiLarge
                )
                .padding(.bottom, .space12)
            }
            balanceView
            .numericTransition(for: model.title)
            .minimumScaleFactor(0.5)
            .font(.app.largeTitle)
            .foregroundStyle(Colors.black)
            .lineLimit(1)
            .padding(.bottom, .space10)

            if let subtitle = model.subtitle {
                subtitleView(subtitle)
                    .numericTransition(for: model.subtitle)
                    .padding(.bottom, .space10)
            }

            switch model.isWatchWallet {
            case true:
                Button {
                    onInfoAction?()
                } label: {
                    HStack {
                        Images.System.eye

                        Text(Localized.Wallet.Watch.Tooltip.title)
                            .foregroundStyle(Colors.black)
                            .font(.callout)

                        Images.System.info
                            .tint(Colors.black)
                    }
                    .padding()
                    .background(Colors.grayDarkBackground)
                    .cornerRadius(.medium)
                    .padding(.top, .space10)
                }

            case false:
                HeaderButtonsView(buttons: model.buttons, action: onHeaderAction)
                    .padding(.top, .space8)
            }
        }
    }

    @ViewBuilder
    private func subtitleView(_ subtitle: String) -> some View {
        let content = HStack(spacing: Spacing.space6) {
            PrivacyText(
                subtitle,
                isEnabled: $isPrivacyEnabled
            )
            .font(.app.headline)
            .foregroundStyle(model.subtitleColor)

            if let subtitleImage = model.subtitleImage {
                subtitleImage
                    .font(.footnote)
                    .foregroundStyle(Colors.secondaryText)
            }
        }

        if let onSubtitleAction {
            Button(action: onSubtitleAction) {
                content
            }
        } else {
            content
        }
    }

    @ViewBuilder
    private var balanceView: some View {
        switch balanceActionType {
        case .privacyToggle:
            PrivacyToggleView(model.title, isEnabled: $isPrivacyEnabled)
        case .action(let action):
            Button(action: action) {
                PrivacyText(model.title, isEnabled: $isPrivacyEnabled)
            }
        case .none:
            Text(model.title)
        }
    }
}

// MARK: - Previews

#Preview {
    let model = WalletHeaderViewModel(
        walletType: .multicoin,
        totalValue: TotalFiatValue(value: 1_000, pnlAmount: 50, pnlPercentage: 5.26),
        currencyCode: Currency.usd.rawValue,
        bannerEventsViewModel: HeaderBannerEventViewModel(events: [])
    )

    WalletHeaderView(
        model: model,
        isPrivacyEnabled: .constant(false),
        balanceActionType: .privacyToggle,
        onHeaderAction: .none,
        onInfoAction: .none
    )
}
