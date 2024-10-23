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

extension JsonRpcRequest {
    func encode() throws -> Data {
        var json: [String: Any] = [
            "jsonrpc": "2.0",
            "id": self.id,
            "method": self.method
        ]
        if let params = params {
            json["params"] = try JSONSerialization.jsonObject(with: Data(params.utf8))
        } else {
            json["params"] = NSNull()
        }
        return try JSONSerialization.data(withJSONObject: json, options: [.sortedKeys])
    }

    func asRequest(url: URL) throws -> URLRequest {
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.allHTTPHeaderFields = ["Content-Type": "application/json"]
        request.httpBody = try encode()
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
        } else if let error = try? container.decodeIfPresent(JsonRpcError.self, forKey: .error) {
            self = .error(error)
        } else {
            throw DecodingError.dataCorruptedError(forKey: .result, in: container, debugDescription: "")
        }
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
