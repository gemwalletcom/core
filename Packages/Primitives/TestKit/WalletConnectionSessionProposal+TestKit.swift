// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public extension WalletConnectionSessionProposal {
    static func mock(
        defaultWallet: Wallet = .mock(),
        wallets: [Wallet] = [.mock()],
        metadata: WalletConnectionSessionAppMetadata = .mock()
    ) -> WalletConnectionSessionProposal {
        WalletConnectionSessionProposal(
            defaultWallet: defaultWallet,
            wallets: wallets,
            metadata: metadata
        )
    }
}
