// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Primitives
import Store
import Style
import InfoSheet
import PrimitivesComponents
import Localization

public struct WalletScene: View {
    private var model: WalletSceneViewModel

    public init(model: WalletSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        @Bindable var preferences = model.observablePreferences

        List {
            Section { } header: {
                WalletHeaderView(
                    model: model.walletHeaderModel,
                    isPrivacyEnabled: $preferences.isHideBalanceEnabled,
                    balanceActionType: .privacyToggle,
                    onHeaderAction: model.onHeaderAction,
                    onSubtitleAction: model.onSelectPortfolio,
                    onInfoAction: model.onSelectWatchWalletInfo
                )
                .padding(.top, .space6)
            }
            .cleanListRow()

            if model.showPerpetuals {
                Section {
                    PerpetualsPreviewView(wallet: model.wallet)
                } header: {
                    HeaderNavigationLinkView(title: model.perpetualsTitle, destination: Scenes.Perpetuals())
                }
                .listRowInsets(.assetListRowInsets)
            }

            if let banner = model.walletBannersModel.allBanners.first {
                Section {
                    BannerView(
                        banner: banner,
                        action: model.onBanner
                    )
                }
                .listRowInsets(.zero)
            }

            if model.showPinnedSection {
                Section {
                    WalletAssetsList(
                        assets: model.sections.pinned,
                        currencyCode: model.currencyCode,
                        onHideAsset: model.onHideAsset,
                        onPinAsset: model.onPinAsset,
                        onCopyAddress: model.onCopyAddress,
                        showBalancePrivacy: $preferences.isHideBalanceEnabled
                    )
                } header: {
                    PinnedSectionHeader()
                }
                .listRowInsets(.assetListRowInsets)
            }

            Section {
                WalletAssetsList(
                    assets: model.sections.assets,
                    currencyCode: model.currencyCode,
                    onHideAsset: model.onHideAsset,
                    onPinAsset: model.onPinAsset,
                    onCopyAddress: model.onCopyAddress,
                    showBalancePrivacy: $preferences.isHideBalanceEnabled
                )
            } header: {
                if model.isLoadingAssets {
                    LoadingTextView(isAnimating: .constant(true))
                        .listRowInsets(.assetListRowInsets)
                        .textCase(nil)
                }
            } footer: {
                ListButton(
                    title: model.manageTokenTitle,
                    image: model.manageImage,
                    action: model.onSelectManage
                )
                .accessibilityIdentifier("manage")
                .padding(.medium)
                .frame(maxWidth: .infinity, alignment: .center)
            }
            .listRowInsets(.assetListRowInsets)
        }
        .id(model.wallet.id)
        .refreshable {
            model.fetch()
        }
        .taskOnce {
            model.fetch()
        }
    }
}
