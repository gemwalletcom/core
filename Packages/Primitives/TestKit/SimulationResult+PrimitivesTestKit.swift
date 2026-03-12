// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

extension SimulationResult {
    public static func mock(
        warnings: [SimulationWarning] = [],
        balanceChanges: [SimulationBalanceChange] = [],
        payload: [SimulationPayloadField] = [],
        header: SimulationHeader? = nil
    ) -> SimulationResult {
        SimulationResult(
            warnings: warnings,
            balanceChanges: balanceChanges,
            payload: payload,
            header: header
        )
    }
}
