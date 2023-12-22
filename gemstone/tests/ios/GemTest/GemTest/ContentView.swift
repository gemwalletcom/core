//
//  ContentView.swift
//  GemTest
//
//  Created by magician on 20/12/23.
//

import SwiftUI
import Gemstone

struct ContentView: View {
    var body: some View {
        VStack {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Text(Gemstone.rustGreeting(name: "GEM"))
        }
        .padding()
        .onAppear {}
    }
}

#Preview {
    ContentView()
}
