// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

struct AddAssetInput {
    
    let chains: [Chain]

    var chain: Chain?
    var address: String?

    var hasManyChains: Bool {
        chains.count > 1
    }

    init(chains: [Chain]) {
        self.chains = chains
        self.chain = chains.first
    }
}
