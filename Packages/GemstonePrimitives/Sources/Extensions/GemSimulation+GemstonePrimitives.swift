// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import BigInt
import enum Gemstone.SimulationSeverity
import enum Gemstone.SimulationWarningType
import enum Gemstone.SimulationPayloadFieldDisplay
import enum Gemstone.SimulationPayloadFieldKind
import enum Gemstone.SimulationPayloadFieldType
import struct Gemstone.SimulationWarning
import struct Gemstone.SimulationBalanceChange
import struct Gemstone.SimulationHeader
import struct Gemstone.SimulationPayloadField
import struct Gemstone.SimulationResult
import Primitives

extension Gemstone.SimulationSeverity {
    public func map() -> Primitives.SimulationSeverity {
        switch self {
        case .low: .low
        case .warning: .warning
        case .critical: .critical
        }
    }
}

extension Gemstone.SimulationWarningType {
    public func map() throws -> Primitives.SimulationWarningType {
        switch self {
        case .tokenApproval(let assetId, let value): .tokenApproval(assetId: try AssetId(id: assetId), value: try value.map { try BigInt.from(string: $0) })
        case .suspiciousSpender: .suspiciousSpender
        case .externallyOwnedSpender: .externallyOwnedSpender
        case .nftCollectionApproval(let assetId): .nftCollectionApproval(assetId: try AssetId(id: assetId))
        case .permitApproval(let assetId, let value): .permitApproval(assetId: try AssetId(id: assetId), value: try value.map { try BigInt.from(string: $0) })
        case .permitBatchApproval(let value): .permitBatchApproval(value: try value.map { try BigInt.from(string: $0) })
        case .validationError: .validationError
        }
    }
}

extension Gemstone.SimulationPayloadFieldType {
    public func map() -> Primitives.SimulationPayloadFieldType {
        switch self {
        case .text: .text
        case .address: .address
        case .timestamp: .timestamp
        }
    }
}

extension Primitives.SimulationPayloadFieldType {
    public func map() -> Gemstone.SimulationPayloadFieldType {
        switch self {
        case .text: .text
        case .address: .address
        case .timestamp: .timestamp
        }
    }
}

extension Gemstone.SimulationPayloadFieldDisplay {
    public func map() -> Primitives.SimulationPayloadFieldDisplay {
        switch self {
        case .primary: .primary
        case .secondary: .secondary
        }
    }
}

extension Primitives.SimulationPayloadFieldDisplay {
    public func map() -> Gemstone.SimulationPayloadFieldDisplay {
        switch self {
        case .primary: .primary
        case .secondary: .secondary
        }
    }
}

extension Gemstone.SimulationPayloadFieldKind {
    public func map() -> Primitives.SimulationPayloadFieldKind {
        switch self {
        case .contract: .contract
        case .method: .method
        case .token: .token
        case .spender: .spender
        case .value: .value
        case .custom: .custom
        }
    }
}

extension Primitives.SimulationPayloadFieldKind {
    public func map() -> Gemstone.SimulationPayloadFieldKind {
        switch self {
        case .contract: .contract
        case .method: .method
        case .token: .token
        case .spender: .spender
        case .value: .value
        case .custom: .custom
        }
    }
}

extension Gemstone.SimulationWarning {
    public func map() throws -> Primitives.SimulationWarning {
        Primitives.SimulationWarning(severity: severity.map(), warning: try warning.map(), message: message)
    }
}

extension Gemstone.SimulationBalanceChange {
    public func map() throws -> Primitives.SimulationBalanceChange {
        Primitives.SimulationBalanceChange(assetId: try AssetId(id: assetId), value: value)
    }
}

extension Gemstone.SimulationPayloadField {
    public func map() -> Primitives.SimulationPayloadField {
        Primitives.SimulationPayloadField(kind: kind.map(), label: label, value: value, fieldType: fieldType.map(), display: display.map())
    }
}

extension Primitives.SimulationPayloadField {
    public func map() -> Gemstone.SimulationPayloadField {
        Gemstone.SimulationPayloadField(kind: kind.map(), label: label, value: value, fieldType: fieldType.map(), display: display.map())
    }
}

extension Gemstone.SimulationHeader {
    public func map() throws -> Primitives.SimulationHeader {
        Primitives.SimulationHeader(assetId: try AssetId(id: assetId), value: value)
    }
}

extension Gemstone.SimulationResult {
    public func map() throws -> Primitives.SimulationResult {
        Primitives.SimulationResult(
            warnings: try warnings.map { try $0.map() },
            balanceChanges: try balanceChanges.map { try $0.map() },
            payload: payload.map { $0.map() },
            header: try header.map { try $0.map() }
        )
    }
}
