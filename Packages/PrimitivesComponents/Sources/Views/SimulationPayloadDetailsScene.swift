// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Localization
import Primitives
import Style

public struct SimulationPayloadDetailsScene: View {
    @Environment(\.dismiss) private var dismiss

    private let primaryFields: [SimulationPayloadField]
    private let secondaryFields: [SimulationPayloadField]
    private let fieldViewModel: (SimulationPayloadField) -> SimulationPayloadFieldViewModel
    private let contextMenuItems: (SimulationPayloadField) -> [ContextMenuItemType]
    private let actionTitle: String?
    private let actionDestination: AnyView?

    public init(
        primaryFields: [SimulationPayloadField],
        secondaryFields: [SimulationPayloadField],
        fieldViewModel: @escaping (SimulationPayloadField) -> SimulationPayloadFieldViewModel,
        contextMenuItems: @escaping (SimulationPayloadField) -> [ContextMenuItemType],
        actionTitle: String? = nil,
        actionDestination: AnyView? = nil
    ) {
        self.primaryFields = primaryFields
        self.secondaryFields = secondaryFields
        self.fieldViewModel = fieldViewModel
        self.contextMenuItems = contextMenuItems
        self.actionTitle = actionTitle
        self.actionDestination = actionDestination
    }

    public var body: some View {
        List {
            if !primaryFields.isEmpty {
                Section {
                    fieldsView(primaryFields)
                }
            }

            if !secondaryFields.isEmpty {
                Section(Localized.Common.details) {
                    fieldsView(secondaryFields)
                }
            }

            if let actionTitle, let actionDestination {
                Section {
                    NavigationLink {
                        actionDestination
                    } label: {
                        ListItemView(title: actionTitle)
                    }
                }
            }
        }
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Button("", systemImage: SystemImage.checkmark, action: { dismiss() })
            }
        }
        .navigationTitle(Localized.Common.details)
        .navigationBarTitleDisplayMode(.inline)
        .listStyle(.insetGrouped)
        .listRowSpacing(.zero)
        .listSectionSpacing(.compact)
    }

    @ViewBuilder
    private func fieldsView(_ fields: [SimulationPayloadField]) -> some View {
        SimulationPayloadFieldsContent(
            fields: fields,
            fieldViewModel: fieldViewModel,
            contextMenuItems: contextMenuItems
        )
    }
}
