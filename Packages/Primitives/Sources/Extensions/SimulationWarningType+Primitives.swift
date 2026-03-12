
public extension SimulationWarningType {
    var approvalValue: ApprovalValue? {
        switch self {
        case .tokenApproval(_, let value), .permitApproval(_, let value), .permitBatchApproval(let value):
            guard let value else {
                return ApprovalValue.unlimited
            }
            return ApprovalValue.exact(value)
        case .suspiciousSpender, .externallyOwnedSpender, .nftCollectionApproval, .validationError:
            return nil
        }
    }
}
