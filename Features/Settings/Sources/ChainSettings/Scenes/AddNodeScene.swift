// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Style
import Localization
import Components
import QRScanner
import PrimitivesComponents

struct AddNodeScene: View {
    @Environment(\.dismiss) private var dismiss

    @State private var model: AddNodeSceneViewModel
    @FocusState private var focusedField: Field?
    enum Field: Int, Hashable {
        case address
    }

    private let onDismiss: (() -> Void)?

    init(model: AddNodeSceneViewModel, onDismiss: (() -> Void)? = nil) {
        _model = State(initialValue: model)
        self.onDismiss = onDismiss
    }

    var body: some View {
        List {
            networkSection
            inputView
            nodeInfoView
        }
        .onChange(of: model.urlInputModel.text) {
            model.onChangeInput()
        }
        .debouncedTask(id: model.fetchTrigger) {
            await model.fetch()
        }
        .safeAreaButton {
            StateButton(
                text: model.actionButtonTitle,
                type: .primary(model.state),
                action: onSelectImport
            )
        }
        .onAppear {
            focusedField = .address
        }
        .frame(maxWidth: .infinity)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbarDismissItem(type: .close, placement: .topBarLeading)
        .sheet(isPresented: $model.isPresentingScanner) {
            ScanQRCodeNavigationStack(action: onHandleScan(_:))
        }
        .alertSheet($model.isPresentingAlertMessage)
    }
}

// MARK: - UI Components

extension AddNodeScene {
    private var networkSection: some View {
        Section(Localized.Transfer.network) {
            ChainView(model: model.chainModel)
        }
    }

    @ViewBuilder
    private var inputView: some View {
        Section {
            InputValidationField(
                model: $model.urlInputModel,
                placeholder: model.inputFieldTitle,
                onClean: { model.fetchTrigger = nil }
            ) {
                HStack(spacing: .small) {
                    ListButton(image: Images.System.paste, action: onSelectPaste)
                    ListButton(image: Images.System.qrCodeViewfinder, action: onSelectScan)
                }
            }
            .focused($focusedField, equals: .address)
            .autocorrectionDisabled()
            .textInputAutocapitalization(.never)
            .submitLabel(.done)
            .onSubmit(onSubmitUrl)
        }
        if case let .error(error) = model.state {
            ListItemErrorView(errorTitle: model.errorTitle, error: error)
        }
    }

    @ViewBuilder
    private var nodeInfoView: some View {
        switch model.state {
        case .noData, .loading, .error:
            EmptyView()
        case let .data(result):
            Section {
                ListItemView(field: result.chainIdField)
                ListItemView(field: result.inSyncField)
                ListItemView(field: result.latestBlockField)
                ListItemView(field: result.latencyField)
            }
            warningSection
        }
    }

    private var warningSection: some View {
        Section {
            ListItemView(model: model.warningModel)
        }
    }
}

// MARK: - Actions

extension AddNodeScene {
    private func onSelectDone() {
        dismiss()
    }

    private func onSubmitUrl() {
        focusedField = nil
        Task {
            await model.fetch()
        }
    }

    private func onSelectPaste() {
        guard let content = UIPasteboard.general.string else { return }
        model.setInput(content.trim())
        focusedField = nil
    }

    private func onSelectImport() {
        do {
            try model.importFoundNode()
            onDismiss?()
        } catch {
            model.isPresentingAlertMessage = AlertMessage(message: error.localizedDescription)
        }
    }

    private func onHandleScan(_ result: String) {
        model.setInput(result)
        focusedField = nil
    }

    private func onSelectScan() {
        model.isPresentingScanner = true
    }
}
