// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import BigInt

public struct ViewModel: Sendable {
    let provider = NativeProvider()

    public func testFetchData() async throws {
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

    public func fetchQuote(_ request: SwapQuoteRequest) async throws {
        let swapper = GemSwapper(rpcProvider: self.provider)
        var quotes = try await swapper.fetchQuote(request: request)
        quotes = quotes.sorted(by: { lhs, rhs in
            BigInt(lhs.toValue)! > BigInt(rhs.toValue)!
        })
        print("<== quotes: \(quotes.count)")
        guard
            let quote = quotes.first,
            let _ = quote.data.routes.first
        else {
            return print("<== fetchQuote: nil")
        }

        for quote in quotes {
            self.dumpQuote(quote)
        }

        try await self.fetchQuoteData(quote: quote)
    }

    public func fetchQuoteById(_ request: SwapQuoteRequest, provider: SwapProvider) async throws {
        let swapper = GemSwapper(rpcProvider: self.provider)
        let quote = try await swapper.fetchQuoteByProvider(provider: provider, request: request)
        self.dumpQuote(quote)

        try await self.fetchQuoteData(quote: quote)
    }

    public func fetchQuoteData(quote: SwapQuote) async throws {
        let swapper = GemSwapper(rpcProvider: self.provider)

        if let permit2 = try await swapper.fetchPermit2ForQuote(quote: quote) {
            print("<== permit2", permit2)
        }

        let data = try await swapper.fetchQuoteData(quote: quote, data: .none)
        print("<== fetchQuoteData:\n", data)
    }

    public func fetchProviders() {
        let swapper = GemSwapper(rpcProvider: self.provider)
        print("<== getProviders:\n", swapper.getProviders())
    }

    public func fetchSolanaPay(uri: String) async throws {
        let wrapper = try paymentDecodeUrl(string: uri)
        guard let url = wrapper.paymentLink else {
            print("invalid url")
            return
        }
        do {
            let solanaPay = SolanaPay(provider: self.provider)
            async let labelCall = solanaPay.getLabel(link: url)
            async let txCall = solanaPay.postAccount(link: url, account: TEST_SOL_WALLET)

            let (label, tx) = try await (labelCall, txCall)
            print(label, tx)
        } catch {
            print(error)
        }
    }

    func dumpQuote(_ quote: SwapQuote) {
        let route = quote.data.routes.first!
        print("<== fetchQuote:\n", quote.description)
        print("==> amount out: \(quote.toValue)")
        print("==> routes count: \(quote.data.routes.count), route data: \(route.routeData)")
        if quote.data.routes.count > 1 {
            print("==> intermediary token: \(route.output)")
        }
        print("suggested slippageBps: \(quote.data.slippageBps)")
    }
}
