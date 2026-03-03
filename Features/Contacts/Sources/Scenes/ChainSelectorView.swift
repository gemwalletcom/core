// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives

struct ChainSelectorView: View {

    @Environment(\.dismiss) private var dismiss

    let chain: Chain?
    let onSelectChain: (Chain) -> Void

    var body: some View {
        ChainSelectorScene(
            chain: chain,
            onSelectChain: {
                onSelectChain($0)
                dismiss()
            }
        )
    }
}
