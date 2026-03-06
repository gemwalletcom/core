// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Style

public struct GridItemView: View {
    private let title: TextValue
    private let value: TextValue
    private let valueAlignment: HorizontalAlignment

    public init(
        title: TextValue,
        value: TextValue,
        valueAlignment: HorizontalAlignment = .trailing
    ) {
        self.title = title
        self.value = value
        self.valueAlignment = valueAlignment
    }

    public init(field: ListItemField, valueAlignment: HorizontalAlignment = .trailing) {
        self.init(title: field.title, value: field.value, valueAlignment: valueAlignment)
    }

    public var body: some View {
        GridRow {
            Text(title.text)
                .textStyle(title.style)
            Text(value.text)
                .textStyle(value.style)
                .gridColumnAlignment(valueAlignment)
        }
    }
}
