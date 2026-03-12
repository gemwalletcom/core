// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Keystore
import Primitives
import WalletCore

struct SwapSigner {
    init() {}

    func isTransferSwap(fromAsset: Asset, data: SwapData) -> Bool {
        switch data.data.dataType {
        case .transfer: true
        case .contract: false
        }
    }

    func transferSwapInput(input: SignerInput, fromAsset: Asset, swapData: SwapData) throws -> SignerInput {
        let value = if input.useMaxAmount && fromAsset.id.type == .native {
            input.value
        } else {
            swapData.quote.fromValueBigInt
        }
        return SignerInput(
            type: .transfer(fromAsset),
            asset: fromAsset,
            value: value,
            fee: input.fee,
            isMaxAmount: input.useMaxAmount,
            memo: swapData.data.memo,
            senderAddress: input.senderAddress,
            destinationAddress: swapData.data.to,
            metadata: input.metadata
        )
    }

    func signSwap(signer: Signable, input: SignerInput, fromAsset: Asset, swapData: SwapData, privateKey: Data) throws -> [String] {
        let transferInput = try transferSwapInput(
            input: input,
            fromAsset: fromAsset,
            swapData: swapData
        )
        switch fromAsset.id.type {
        case .native: return try [signer.signTransfer(input: transferInput, privateKey: privateKey)]
        case .token: return try [signer.signTokenTransfer(input: transferInput, privateKey: privateKey)]
        }
    }
}
