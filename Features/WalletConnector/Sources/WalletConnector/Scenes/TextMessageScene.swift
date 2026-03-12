// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Localization
import Style
import PrimitivesComponents
import Components

struct TextMessageScene: View {
    @State private var isPresentingShareSheet = false

    let model: TextMessageViewModel
    
    var body: some View {
        ScrollView {
            Text(model.text)
                .font(.caption)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding()
        }
        .toolbarContent {
            ToolbarItemView(placement: .topBarTrailing) {
                Button {
                    isPresentingShareSheet.toggle()
                } label: {
                    Images.System.share
                }
            }
        }
        .background(Colors.grayBackground)
        .navigationTitle(Localized.SignMessage.message)
        .navigationBarTitleDisplayMode(.inline)
        .sheet(isPresented: $isPresentingShareSheet) {
            ShareSheet(activityItems: [model.text])
        }
    }
}
