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

    public func request(target: Gemstone.AlienTarget) async throws -> Gemstone.AlienResponse {
        print("==> handle request: \(target)")

        if let data = await self.cache.get(key: target) {
            print("<== cached response size: \(data.count)")
            return Gemstone.AlienResponse(status: nil, data: data)
        }

        let (data, response) = try await self.session.data(for: target.asRequest())
        let status = (response as? HTTPURLResponse)?.statusCode

        print("<== response size: \(data.count)")

        if let ttl = target.headers?["x-cache-ttl"] {
            await self.cache.set(value: data, forKey: target, ttl: TimeInterval(ttl))
        }

        return Gemstone.AlienResponse(status: status.map(UInt16.init), data: data)
    }
}
