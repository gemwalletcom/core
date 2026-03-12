// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Localization
import Primitives

public struct SimulationPayloadFieldViewModel: Identifiable {
    public let field: SimulationPayloadField
    public let chain: Chain
    public let addressName: AddressName?
    public let relativeDateFormatter: RelativeDateFormatter

    public init(
        field: SimulationPayloadField,
        chain: Chain,
        addressName: AddressName? = nil,
        relativeDateFormatter: RelativeDateFormatter = RelativeDateFormatter()
    ) {
        self.field = field
        self.chain = chain
        self.addressName = addressName
        self.relativeDateFormatter = relativeDateFormatter
    }

    public var id: SimulationPayloadField {
        field
    }

    public var title: String {
        if field.kind == .custom {
            return field.label ?? ""
        }

        switch field.kind {
        case .contract:
            return Localized.Asset.contract
        case .method:
            return Localized.Common.method
        case .token:
            return Localized.Common.token
        case .spender:
            return Localized.Transfer.to
        case .value:
            return Localized.Perpetual.value
        case .custom:
            return field.label ?? ""
        }
    }

    public var subtitle: String {
        switch field.fieldType {
        case .address:
            let address = AddressFormatter(address: field.value, chain: chain).value()
            guard let addressName, addressName.name.isNotEmpty, addressName.name != field.value else {
                return address
            }
            return "\(addressName.name) (\(address))"
        case .timestamp:
            return relativeDateFormatter.string(fromTimestampValue: field.value)
        case .text:
            return field.value
        }
    }

    public var contextMenuItems: [ContextMenuItemType] {
        field.fieldType == .address ? [.copy(value: field.value)] : []
    }

}
