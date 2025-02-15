// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

extension SwapQuote: @retroactive CustomStringConvertible {
    public var description: String {
        let routes: [[String: Any]] = data.routes.map { route in
            [
                "input": route.input,
                "output": route.output,
                "routeData": route.routeData,
                "gasLimit": route.gasLimit ?? "",
            ]
        }
        let json: [String: Any] = [
            "fromValue": fromValue,
            "toValue": toValue,
            "data": [
                "provider": swapProviderNameToString(provider: data.provider),
                "slippageBps": data.slippageBps,
                "routes": routes,
            ],
        ]
        let bytes = try! JSONSerialization.data(withJSONObject: json, options: [.prettyPrinted, .sortedKeys])
        return String(data: bytes, encoding: .utf8)!
    }
}
