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
            Button("Fetch data") {
                Task.detached {
                    try await self.fetchData()
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
}

#Preview {
    ContentView()
}
