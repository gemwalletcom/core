import Foundation
import BigInt

public enum ApprovalValue: Sendable, Equatable, Hashable {
    case exact(BigInt)
    case unlimited

    private static let unlimitedMarker = "Unlimited"

    public init?(rawValue: String) {
        switch rawValue {
        case Self.unlimitedMarker:
            self = .unlimited
        default:
            guard let value = BigInt(rawValue, radix: 10) else {
                return nil
            }
            self = .exact(value)
        }
    }

    public var rawValue: String {
        switch self {
        case .exact(let value):
            value.description
        case .unlimited:
            Self.unlimitedMarker
        }
    }
}
