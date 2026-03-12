// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Primitives

public struct SimulationPayloadFieldsContent: View {
    private let fields: [SimulationPayloadField]
    private let fieldViewModel: (SimulationPayloadField) -> SimulationPayloadFieldViewModel
    private let contextMenuItems: (SimulationPayloadField) -> [ContextMenuItemType]

    public init(
        fields: [SimulationPayloadField],
        fieldViewModel: @escaping (SimulationPayloadField) -> SimulationPayloadFieldViewModel,
        contextMenuItems: @escaping (SimulationPayloadField) -> [ContextMenuItemType]
    ) {
        self.fields = fields
        self.fieldViewModel = fieldViewModel
        self.contextMenuItems = contextMenuItems
    }

    public var body: some View {
        ForEach(Array(fields.enumerated()), id: \.offset) {
            let field = $0.element
            let viewModel = fieldViewModel(field)
            ListItemView(title: viewModel.title, subtitle: viewModel.subtitle)
                .contextMenu(contextMenuItems(field))
        }
    }
}
