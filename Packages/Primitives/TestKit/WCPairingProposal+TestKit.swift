// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public extension WCPairingProposal {
    static func mock(
        pairingId: String = "pairing-id",
        proposal: WalletConnectionSessionProposal = .mock(),
        verificationStatus: WalletConnectionVerificationStatus = .verified
    ) -> WCPairingProposal {
        WCPairingProposal(
            pairingId: pairingId,
            proposal: proposal,
            verificationStatus: verificationStatus
        )
    }
}
