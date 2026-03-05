// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Preferences
import WebSocketClient
import GemAPI
import DeviceService

public struct AuthenticatedRequestProvider: WebSocketRequestProvider {
    private let securePreferences: SecurePreferences

    public init(securePreferences: SecurePreferences) {
        self.securePreferences = securePreferences
    }

    public func makeRequest() throws -> URLRequest {
        let keyPair = try DeviceService.getOrCreateKeyPair(securePreferences: securePreferences)
        let signer = try DeviceRequestSigner(privateKey: keyPair.privateKey)

        var request = URLRequest(url: Constants.deviceStreamWebSocketURL)
        request.httpMethod = "GET"
        try signer.sign(request: &request)
        return request
    }
}
