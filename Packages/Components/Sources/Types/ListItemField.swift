// Copyright (c). Gem Wallet. All rights reserved.

import Style

public struct ListItemField {
    public let title: TextValue
    public let value: TextValue

    public init(title: TextValue, value: TextValue) {
        self.title = title
        self.value = value
    }

    public init(title: String, value: String) {
        self.init(
            title: TextValue(text: title, style: .body),
            value: TextValue(text: value, style: .calloutSecondary)
        )
    }
}
