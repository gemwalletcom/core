// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Gemstone

struct ContentView: View {

    let provider = NativeProvider()

    let eth2usdcRequest: SwapQuoteRequest = SwapQuoteRequest(
        fromAsset: "ethereum",
        toAsset: "ethereum_0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        walletAddress: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
        destinationAddress: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
        value: "100000000000000000", // 0.01 ETH
        mode: .exactIn,
        options: nil
    )

    let usdc2ethRequest: SwapQuoteRequest = SwapQuoteRequest(
        fromAsset: "ethereum_0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        toAsset: "ethereum",
        walletAddress: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
        destinationAddress: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
        value: "100000000", // 100 USDC
        mode: .exactIn,
        options: nil
    )

    let sol2usdcRequest: SwapQuoteRequest = SwapQuoteRequest(
        fromAsset: "solana",
        toAsset: "solana_EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        walletAddress: "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC",
        destinationAddress: "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC",
        value: "1000000000", // 1 SOL
        mode: .exactIn,
        options: nil
    )

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
            Button("List Providers") {
                self.fetchProviders()
            }
            Button("Fetch ETH Quote") {
                Task.detached {
                    do {
                        try await self.fetchQuote(self.eth2usdcRequest)
                    }
                    catch {
                        print(error)
                    }
                }
            }
            Button("Fetch SOL Quote") {
                Task.detached {
                    do {
                        try await self.fetchQuote(self.sol2usdcRequest)
                    }
                    catch {
                        print(error)
                    }
                }
            }
            Button("Decode Tick Array Test") {
                self.decodeTickArray()
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

    func fetchQuote(_ request: SwapQuoteRequest) async throws {
        let swapper = GemSwapper(rpcProvider: self.provider)
        guard let quote = try await swapper.fetchQuote(request: request).first else {
            return print("<== fetchQuote: nil")
        }
        print("<== fetchQuote:\n", quote)

        let data = try await swapper.fetchQuoteData(quote: quote, data: .none)
        print("<== fetchQuoteData:\n", data)
    }

    func fetchProviders() {
        let swapper = GemSwapper(rpcProvider: self.provider)
        print("<== getProviders:\n", swapper.getProviders())
    }

    func decodeTickArray() {
        Task.detached {
            do {
                let ok = try await Gemstone.decodeTickArray()
                print("decodeTickArray is \(ok)")
            }
            catch {
                print(error)
            }
        }
    }
}

#Preview {
    ContentView()
}
