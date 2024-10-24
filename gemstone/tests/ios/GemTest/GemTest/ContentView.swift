// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Gemstone

struct ContentView: View {

    let provider = NativeProvider()

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
                    do {
                        try await self.fetchQuote()
                    }
                    catch {
                        print(error)
                    }
                }
            }
            Button("Gas Price") {
                Task.detached {
                    do {
                        try await self.fetchGasPrice()
                    }
                    catch {
                        print(error)
                    }
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
        let warp = AlienProviderWarp(provider: provider)
        let data = try await warp.teleport(target: target)
        let json = try JSONSerialization.jsonObject(with: data)
        print(json)
    }

    func fetchQuote() async throws {
        // ETH -> USDC
        let string = """
        {
        "fromAsset": {
            "chain": "ethereum",
            "tokenId": null
        },
        "toAsset": {
            "chain": "ethereum",
            "tokenId": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        },
        "walletAddress": "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
        "destinationAddress": "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
        "amount": "10000000000000000",
        "mode": "exactin",
        "includeData": false
        }
        """

        let swapper = GemSwapper(rpcProvider: NativeProvider(), feeOptions: nil)
        let quote = try await swapper.fetchQuote(request: string)
        print(quote)
    }

    func fetchGasPrice() async throws {
        let provider = NativeProvider()
        let requests = [
            JsonRpcRequest(
                method: "eth_gasPrice",
                params: nil,
                id: 1
            ),
            JsonRpcRequest(
                method: "eth_blockNumber",
                params: nil,
                id: 2
            ),
            JsonRpcRequest(
                method: "eth_chainId",
                params: nil,
                id: 3
            ),
        ]
        let results = try await provider.batchJsonrpcCall(requests: requests, chain: "ethereum")

        print(results)
    }
}

#Preview {
    ContentView()
}
