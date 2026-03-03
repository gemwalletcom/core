// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Keystore
import Primitives
import WalletCore

public struct Signer: Sendable {
    let wallet: Primitives.Wallet
    let keystore: any Keystore

    public init(
        wallet: Primitives.Wallet,
        keystore: any Keystore
    ) {
        self.wallet = wallet
        self.keystore = keystore
    }

    func sign(input: SignerInput, chain: Chain, privateKey: Data) throws -> [String] {
        let signer = signer(for: chain)
        switch input.type {
        case .transfer(let asset), .deposit(let asset):
            switch asset.id.type {
            case .native:
                return try [signer.signTransfer(input: input, privateKey: privateKey)]
            case .token:
                return try [signer.signTokenTransfer(input: input, privateKey: privateKey)]
            }
        case .transferNft:
            return try [signer.signNftTransfer(input: input, privateKey: privateKey)]
        case .tokenApprove:
            return try [signer.signTokenTransfer(input: input, privateKey: privateKey)]
        case .swap(let fromAsset, _, let swapData):
            let swapSigner = SwapSigner()
            if swapSigner.isTransferSwap(fromAsset: fromAsset, data: swapData) {
                return try swapSigner
                    .signSwap(
                        signer: signer,
                        input: input,
                        fromAsset: fromAsset,
                        swapData: swapData,
                        privateKey: privateKey
                    )
            }
            return try signer.signSwap(input: input, privateKey: privateKey)
        case .generic:
            return try [signer.signData(input: input, privateKey: privateKey)]
        case .stake:
            return try signer.signStake(input: input, privateKey: privateKey)
        case .earn:
            return try signer.signEarn(input: input, privateKey: privateKey)
        case .account:
            return try [signer.signAccountAction(input: input, privateKey: privateKey)]
        case .perpetual:
            return try signer.signPerpetual(input: input, privateKey: privateKey)
        case .withdrawal:
            return try [signer.signWithdrawal(input: input, privateKey: privateKey)]
        }
    }

    public func sign(input: SignerInput) async throws -> [String] {
        let chain = input.asset.chain
        var privateKey = try await keystore.getPrivateKey(wallet: wallet, chain: chain)
        defer { privateKey.zeroize() }
        return try sign(input: input, chain: chain, privateKey: privateKey)
    }

    public func signMessage(chain: Chain, message: SignMessage) async throws -> String {
        var privateKey = try await keystore.getPrivateKey(wallet: wallet, chain: chain)
        defer { privateKey.zeroize() }
        return try signer(for: chain).signMessage(message: message, privateKey: privateKey)
    }

    func signer(for chain: Chain) -> Signable {
        switch chain.type {
        case .solana: SolanaSigner()
        case .ethereum: EthereumSigner()
        case .cosmos: CosmosSigner()
        case .ton: TonSigner()
        case .tron: TronSigner()
        case .bitcoin: BitcoinSigner()
        case .xrp: XrpSigner()
        case .near: NearSigner()
        case .stellar: StellarSigner()
        case .algorand: AlgorandSigner()
        case .polkadot: PolkadotSigner()
        case .cardano: CardanoSigner()
        case .sui, .hyperCore, .aptos: ChainSigner(chain: chain)
        }
    }
}
