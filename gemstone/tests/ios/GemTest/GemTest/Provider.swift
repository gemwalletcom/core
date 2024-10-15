// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public actor NativeProvider {
    let session: URLSession

    init(session: URLSession = .shared) {
        self.session = session
    }
}

extension AlienTarget {
    func asRequest() throws -> URLRequest {
        guard let url = URL(string: self.url) else {
            let error = AlienError.InvalidUrl(url: self.url)
            throw error
        }
        var request = URLRequest(url: url)
        request.httpMethod = self.method.description
        if let headers = self.headers {
            request.allHTTPHeaderFields = headers
        }
        if let body = self.body {
            request.httpBody = body
        }
        return request
    }
}

extension NativeProvider: AlienProvider {
    public func request(target: AlienTarget) async throws -> Data {
        let req = try target.asRequest()
        let (data, _) = try await session.data(for: req)
        return data
    }
}
