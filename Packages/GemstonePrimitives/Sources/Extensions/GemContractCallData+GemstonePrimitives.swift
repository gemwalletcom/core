// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

extension GemContractCallData {
    public func map() -> ContractCallData {
        ContractCallData(
            contractAddress: contractAddress,
            callData: callData,
            approval: approval?.map(),
            gasLimit: gasLimit
        )
    }
}

extension ContractCallData {
    public func map() -> GemContractCallData {
        GemContractCallData(
            contractAddress: contractAddress,
            callData: callData,
            approval: approval?.map(),
            gasLimit: gasLimit
        )
    }
}
