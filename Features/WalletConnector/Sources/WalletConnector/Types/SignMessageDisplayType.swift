// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public enum SignMessageDisplayType: Sendable {
    case payload(primary: [SimulationPayloadField], secondary: [SimulationPayloadField])
    case text(String)
}
