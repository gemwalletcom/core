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
        let data = try await warp.teleport(targets: [target])
        let json = try JSONSerialization.jsonObject(with: data[0])
        print(json)
    }

    func fetchQuote() async throws {
        // ETH -> USDC
        let request = SwapQuoteRequest(
            fromAsset: "ethereum",
            toAsset: "ethereum_0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
            walletAddress: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
            destinationAddress: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
            value: "100000000000000000",
            mode: .exactIn,
            options: nil
        )

        let swapper = GemSwapper(rpcProvider: NativeProvider())
        guard let quote = try await swapper.fetchQuote(request: request).first else {
            return print("<== fetchQuote: nil")
        }
        print("<== fetchQuote:\n", quote)

        let data = try await swapper.fetchQuoteData(quote: quote, permit2: nil)
        print("<== fetchQuoteData:\n", data)
    }
}

#Preview {
    ContentView()
}
