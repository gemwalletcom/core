// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import SwiftUI

struct ContentView: View {
    let provider = NativeProvider()

    var body: some View {
        VStack(alignment: .leading) {
            HStack {
                Image(systemName: "diamond")
                    .imageScale(.large)
                    .foregroundStyle(.tint)
                Text("Gemstone lib version: " + Gemstone.libVersion())
            }
            Button("Post Data") {
                Task.detached {
                    try await self.fetchData()
                }
            }
            Button("List Providers") {
                self.fetchProviders()
            }
            Text("Quotes:")
            Button("Fetch ETH -> USDC") {
                self.testQuote(quote: .eth2usdc)
            }
            Button("Fetch SOL -> USDC") {
                self.testQuote(quote: .sol2usdc)
            }
            Button("Fetch JUP -> BONK") {
                self.testQuote(quote: .jup2bonk)
            }
            Button("Fetch UNI -> LINK") {
                self.testQuote(quote: .uni2link)
            }
            Button("Fetch Cake -> BTCB") {
                self.testQuote(quote: .cake2btcb)
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
            method: .post,
            headers: headers,
            body: body
        )
        let warp = AlienProviderWarp(provider: provider)
        let data = try await warp.teleport(targets: [target])
        let json = try JSONSerialization.jsonObject(with: data[0])
        print(json)
    }

    func testQuote(quote: SwapQuoteRequest) {
        Task {
            do {
                try await self.fetchQuote(quote)
            }
            catch {
                print(error)
            }
        }
    }

    func fetchQuote(_ request: SwapQuoteRequest) async throws {
        let swapper = GemSwapper(rpcProvider: self.provider)
        guard
            let quote = try await swapper.fetchQuote(request: request).first,
            let route = quote.data.routes.first
        else {
            return print("<== fetchQuote: nil")
        }

        print("<== fetchQuote:\n", quote)
        print("==> amount out: \(quote.toValue)")
        print("==> routes count: \(quote.data.routes.count), fee tier: \(route.routeData)")
        if quote.data.routes.count > 1 {
            print("==> intermediary token: \(route.output)")
        }
        if let slippage = quote.data.suggestedSlippageBps {
            print("suggested slippageBps: \(slippage)")
        }

        let data = try await swapper.fetchQuoteData(quote: quote, data: .none)
        print("<== fetchQuoteData:\n", data)
    }

    func fetchProviders() {
        let swapper = GemSwapper(rpcProvider: self.provider)
        print("<== getProviders:\n", swapper.getProviders())
    }
}

#Preview {
    ContentView()
}
