// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import SwiftUI

struct ContentView: View {
    let model = ViewModel()

    var body: some View {
        VStack(alignment: .leading) {
            HStack {
                Image(systemName: "diamond")
                    .imageScale(.large)
                    .foregroundStyle(.tint)
                Text("Gemstone lib version: " + Gemstone.libVersion())
            }
            Button("Post Data") {
                Task {
                    try await self.model.testFetchData()
                }
            }
            Button("List Providers") {
                Task {
                    self.model.fetchProviders()
                }
            }
            Text("Swap:")
            Button("Fetch ETH -> USDC") {
                self.testQuote(quote: .eth2usdc)
            }
            Button("Fetch ETH -> BTC") {
                self.testQuote(quote: .eth2btc, id: .thorchain)
            }
            Button("Fetch ETH -> SOL") {
                self.testQuote(quote: .eth2sol, id: .deBridge)
            }
            Button("Fetch v4 ETH -> USDC") {
                self.testQuote(quote: .eth2usdc_v4, id: .uniswapV4)
            }
            Button("Fetch v4 UNI -> DAI") {
                self.testQuote(quote: .uni2dai_v4)
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
            Button("Fetch Cake -> BNB") {
                self.testQuote(quote: .cake2bnb)
            }
            Button("Fetch Cake -> BTCB") {
                self.testQuote(quote: .cake2btcb)
            }
            Button("Fetch ETH on ABS -> USDC") {
                self.testQuote(quote: .absETH2USDC)
            }
            Button("Fetch SUI -> USDC") {
                self.testQuote(quote: .sui2USDC)
            }
            Text("Bridge:")
            Button("Bridge Op ETH -> Ethereum") {
                self.testQuote(quote: .op2Eth)
            }
            Button("Bridge Op ETH -> Arbitrum") {
                self.testQuote(quote: .op2Arb)
            }
            Button("Bridge Op ETH -> Ink") {
                self.testQuote(quote: .op2Ink)
            }
            Button("Bridge ETH -> Unichain") {
                self.testQuote(quote: .eth2Unichain)
            }
            Button("Bridge ETH USDC -> Base") {
                self.testQuote(quote: .ethUSDC2Base)
            }
            Button("Bridge Base USDC -> ETH") {
                self.testQuote(quote: .baseUSDC2Eth)
            }
            Text("Solana Pay:")
            Button("Paste URI") {
                guard let text = UIPasteboard.general.string else {
                    return
                }
                Task {
                    try await self.model.fetchSolanaPay(uri: text)
                }
            }
        }
        .padding()
        .onAppear {}
    }

    func testQuote(quote: SwapQuoteRequest) {
        Task {
            do {
                try await self.model.fetchQuote(quote)
            } catch {
                print(error)
            }
        }
    }

    func testQuote(quote: SwapQuoteRequest, id: SwapProvider) {
        Task {
            do {
                try await self.model.fetchQuoteById(quote, provider: id)
            } catch {
                print(error)
            }
        }
    }
}
