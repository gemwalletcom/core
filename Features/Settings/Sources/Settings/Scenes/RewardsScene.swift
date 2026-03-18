// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Primitives
import Localization
import Style
import PrimitivesComponents

public struct RewardsScene: View {
    @State private var model: RewardsViewModel

    public init(model: RewardsViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            switch model.state {
            case .loading:
                CenterLoadingView()
            case .error(let error):
                stateErrorView(error: error)
            case .data(let rewards):
                inviteFriendsSection(code: rewards.code)
                if let disableReason = model.disableReason {
                    disableReasonSection(reason: disableReason)
                }
                if model.hasPendingReferral {
                    pendingReferralSection
                }
                if model.isInfoEnabled {
                    infoSection(rewards: rewards)
                }
                if rewards.redemptionOptions.isNotEmpty {
                    redemptionOptionsSection(options: rewards.redemptionOptions)
                }
            case .noData:
                inviteFriendsSection(code: nil)
            }
        }
        .refreshable { await model.fetch() }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listStyle(.insetGrouped)
        .navigationTitle(model.title)
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                if model.showsWalletSelector {
                    WalletBarView(model: model.walletBarViewModel) {
                        model.isPresentingSheet = .walletSelector
                    }
                } else {
                    Button {
                        model.isPresentingSheet = .url(model.rewardsUrl)
                    } label: {
                        Images.System.info
                    }
                }
            }
        }
        .sheet(item: $model.isPresentingSheet) { sheet in
            switch sheet {
            case .walletSelector:
                SelectableListNavigationStack(
                    model: model.walletSelectorModel,
                    onFinishSelection: { wallets in
                        if let wallet = wallets.first {
                            model.selectWallet(wallet)
                        }
                        model.isPresentingSheet = nil
                    },
                    listContent: { wallet in
                        SimpleListItemView(model: wallet)
                    }
                )
            case .share:
                if let shareText = model.shareText {
                    ShareSheet(activityItems: [shareText])
                }
            case .createCode:
                TextInputScene(model: model.createCodeViewModel) {
                    model.isPresentingSheet = nil
                }
                .presentationDetents([.medium])
            case .activateCode(let code):
                TextInputScene(model: model.redeemCodeViewModel(code: code)) {
                    model.isPresentingSheet = nil
                }
                .presentationDetents([.medium])
            case .url(let url):
                SFSafariView(url: url)
            }
        }
        .taskOnce {
            Task { await model.onTaskOnce() }
        }
        .toast(message: $model.toastMessage)
        .alertSheet($model.isPresentingAlert)
    }

    @ViewBuilder
    private func stateErrorView(error: Error) -> some View {
        Section {
            StateEmptyView(
                title: model.errorTitle,
                description: error.localizedDescription,
                image: nil
            ) {
                Button(Localized.Common.tryAgain) {
                    Task { await model.fetch() }
                }
                .buttonStyle(.blue())
            }
        }
    }

    @ViewBuilder
    private func inviteFriendsSection(code: String?) -> some View {
        Section {
            VStack(spacing: Spacing.large) {
                Text("🎁")
                    .font(.app.extraLargeTitle)
                    .padding(.top, Spacing.medium)

                VStack(spacing: Spacing.small) {
                    Text(model.createCodeTitle)
                        .font(.title2.bold())
                        .multilineTextAlignment(.center)

                    Text(.init(model.createCodeDescription))
                        .textStyle(.calloutSecondary)
                        .multilineTextAlignment(.center)
                }

                HStack(spacing: Spacing.medium) {
                    featureItem(emoji: "👥", text: Localized.Rewards.InviteFriends.title)
                    featureItem(emoji: "💎", text: Localized.Rewards.EarnPoints.title)
                    featureItem(emoji: "🎉", text: Localized.Rewards.GetRewards.title)
                }

                Button {
                    if code != nil {
                        model.isPresentingSheet = .share
                    } else {
                        model.isPresentingSheet = .createCode
                    }
                } label: {
                    HStack(spacing: Spacing.small) {
                        if code != nil {
                            Images.System.share
                        }
                        Text(code != nil ? Localized.Rewards.InviteFriends.title : model.createCodeButtonTitle)
                    }
                }
                .buttonStyle(.blue())
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, Spacing.small)
        }

        if model.canUseReferralCode {
            Section {
                Button {
                    model.isPresentingSheet = .activateCode(code: "")
                } label: {
                    Text(model.activateCodeFooterTitle)
                        .frame(maxWidth: .infinity)
                }
            } footer: {
                Text(model.activateCodeFooterDescription)
            }
        }
    }

    @ViewBuilder
    private func featureItem(emoji: String, text: String) -> some View {
        VStack(spacing: Spacing.extraSmall) {
            Text(emoji)
                .font(.title2)
            Text(text)
                .font(.caption)
                .foregroundStyle(Colors.secondaryText)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity)
    }

    @ViewBuilder
    private func redemptionOptionsSection(options: [RewardRedemptionOption]) -> some View {
        Section {
            ForEach(options.map { RewardRedemptionOptionViewModel(option: $0) }) { viewModel in
                NavigationCustomLink(
                    with: ListItemView(
                        title: viewModel.title,
                        subtitle: viewModel.subtitle,
                        imageStyle: .asset(assetImage: viewModel.assetImage)
                    )
                ) {
                    if model.canRedeem(option: viewModel.option) {
                        model.showRedemptionAlert(for: viewModel.option)
                    } else {
                        model.showError(Localized.Rewards.insufficientPoints)
                    }
                }
            }
        } header: {
            Text(Localized.Rewards.WaysSpend.title)
        }
    }

    @ViewBuilder
    private func infoSection(rewards: Rewards) -> some View {
        Section {
            if let code = rewards.code {
                ListItemView(
                    title: model.myReferralCodeTitle,
                    subtitle: code
                )
                .contextMenu(model.referralLink.map { [.copy(value: $0)] } ?? [])
            }
            ListItemView(
                title: model.referralCountTitle,
                subtitle: "\(rewards.referralCount)"
            )
            ListItemView(
                title: model.pointsTitle,
                subtitle: "\(rewards.points) 💎"
            )
            if let invitedBy = rewards.usedReferralCode {
                ListItemView(
                    title: model.invitedByTitle,
                    subtitle: invitedBy
                )
            }
        } header: {
            Text(model.statsSectionTitle)
        }
    }

    @ViewBuilder
    private func disableReasonSection(reason: String) -> some View {
        Section {
            ListItemErrorView(
                errorTitle: model.errorTitle,
                error: AnyError(reason)
            )
        }
    }

    @ViewBuilder
    private var pendingReferralSection: some View {
        Section {
            ListItemInfoView(
                title: model.pendingReferralTitle,
                description: model.pendingReferralDescription
            )

            HStack {
                Spacer()
                StateButton(
                    text: model.pendingReferralButtonTitle,
                    type: model.activatePendingButtonType
                ) {
                    Task { await model.activatePendingReferral() }
                }
                .frame(height: .scene.button.height)
                .frame(maxWidth: .scene.button.maxWidth)
                Spacer()
            }
        }
    }
}
