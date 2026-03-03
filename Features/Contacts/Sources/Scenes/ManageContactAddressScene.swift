// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Primitives
import PrimitivesComponents
import Style
import GemstonePrimitives
import QRScanner

public struct ManageContactAddressScene: View {

    @Environment(\.dismiss) private var dismiss

    @State private var model: ManageContactAddressViewModel

    @FocusState private var focusedField: Field?
    enum Field: Int, Hashable {
        case address
        case memo
    }

    public init(model: ManageContactAddressViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            chainSection
            addressSection
            if model.showMemo {
                memoSection
            }
        }
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button("", systemImage: SystemImage.checkmark, action: onComplete)
                    .disabled(model.buttonState == .disabled)
            }
        }
        .listStyle(.insetGrouped)
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            focusedField = .address
        }
        .sheet(isPresented: $model.isPresentingScanner) {
            ScanQRCodeNavigationStack(action: onScan)
        }
        .navigationDestination(for: Scenes.NetworksSelector.self) { _ in
            ChainSelectorView(
                chain: model.chain,
                onSelectChain: model.onSelectChain
            )
        }
    }
}

// MARK: - UI Components

extension ManageContactAddressScene {
    private var chainSection: some View {
        Section(model.networkTitle) {
            NavigationLink(value: Scenes.NetworksSelector()) {
                ChainView(model: ChainViewModel(chain: model.chain))
            }
        }
    }

    private var addressSection: some View {
        Section {
            AddressInputView(
                model: $model.addressInputModel,
                onSelectScan: model.onSelectScan
            )
            .focused($focusedField, equals: .address)
        }
    }

    private var memoSection: some View {
        Section {
            FloatTextField(
                model.memoTitle,
                text: $model.memo,
                allowClean: true
            )
            .focused($focusedField, equals: .memo)
            .textInputAutocapitalization(.never)
            .autocorrectionDisabled()
        }
    }
}

// MARK: - Actions

extension ManageContactAddressScene {
    private func onScan(_ result: String) {
        model.onHandleScan(result)
        focusedField = nil
    }

    private func onComplete() {
        focusedField = nil
        model.complete()
        dismiss()
    }
}
