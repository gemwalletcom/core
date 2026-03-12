// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Style
import PrimitivesComponents
import Components
import Localization
import Primitives
import Store

public struct WalletImageScene: View {
    enum Tab: Equatable {
        case emoji, collections
    }

    @Environment(\.dismiss) private var dismiss
    @State private var selectedTab: Tab = .emoji
    @State private var model: WalletImageViewModel

    public init(model: WalletImageViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        VStack {
            if let dbWallet = model.dbWallet {
                AvatarView(
                    avatarImage: WalletViewModel(wallet: dbWallet).avatarImage,
                    size: model.emojiViewSize,
                    action: setDefaultAvatar
                )
                .padding(.top, .medium)
                .padding(.bottom, .extraLarge)
            }
            switch model.source {
            case .onboarding:
                listView
            case .wallet:
                pickerView
                    .padding(.bottom, .medium)
                    .padding(.horizontal, .medium)
                listView
            }
        }
        .bindQuery(model.walletQuery, model.nftQuery)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .background(Colors.grayBackground)
    }
    
    private var pickerView: some View {
        Picker("", selection: $selectedTab) {
            Text(Localized.Common.emoji).tag(Tab.emoji)
            Text(Localized.Nft.collections).tag(Tab.collections)
        }
        .pickerStyle(.segmented)
    }
    
    @ViewBuilder
    private var listView: some View {
        ScrollView {
            LazyVGrid(
                columns: model.getColumns(for: selectedTab),
                alignment: .center,
                spacing: .medium
            ) {
                switch selectedTab {
                case .emoji:
                    emojiListView
                case .collections:
                    nftAssetListView
                }
            }
            .padding(.horizontal, .medium)
        }
        .overlay {
            if model.nftDataList.isEmpty, case .collections = selectedTab {
                EmptyContentView(model: model.emptyContentModel)
            }
        }
    }
    
    private var emojiListView: some View {
        ForEach(model.emojiList) { value in
            NavigationCustomLink(
                with: EmojiView(color: value.color, emoji: value.emoji)
            ) {
                model.setAvatarEmoji(value: value)
                onDismiss()
            }
            .frame(maxWidth: .infinity)
            .transition(.opacity)
        }
    }
    
    private var nftAssetListView: some View {
        ForEach(model.buildNftAssetsItems(from: model.nftDataList)) { item in
            let view = GridPosterView(model: GridPosterViewModel(assetImage: item.assetImage, title: nil))
            NavigationCustomLink(with: view) {
                onSelectNftAsset(item)
            }
        }
    }
}

// MARK: - Actions

private extension WalletImageScene {
    func onSelectNftAsset(_ item: WalletImageViewModel.NFTAssetImageItem) {
        guard let url = item.assetImage.imageURL else {
            return
        }
        Task {
            await model.setImage(from: url)
        }
    }
    
    func setDefaultAvatar() {
        model.setDefaultAvatar()
        onDismiss()
    }
    
    func onDismiss() {
        switch model.source {
        case .onboarding: dismiss()
        case .wallet: break
        }
    }
}
