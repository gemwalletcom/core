public extension Array where Element == SimulationWarning {
    var visibleWarnings: [SimulationWarning] {
        if let blockingWarning = first(where: { $0.warning.suppressesOtherWarnings }) {
            return [blockingWarning]
        }

        if let unlimitedWarning = first(where: { $0.warning.isUnlimitedWarning }) {
            return [unlimitedWarning]
        }

        return self
    }
}

private extension SimulationWarningType {
    var suppressesOtherWarnings: Bool {
        switch self {
        case .externallyOwnedSpender, .validationError:
            true
        case .tokenApproval, .suspiciousSpender, .nftCollectionApproval, .permitApproval, .permitBatchApproval:
            false
        }
    }

    var isUnlimitedWarning: Bool {
        switch self {
        case .tokenApproval(_, let value), .permitApproval(_, let value), .permitBatchApproval(let value):
            value == nil
        case .suspiciousSpender, .externallyOwnedSpender, .nftCollectionApproval, .validationError:
            false
        }
    }
}
