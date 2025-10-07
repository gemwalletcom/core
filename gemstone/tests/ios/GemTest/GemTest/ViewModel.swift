// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone

public struct ViewModel: Sendable {
    let provider = NativeProvider()

    public func fetchQuote(_ request: SwapperQuoteRequest) async throws {
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

    public func fetchQuoteById(_ request: SwapperQuoteRequest, provider: SwapProvider) async throws {
        let swapper = GemSwapper(rpcProvider: self.provider)
        let quote = try await swapper.fetchQuoteByProvider(provider: provider, request: request)
        self.dumpQuote(quote)

        try await self.fetchQuoteData(quote: quote)
    }

    public func fetchQuoteData(quote: SwapperQuote) async throws {
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

    func dumpQuote(_ quote: SwapperQuote) {
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
