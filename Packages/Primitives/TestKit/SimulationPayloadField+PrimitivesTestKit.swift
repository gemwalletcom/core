// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public extension SimulationPayloadField {
    static func standard(
        kind: SimulationPayloadFieldKind,
        value: String,
        fieldType: SimulationPayloadFieldType,
        display: SimulationPayloadFieldDisplay = .secondary
    ) -> Self {
        if kind == .custom {
            preconditionFailure("Use custom(label:value:fieldType:) for custom payload fields")
        }
        return Self(kind: kind, label: nil, value: value, fieldType: fieldType, display: display)
    }

    static func custom(
        label: String,
        value: String,
        fieldType: SimulationPayloadFieldType,
        display: SimulationPayloadFieldDisplay = .secondary
    ) -> Self {
        precondition(!label.isEmpty, "Custom payload fields require a label")
        return Self(kind: .custom, label: label, value: value, fieldType: fieldType, display: display)
    }
}
