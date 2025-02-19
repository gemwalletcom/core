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

extension SwapQuoteData: @retroactive CustomStringConvertible {
    public var description: String {
        var json: [String: Any] = [
            "to": to,
            "value": value,
            "data": data,
            "gasLimit": gasLimit ?? NSNull(),
        ]
        if let approvalData = approval {
            json["approval"] = [
                "token": approvalData.token,
                "spender": approvalData.spender,
                "value": approvalData.value,
            ]
        }
        let bytes = try! JSONSerialization.data(withJSONObject: json, options: [.prettyPrinted, .sortedKeys])
        return String(data: bytes, encoding: .utf8)!
    }
}
