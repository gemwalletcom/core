// Copyright (c). Gem Wallet. All rights reserved.
import Foundation
import Gemstone

extension AlienTarget: URLRequestConvertible {
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
    func dictionary() throws -> [String: Any] {
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
        return json
    }
    func encode() throws -> Data {
        let json = try dictionary()
        return try JSONSerialization.data(withJSONObject: json, options: [.sortedKeys])
    }
}

extension Array where Element == JsonRpcRequest {
    func encode() throws -> Data {
        var array = [[String: Any]]()
        for request in self {
            array.append(try request.dictionary())
        }
        return try JSONSerialization.data(withJSONObject: array, options: [.sortedKeys])
    }

    func asRequest(url: URL) throws -> URLRequest {
        var req = URLRequest(url: url)
        req.httpMethod = "POST"
        req.allHTTPHeaderFields = ["Content-Type": "application/json"]
        req.httpBody = try encode()
        return req
    }
}

struct JsonRpcTarget: URLRequestConvertible {
    let request: JsonRpcRequest
    let url: URL

    func asRequest() throws -> URLRequest {
        var req = URLRequest(url: url)
        req.httpMethod = "POST"
        req.allHTTPHeaderFields = ["Content-Type": "application/json"]
        req.httpBody = try request.encode()
        return req
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
            self = .value(JsonRpcResponse(result: result, error: nil, id: id))
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
