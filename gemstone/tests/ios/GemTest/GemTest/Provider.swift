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

extension NativeProvider: AlienProvider {
    public func request(target: AlienTarget) async throws -> Data {
        print("==> handle request: \(target)")
        let req = try target.asRequest()
        let (data, _) = try await session.data(for: req)
        return data
    }

    public func jsonrpcCall(requests: [JsonRpcRequest], chain: Chain) async throws -> [JsonRpcResult] {
        let url = nodeConfig[chain]!
        let targets = requests.map { JsonRpcTarget(request: $0, url: url) }
        return try await withThrowingTaskGroup(of: JsonRpcResult.self) { group in
            var results = [JsonRpcResult]()

            for target in targets {
                group.addTask {
                    let (data, response) = try await self.session.data(for: target.asRequest())
                    if (response as? HTTPURLResponse)?.statusCode != 200 {
                        throw AlienError.ResponseError(msg: "invalid response: \(response)")
                    }
                    print("<== response: \(String(decoding: data, as: UTF8.self))")
                    return try JSONDecoder().decode(JsonRpcResult.self, from: data)
                }
            }
            for try await result in group {
                results.append(result)
            }

            return results
        }
    }

    public func batchJsonrpcCall(requests: [JsonRpcRequest], chain: Chain) async throws -> [JsonRpcResult] {
        let url = nodeConfig[chain]!
        let req = try requests.asRequest(url: url)
        let (data, _) = try await session.data(for: req)
        return try JSONDecoder().decode([JsonRpcResult].self, from: data)
    }
}
