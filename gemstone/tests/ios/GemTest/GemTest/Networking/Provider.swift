// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public actor NativeProvider {
    let nodeConfig: [String: URL]
    let session: URLSession
    let cache: Cache<AlienTarget, Data>

    init(session: URLSession = .shared) {
        self.nodeConfig = [
            "ethereum": URL(string: "https://eth.llamarpc.com")!,
            "optimism": URL(string: "https://optimism.llamarpc.com")!,
            "thorchain": URL(string: "https://thornode.ninerealms.com")!,
            "solana": URL(string: "https://solana-rpc.publicnode.com")!,
            "smartchain": URL(string: "https://binance.llamarpc.com")!,
            "arbitrum": URL(string: "https://arbitrum.llamarpc.com")!,
            "base": URL(string: "https://base.llamarpc.com")!,
            "polygon": URL(string: "https://polygon.llamarpc.com")!,
            "sui": URL(string: "https://sui-rpc.publicnode.com")!,
            "abstract": URL(string: "https://api.mainnet.abs.xyz")!,
            "unichain": URL(string: "https://mainnet.unichain.org")!,
            "ink": URL(string: "https://rpc-qnd.inkonchain.com")!
        ]
        self.session = session
        self.cache = Cache()
    }
}

extension NativeProvider: AlienProvider {
    public nonisolated func getEndpoint(chain: String) throws -> String {
        guard let url = nodeConfig[chain] else {
            throw AlienError.RequestError(msg: "\(chain) is not supported.")
        }
        return url.absoluteString
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
                    print("==> handle request: \(target)")
                    if let data = await self.cache.get(key: target) {
                        print("<== cached response size: \(data.count)")
                        return data
                    }

                    let (data, response) = try await self.session.data(for: target.asRequest())
                    if (response as? HTTPURLResponse)?.statusCode != 200 {
                        throw AlienError.ResponseError(msg: "invalid response: \(response)")
                    }
                    print("<== response size: \(data.count)")

                    // save cache
                    if let ttl = target.headers?["x-cache-ttl"] {
                        await self.cache.set(value: data, forKey: target, ttl: TimeInterval(ttl))
                    }
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
