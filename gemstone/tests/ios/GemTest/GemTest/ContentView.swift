// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Gemstone

struct ContentView: View {

    let warp: AlienProviderWarp

    init() {
        let warp = AlienProviderWarp(provider: NativeProvider())
        self.warp = warp
    }

    var body: some View {
        VStack {
            Image(systemName: "diamond")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Text("Gemstone lib version: " + Gemstone.libVersion())
            Button("Post Data") {
                Task.detached {
                    try await self.fetchData()
                }
            }
            Button("Fetch Quote") {
                Task.detached {
                    try await self.fetchQuote()
                }
            }
            Button("Gas Price") {
                Task.detached {
                    try await self.fetchGasPrice()
                }
            }
        }
        .padding()
        .onAppear {}
    }

    func fetchData() async throws {
        let headers = [
            "X-Header": "X-Value",
            "Content-Type": "application/json"
        ]
        let body = try JSONEncoder().encode(["foo": "bar"])
        let target = AlienTarget(
            url: "https://httpbin.org/post?foo=bar",
            method: "POST",
            headers: headers,
            body: body
        )
        let data = try await warp.teleport(target: target)
        let json = try JSONSerialization.jsonObject(with: data)
        print(json)
    }

    func fetchQuote() async throws {
        let json = """
        {
        "fromAsset": {
            "chain": "ethereum",
            "tokenId": null
        },
        "toAsset": {
            "chain": "ethereum",
            "tokenId": null
        },
        "walletAddress": "0x1234567890abcdef",
        "destinationAddress": "0x1234567890abcdef",
        "amount": "0.0",
        "mode": "exactin",
        "includeData": false
        }
        """
        let swapper = GemSwapper(rpcProvider: NativeProvider())
        let quote = try await swapper.fetchQuote(request: json)
        print(quote)
    }

    func fetchGasPrice() async throws {
        let provider = NativeProvider()
        let request = JsonRpcRequest(
            method: "eth_gasPrice",
            params: nil,
            id: 1
        )
        let results = try await provider.jsonrpcCall(requests: [request], chain: "ethereum")

        print(results)
    }
}

#Preview {
    ContentView()
}
