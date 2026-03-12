// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Primitives
import Style

public struct TransactionView: View {
    @State private var isPresentingUrl: URL? = nil
    private let model: TransactionViewModel

    public init(model: TransactionViewModel) {
        self.model = model
    }

    public var body: some View {
        ListItemView(
            title: model.titleTextValue,
            titleExtra: model.titleExtraTextValue,
            titleTag: model.titleTagTextValue,
            titleTagType: model.titleTagType,
            subtitle: model.subtitleTextValue,
            subtitleExtra: model.subtitleExtraTextValue,
            imageStyle: .asset(assetImage: model.assetImage)
        )
        .contextMenu(
            .url(title: model.viewOnTransactionExplorerText, onOpen: { isPresentingUrl = model.transactionExplorerUrl })
        )
        .safariSheet(url: $isPresentingUrl)
    }
}

// MARK: - Previews

private struct ExplorerMock: ExplorerLinkFetchable {
    func addressUrl(chain: Chain, address: String) -> BlockExplorerLink {
       .init(name: "", link: "")
    }
    func transactionUrl(chain: Chain, hash: String) -> BlockExplorerLink {
       .init(name: "", link: "")
    }
    func swapTransactionUrl(chain: Chain, provider: String, identifier: String) -> BlockExplorerLink? {
        .init(name: "", link: "")
    }
}

#Preview {
    let pendingTransactionMock = Transaction(
        id: TransactionId(chain: .smartChain, hash: "0xe5fb66cef0fb71fa75e0245484a40d17952cf46053724c6ac61209bf307d6e56"),
        assetId: .init(chain: .smartChain, tokenId: ""),
        from: "0x92abCE21234D71EC443E679f3a1feAFD3Fc830fB",
        to: "0x8d7460E51bCf4eD26877cb77E56f3ce7E9f5EB8F",
        contract: nil,
        type: .transfer,
        state: .pending,
        blockNumber: "39348339",
        sequence: "1",
        fee: "21000000000000",
        feeAssetId: .init(chain: .smartChain, tokenId: ""),
        value: "100000000000000",
        memo: nil,
        direction: .outgoing,
        utxoInputs: [],
        utxoOutputs: [],
        metadata: nil,
        createdAt: Date()
    )
    let pendingTransactionExtendedMock = TransactionExtended(
        transaction: pendingTransactionMock,
        asset: .init(.smartChain),
        feeAsset: .init(.smartChain),
        price: nil,
        feePrice: nil,
        assets: [],
        prices: [],
        fromAddress: AddressName(chain: .smartChain, address: "0x92abCE21234D71EC443E679f3a1feAFD3Fc830fB", name: "test1", type: nil, status: .verified),
        toAddress: AddressName(chain: .smartChain, address: "0x8d7460E51bCf4eD26877cb77E56f3ce7E9f5EB8F", name: "test2", type: nil, status: .verified)
    )

    let transactionVMMock = TransactionViewModel(
        explorerService: ExplorerMock(),
        transaction: pendingTransactionExtendedMock,
        currency: Currency.usd.rawValue
    )

    TransactionView(model: transactionVMMock)
}
