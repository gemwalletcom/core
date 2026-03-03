// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

extension ContractCallData {
    public static func mock(
        contractAddress: String = "",
        callData: String = "",
        approval: ApprovalData? = nil,
        gasLimit: String? = nil
    ) -> ContractCallData {
        ContractCallData(
            contractAddress: contractAddress,
            callData: callData,
            approval: approval,
            gasLimit: gasLimit
        )
    }
}
