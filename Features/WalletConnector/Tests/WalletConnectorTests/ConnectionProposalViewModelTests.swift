// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Primitives
import PrimitivesTestKit

@testable import WalletConnector

struct ConnectionProposalViewModelTests {

    @Test
    func appTextKeepsNameAndDomain() {
        let metadata = WalletConnectionSessionAppMetadata.mock(
            name: "PancakeSwap - Trade",
            url: "https://pancakeswap.finance/swap"
        )
        let model = ConnectionProposalViewModel(
            confirmTransferDelegate: { _ in },
            pairingProposal: .mock(proposal: .mock(metadata: metadata))
        )

        #expect(model.appText == "PancakeSwap (pancakeswap.finance)")
    }
}
