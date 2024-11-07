// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public actor NativeProvider {
    let nodeConfig: [String: URL]
    let session: URLSession

    init(session: URLSession = .shared) {
        self.nodeConfig = [
            "ethereum": URL(string: "https://eth.llamarpc.com")!,
            "optimism": URL(string: "https://optimism.llamarpc.com")!,
            "solana": URL(string: "https://api.mainnet-beta.solana.com/")!
        ]
        self.session = session
    }
}

extension NativeProvider: AlienProvider {
    nonisolated public func getEndpoint(chain: Chain) throws -> String {
        return nodeConfig[chain]!.absoluteString
    }

    public func request(target: Gemstone.AlienTarget) async throws -> Data {
        let results = try await self.batchRequest(targets: [target])
        guard
            results.count == 1
        else {
            throw AlienError.ResponseError(msg: "invalid response: \(target)")
        }
        return results[0]
    }

    public func batchRequest(targets: [AlienTarget]) async throws -> [Data] {
        return try await withThrowingTaskGroup(of: Data.self) { group in
            var results = [Data]()

            for target in targets {
                group.addTask {
                    print("==> handle request:\n\(target)")
                    let (data, response) = try await self.session.data(for: target.asRequest())
                    if (response as? HTTPURLResponse)?.statusCode != 200 {
                        throw AlienError.ResponseError(msg: "invalid response: \(response)")
                    }
                    print("<== response:\n\(String(decoding: data, as: UTF8.self))")
                    return data
                }
            }
            for try await result in group {
                results.append(result)
            }

            return results
        }
    }
}
