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
    public func getEndpoint(chain: Chain) async throws -> String {
        return nodeConfig[chain]!.absoluteString
    }

    public func request(targets: [AlienTarget]) async throws -> [Data] {
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
