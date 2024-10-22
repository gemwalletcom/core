// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public actor NativeProvider {
    let nodeConfig: [String: URL]
    let session: URLSession

    init(session: URLSession = .shared) {
        self.nodeConfig = [
            "ethereum": URL(string: "https://eth.llamarpc.com")!,
            "optimism": URL(string: "https://optimism.llamarpc.com")!
        ]
        self.session = session
    }
}

extension AlienTarget {
    func asRequest() throws -> URLRequest {
        guard let url = URL(string: self.url) else {
            let error = AlienError.RequestError(msg: "invalid url: \(self.url)")
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

extension JsonRpcRequest: @retroactive Encodable {
    enum CodingKeys: String, CodingKey {
        case jsonrpc
        case id
        case method
        case params
    }
    
    public func encode(to encoder: any Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode("2.0", forKey: .jsonrpc)
        try container.encode(self.id, forKey: .id)
        try container.encode(self.method, forKey: .method)

        if let params = self.params {
            let decoded = try JSONDecoder().decode([String].self, from: params.data(using: .utf8)!)
            try container.encode(decoded, forKey: .params)
        } else {
            try container.encodeNil(forKey: .params)
        }
    }
    
    func asRequest(url: URL) throws -> URLRequest {
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.allHTTPHeaderFields = ["Content-Type": "application/json"]
        request.httpBody = try JSONEncoder().encode(self)
        return request
    }
}

extension JsonRpcResult: @retroactive Decodable {
    enum CodingKeys: String, CodingKey {
        case id
        case result
        case error
    }
    
    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let id = try container.decode(UInt64.self, forKey: .id)
        if let result = try? container.decodeIfPresent(String.self, forKey: .result) {
            self = .value(JsonRpcResponse(result: result.data(using: .utf8), error: nil, id: id))
            return
        }
        if let error = try? container.decodeIfPresent(JsonRpcError.self, forKey: .error) {
            self = .error(error)
        }
        throw DecodingError.dataCorruptedError(forKey: .result, in: container, debugDescription: "")
    }
}

extension JsonRpcError: @retroactive Decodable {
    enum CodingKeys: String, CodingKey {
        case code
        case message
    }
    
    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let code = try container.decode(Int32.self, forKey: .code)
        let message = try container.decode(String.self, forKey: .message)
        self = .init(code: code, message: message)
    }
}

extension NativeProvider: AlienProvider {
    public func request(target: AlienTarget) async throws -> Data {
        let req = try target.asRequest()
        let (data, _) = try await session.data(for: req)
        return data
    }

    public func jsonrpcCall(requests: [JsonRpcRequest], chain: Chain) async throws -> [JsonRpcResult] {
        let url = nodeConfig[chain]!
        var results = [JsonRpcResult]()
        for request in requests {
            let req = try request.asRequest(url: url)
            let (data, response) = try await session.data(for: req)
            if (response as? HTTPURLResponse)?.statusCode != 200 {
                throw AlienError.ResponseError(msg: "invalid response: \(response)")
            }
            let result = try JSONDecoder().decode(JsonRpcResult.self, from: data)
            results.append(result)
        }
        return results
    }
}
