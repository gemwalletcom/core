// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Primitives
import PrimitivesComponents
import Style
import Localization

public struct ManageContactScene: View {

    @Environment(\.dismiss) private var dismiss

    @State private var model: ManageContactViewModel

    @FocusState private var focusedField: Field?
    enum Field: Int, Hashable {
        case name
        case description
    }

    public init(model: ManageContactViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            contactSection
            addressesSection
        }
        .listStyle(.insetGrouped)
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button("", systemImage: SystemImage.checkmark, action: onSave)
                    .disabled(model.buttonState == .disabled)
            }
        }
        .onAppear {
            if model.isAddMode {
                focusedField = .name
            }
        }
        .navigationDestination(for: Scenes.ContactAddress.self) {
            contactAddressScene(mode: .edit($0.address))
        }
        .sheet(item: $model.isPresentingAddress) { mode in
            NavigationStack {
                contactAddressScene(mode: mode)
                    .toolbarDismissItem(type: .close, placement: .cancellationAction)
            }
        }
    }
    
    @ViewBuilder
    private func contactAddressScene(mode: ManageContactAddressViewModel.Mode) -> some View {
        ManageContactAddressScene(
            model: ManageContactAddressViewModel(
                contactId: model.contactId,
                nameService: model.nameService,
                mode: mode,
                onComplete: model.onAddressComplete
            )
        )
    }
}

// MARK: - UI Components

extension ManageContactScene {
    private var contactSection: some View {
        Section {
            InputValidationField(
                model: $model.nameInputModel,
                placeholder: model.nameTitle,
                allowClean: true
            )
            .focused($focusedField, equals: .name)
            .textInputAutocapitalization(.words)

            FloatTextField(
                model.descriptionTitle,
                text: $model.description,
                allowClean: true
            )
            .focused($focusedField, equals: .description)
        }
    }

    private var addressesSection: some View {
        Section {
            ForEach(model.addresses, id: \.id) { address in
                NavigationLink(value: Scenes.ContactAddress(address: address)) {
                    ListItemView(model: model.listItemModel(for: address))
                }
            }
            .onDelete(perform: model.deleteAddress)

            Button(action: onAddAddress) {
                HStack {
                    Images.System.plus
                    Text(Localized.Common.address)
                }
            }
        } header: {
            Text(model.addressesSectionTitle)
        }
    }
}

// MARK: - Actions

extension ManageContactScene {
    private func onAddAddress() {
        focusedField = .none
        model.isPresentingAddress = .add
    }

    private func onSave() {
        focusedField = .none
        model.onSave()
        dismiss()
    }
}

