// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

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
        guard
            let quote = try await swapper.fetchQuote(request: request).first,
            let route = quote.data.routes.first
        else {
            return print("<== fetchQuote: nil")
        }

        print("<== fetchQuote:\n", quote)
        print("==> amount out: \(quote.toValue)")
        print("==> routes count: \(quote.data.routes.count), route data: \(route.routeData)")
        if quote.data.routes.count > 1 {
            print("==> intermediary token: \(route.output)")
        }
        print("suggested slippageBps: \(quote.data.slippageBps)")

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
}
