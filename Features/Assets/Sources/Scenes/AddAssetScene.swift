// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import QRScanner
import Primitives
import Style
import PrimitivesComponents
import Localization

public struct AddAssetScene: View {
    @State private var model: AddAssetSceneViewModel
    @State private var networksModel: NetworkSelectorViewModel
    @State private var isPresentingUrl: URL?

    @FocusState private var focusedField: Field?
    enum Field: Int, Hashable {
        case address
    }

    var action: ((Asset) -> Void)?

    public init(model: AddAssetSceneViewModel, action: ((Asset) -> Void)? = nil) {
        _model = State(initialValue: model)
        _networksModel = State(initialValue: NetworkSelectorViewModel(state: .data(.plain(model.chains))))
        self.action = action
    }

    public var body: some View {
        addTokenList
            .safeAreaButton {
                StateButton(
                    text: model.actionButtonTitle,
                    type: .primary(model.state),
                    action: onSelectImportToken
                )
            }
            .toolbarInfoButton(url: model.customTokenUrl)
            .onAppear {
                focusedField = .address
            }
            .onChange(of: model.input.address, onAddressClean)
            .listSectionSpacing(.compact)
            .navigationTitle(model.title)
            .navigationDestination(for: Scenes.NetworksSelector.self) { _ in
                NetworkSelectorScene(
                    model: $networksModel,
                    onFinishSelection: onFinishChainSelection(chains:)
                )
            }
            .sheet(isPresented: $model.isPresentingScanner) {
                ScanQRCodeNavigationStack(action: onHandleScan(_:))
            }
            .safariSheet(url: $isPresentingUrl)
    }
}

// MARK: - UI Components

extension AddAssetScene {
    @ViewBuilder
    private var addTokenList: some View {
        List {
            if let chain = model.input.chain {
                Section(model.networkTitle) {
                    if model.input.hasManyChains {
                        NavigationLink(value: Scenes.NetworksSelector()) {
                            ChainView(model: ChainViewModel(chain: chain))
                        }
                    } else {
                        ChainView(model: ChainViewModel(chain: chain))
                    }
                }
            }
            Section {
                FloatTextField(model.addressTitleField, text: model.addressBinding) {
                    HStack(spacing: .small) {
                        ListButton(image: model.pasteImage, action: onSelectPaste)
                        ListButton(image: model.qrImage, action: onSelectScan)
                    }
                }
                .focused($focusedField, equals: .address)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .submitLabel(.search)
                .onSubmit(fetch)
            }

            switch model.state {
            case .noData:
                EmptyView()
            case .loading:
                ListItemLoadingView()
                    .id(UUID())
            case .data(let asset):
                Section {
                    ListItemView(title: asset.nameTitle, subtitle: asset.name)
                    ListItemView(title: asset.symbolTitle, subtitle: asset.symbol)
                    ListItemView(title: asset.decimalsTitle, subtitle: asset.decimals)
                    ListItemView(title: asset.typeTitle, subtitle: asset.type)
                }
                if let url = asset.explorerUrl, let text = asset.explorerText {
                    Section {
                        SafariNavigationLink(url: url) {
                            ListItemView(title: text)
                        }
                    }
                }
                Section {
                    ListItemView(
                        title: Localized.Asset.Verification.warningTitle,
                        titleStyle: .headline,
                        titleExtra: Localized.Asset.Verification.warningMessage,
                        titleStyleExtra: .bodySecondary,
                        imageStyle: model.warningImageStyle
                    ) {
                        isPresentingUrl = model.tokenVerificationUrl
                    }
                }
            case .error(let error):
                ListItemErrorView(
                    errorTitle: model.errorTitle,
                    errorSystemNameImage: model.errorSystemImage,
                    error: error
                )
            }
        }
    }
}

// MARK: - Actions

extension AddAssetScene {
    private func onFinishChainSelection(chains: [Chain]) {
        model.input.chain = chains.first
        onAddressClean(nil, nil)
    }

    private func onSelectImportToken() {
        guard case let .data(asset) = model.state else { return }
        action?(asset.asset)
    }

    private func onSelectScan() {
        model.isPresentingScanner = true
    }

    private func onSelectPaste() {
        guard let address = UIPasteboard.general.string else { return }
        model.input.address = address
        focusedField = nil
        fetch()
    }

    private func onHandleScan(_ result: String) {
        model.input.address = result
        focusedField = nil
        fetch()
    }

    private func onAddressClean(_ oldValue: String?, _ newValue: String?) {
        guard newValue == nil else { return }
        model.input.address = newValue
        fetch()
    }
}

// MARK: - Effects

extension AddAssetScene {
    private func fetch() {
        Task {
            await model.fetch()
        }
    }
}
